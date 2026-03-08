//! ZTE GT5S (G5TS) CPE client
//!
//! Uses ubus JSON-RPC 2.0 over HTTPS, completely different from the MF289F goform API.
//!
//! ## HTTP Workflow
//!
//! All requests are POST to `https://<ip>/ubus/?t=<unix_timestamp>` with:
//! - `Content-Type: application/json`
//! - `Referer: https://<ip>/` (required or router returns 307 redirect)
//!
//! Request body is a JSON array of RPC calls:
//! ```json
//! [{"jsonrpc":"2.0","id":<n>,"method":"call","params":["<session>","<module>","<method>",{<args>}]}]
//! ```
//!
//! ### Authentication
//! 1. Fetch salt: `["00...0", "zwrt_web", "web_login_info", {}]`
//!    → `{"zte_web_sault": "<hex>", "login_fail_num": <n>}`
//! 2. Compute password: `SHA256(SHA256(password).UPPER + salt).UPPER`
//! 3. Login: `["00...0", "zwrt_web", "web_login", {"password": "<hash>"}]`
//!    → `{"result": 0, "ubus_rpc_session": "<32hex>", "timeout": 300}`
//! 4. Use returned session token for all subsequent calls.

#[cfg(test)]
mod tests;

pub mod commands;

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use anyhow::{anyhow, bail, Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use reqwest::header::{CONTENT_TYPE, REFERER};
use rsa::pkcs8::DecodePublicKey;
use rsa::Pkcs1v15Encrypt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bands::LteBand;
use crate::{
    ApnAuthMode, ApnProfile, BearerPreference, ConnectionMode, DhcpSettings, MtuSettings,
    PdpType, RouterClient, SmsSettings, normalize_router_url,
};
use commands::UbusCommand;

const NULL_SESSION: &str = "00000000000000000000000000000000";

fn gt5s_net_select_value(preference: BearerPreference) -> Result<&'static str> {
    match preference {
        // Keep legacy `Auto` behavior aligned with current GT5S UI default.
        BearerPreference::Auto | BearerPreference::LteAndNr5g => Ok("4G_AND_5G"),
        BearerPreference::Nr5gNsa => Ok("LTE_AND_5G"),
        BearerPreference::OnlyNr5g => Ok("Only_5G"),
        BearerPreference::OnlyLte => Ok("Only_LTE"),
        BearerPreference::OnlyGsm | BearerPreference::OnlyWcdma => {
            bail!("GSM/WCDMA-only bearer preferences are not supported on GT5S")
        }
    }
}

/// A single ubus JSON-RPC request.
#[derive(Serialize)]
struct UbusRequest {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: (String, String, String, Value),
}

/// A single ubus JSON-RPC response.
#[derive(Deserialize)]
struct UbusResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    /// Result can be [code] or [code, data]; we deserialize as raw Value.
    result: Option<Value>,
}

#[derive(Deserialize)]
struct LoginInfo {
    zte_web_sault: String,
    login_fail_num: i64,
    #[allow(dead_code)]
    login_fail_lock_lefttime: Option<i64>,
}

#[derive(Deserialize)]
struct LoginResult {
    result: i64,
    ubus_rpc_session: Option<String>,
    #[allow(dead_code)]
    timeout: Option<i64>,
}

/// WAN interface status returned by the GT5S.
#[derive(Deserialize)]
pub struct WwanIfaceStatus {
    pub connect_status: String,
    pub enable: i64,
    pub connect_mode: Option<i64>,
    pub roam_enable: Option<i64>,
    pub ipv4_address: Option<String>,
    pub ipv6_address: Option<String>,
    pub ipv4_dns_prefer: Option<String>,
    pub ipv4_dns_standby: Option<String>,
}

/// ZTE GT5S Client
///
/// Uses ubus JSON-RPC 2.0 over HTTPS to communicate with the router.
pub struct Gt5sClient {
    target: String,
    client: reqwest::Client,
    session: String,
    request_id: AtomicU64,
    /// AES-256-GCM key for encrypting sensitive fields (SMS body, APN password, etc.)
    aes_key: Option<[u8; 32]>,
}

impl Gt5sClient {
    pub fn new(url: &str) -> Result<Self> {
        #[allow(unused_mut)]
        let mut builder = reqwest::ClientBuilder::new().cookie_store(true);

        // Accept self-signed certificates when TLS is enabled
        #[cfg(any(feature = "tls-native", feature = "tls-rustls"))]
        {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build()?;
        let target = normalize_router_url(url)?;

        Ok(Gt5sClient {
            target,
            client,
            session: NULL_SESSION.to_string(),
            request_id: AtomicU64::new(1),
            aes_key: None,
        })
    }

    /// Get the current WAN connection status.
    pub async fn get_connection_status(&self) -> Result<WwanIfaceStatus> {
        let resp = self
            .send_command(&commands::GetWwanIfaceCommand::default())
            .await
            .context("Failed to get connection status")?;
        serde_json::from_value(resp).context("Failed to parse connection status")
    }

    /// Get login info including the salt and remaining login attempts.
    async fn get_login_info(&self) -> Result<LoginInfo> {
        let resp = self
            .send_command_with_session(
                NULL_SESSION,
                &commands::LoginInfoCommand::default(),
            )
            .await
            .context("Failed to get login info")?;
        serde_json::from_value(resp).context("Failed to parse login info")
    }

    async fn set_wwan_enable(&self, enable: bool) -> Result<()> {
        let resp = self
            .send_command(&commands::SetWwanIfaceCommand {
                enable: Some(if enable { 1 } else { 0 }),
                ..Default::default()
            })
            .await
            .context("Failed to set WWAN interface")?;

        if resp.get("enable").is_some() {
            Ok(())
        } else {
            bail!("Unexpected response: {}", resp)
        }
    }

    /// Set GT5S bearer preference including 5G-specific modes.
    pub async fn set_bearer_preference(&self, preference: BearerPreference) -> Result<()> {
        let net_select = gt5s_net_select_value(preference)?;
        self.send_command(&commands::SetNetSelectCommand { net_select })
            .await
            .context("Failed to set GT5S bearer preference")?;
        Ok(())
    }

    /// Read a UCI config section via ubus.
    async fn uci_get(&self, config: &str, section: Option<&str>) -> Result<Value> {
        self.send_command(&commands::UciGetCommand {
            config: config.to_string(),
            section: section.map(|s| s.to_string()),
        })
        .await
    }

    /// Establish AES-GCM encryption key with the router via RSA key exchange.
    async fn setup_encryption(&mut self) -> Result<()> {
        // 1. Get RSA public key from router
        let resp = self
            .send_command(&commands::GetCertificateCommand::default())
            .await
            .context("Failed to get RSA certificate")?;

        let pem = resp
            .get("result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing RSA certificate in response"))?;

        // Router returns PEM without line breaks; extract the base64 body and decode as DER
        let pem_body = pem
            .trim()
            .strip_prefix("-----BEGIN PUBLIC KEY-----")
            .and_then(|s| s.strip_suffix("-----END PUBLIC KEY-----"))
            .ok_or_else(|| anyhow!("Unexpected PEM format"))?
            .trim();
        let der = BASE64
            .decode(pem_body)
            .context("Failed to decode PEM base64")?;
        let pub_key = rsa::RsaPublicKey::from_public_key_der(&der)
            .context("Failed to parse RSA public key DER")?;

        // 2. Generate random 32-byte AES key and RSA-encrypt it (no rng across await)
        let (aes_key_bytes, encrypted_b64) = {
            use rand::RngCore;
            let mut aes_key_bytes = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut aes_key_bytes);
            let aes_key_hex: String =
                aes_key_bytes.iter().map(|b| format!("{:02x}", b)).collect();

            let mut rng = rand::thread_rng();
            let encrypted = pub_key
                .encrypt(&mut rng, Pkcs1v15Encrypt, aes_key_hex.as_bytes())
                .context("Failed to RSA-encrypt AES key")?;
            (aes_key_bytes, BASE64.encode(&encrypted))
        };

        // 3. Send encrypted key to router
        self.send_command(&commands::SetEncryptionKeyCommand {
            web_enstr: encrypted_b64,
        })
        .await
        .context("Failed to set encryption key")?;

        self.aes_key = Some(aes_key_bytes);
        Ok(())
    }

    /// AES-256-GCM encrypt a plaintext string.
    /// Returns base64(hex(iv_12) + hex(tag_16) + hex(ciphertext)) matching the JS format.
    fn aes_encrypt(&self, plaintext: &str) -> Result<String> {
        let key_bytes = self
            .aes_key
            .ok_or_else(|| anyhow!("Encryption not set up; call setup_encryption first"))?;

        // JS does hexToBytes(oo) where oo is the hex string; that gives back the original 32 bytes
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow!("Failed to create AES cipher: {}", e))?;

        let mut iv = [0u8; 12];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut iv);
        let nonce = Nonce::from_slice(&iv);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("AES-GCM encryption failed: {}", e))?;

        // AES-GCM appends the 16-byte tag to ciphertext
        let ct_len = ciphertext.len() - 16;
        let tag = &ciphertext[ct_len..];
        let ct = &ciphertext[..ct_len];

        let iv_hex: String = iv.iter().map(|b| format!("{:02x}", b)).collect();
        let tag_hex: String = tag.iter().map(|b| format!("{:02x}", b)).collect();
        let ct_hex: String = ct.iter().map(|b| format!("{:02x}", b)).collect();

        // Convert hex string to bytes for base64
        let combined_hex = format!("{}{}{}", iv_hex, tag_hex, ct_hex);
        let combined_bytes: Vec<u8> = (0..combined_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&combined_hex[i..i + 2], 16).unwrap())
            .collect();

        Ok(BASE64.encode(&combined_bytes))
    }


    /// Send a ubus command using the current session token.
    pub async fn send_command<T: UbusCommand>(&self, command: &T) -> Result<Value> {
        self.send_command_with_session(&self.session, command).await
    }

    /// Send a ubus command with a specific session token.
    async fn send_command_with_session<T: UbusCommand>(
        &self,
        session: &str,
        command: &T,
    ) -> Result<Value> {
        let params = serde_json::to_value(command)
            .context("Failed to serialize command")?;

        let id = self.request_id.fetch_add(1, Ordering::Relaxed);
        let request = UbusRequest {
            jsonrpc: "2.0",
            id,
            method: "call",
            params: (
                session.to_string(),
                command.module().to_string(),
                command.method().to_string(),
                params,
            ),
        };

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let url = format!("{}ubus/?t={}", self.target, ts);

        let response = self
            .client
            .post(&url)
            .header(REFERER, &self.target)
            .header(CONTENT_TYPE, "application/json")
            .json(&[&request])
            .send()
            .await
            .context("Failed to send ubus request")?
            .json::<Vec<UbusResponse>>()
            .await
            .context("Failed to parse ubus response")?;

        let resp = response
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Empty ubus response"))?;

        let result_arr = resp
            .result
            .ok_or_else(|| anyhow!("Missing result in ubus response"))?;

        let arr = result_arr
            .as_array()
            .ok_or_else(|| anyhow!("ubus result is not an array"))?;

        let code = arr
            .first()
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow!("Missing status code in ubus result"))?;

        if code != 0 {
            bail!("ubus error code {}", code);
        }

        let data = arr.get(1).cloned().unwrap_or(Value::Object(Default::default()));

        Ok(data)
    }
}

#[async_trait::async_trait]
impl RouterClient for Gt5sClient {
    async fn login(&mut self, password: &str) -> Result<()> {
        let info = self.get_login_info().await?;

        if info.login_fail_num <= 0 {
            bail!("Account is locked due to too many failed login attempts");
        }

        let hash = gt5s_password_hash(password, &info.zte_web_sault);

        let resp = self
            .send_command(&commands::LoginCommand { password: hash })
            .await
            .context("Failed to send login request")?;

        let login: LoginResult =
            serde_json::from_value(resp).context("Failed to parse login response")?;

        match login.result {
            0 => {
                self.session = login
                    .ubus_rpc_session
                    .ok_or_else(|| anyhow!("Login succeeded but no session token returned"))?;
                // Set up AES-GCM encryption key exchange after login
                if let Err(e) = self.setup_encryption().await {
                    eprintln!("Warning: encryption setup failed: {e:#}");
                }
                Ok(())
            }
            3 => bail!("Another user is already logged in"),
            _ => bail!("Login failed (result={})", login.result),
        }
    }

    async fn logout(&mut self) -> Result<()> {
        self.send_command(&commands::LogoutCommand::default())
            .await
            .context("Failed to logout")?;
        self.session = NULL_SESSION.to_string();
        Ok(())
    }

    async fn disconnect_network(&self) -> Result<()> {
        self.set_wwan_enable(false).await
    }

    async fn connect_network(&self) -> Result<()> {
        self.set_wwan_enable(true).await
    }

    async fn reboot(&self) -> Result<()> {
        self.send_command(&commands::RebootCommand::default())
            .await
            .context("Failed to reboot")?;
        Ok(())
    }

    async fn get_version(&self) -> Result<(String, String)> {
        let resp = self
            .uci_get("zwrt_common_info", Some("common_config"))
            .await
            .context("Failed to get version info")?;

        let values = resp
            .get("values")
            .ok_or_else(|| anyhow!("Missing values in UCI response"))?;

        let wa_inner_version = values
            .get("wa_inner_version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let hw_version = values
            .get("hardware_version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok((hw_version, wa_inner_version))
    }

    async fn set_connection_mode(
        &self,
        connection_mode: ConnectionMode,
        roam: bool,
    ) -> Result<()> {
        let mode = match connection_mode {
            ConnectionMode::Auto => 0,
            ConnectionMode::Manual => 1,
        };

        let resp = self
            .send_command(&commands::SetWwanIfaceCommand {
                connect_mode: Some(mode),
                roam_enable: Some(if roam { 1 } else { 0 }),
                ..Default::default()
            })
            .await
            .context("Failed to set connection mode")?;

        if resp.get("connect_mode").is_some() || resp.get("roam_enable").is_some() {
            Ok(())
        } else {
            bail!("Unexpected response: {}", resp)
        }
    }

    async fn set_network_bearer_preference(
        &self,
        bearer_preference: BearerPreference,
    ) -> Result<()> {
        self.set_bearer_preference(bearer_preference).await
    }

    async fn set_upnp(&self, enabled: bool) -> Result<()> {
        self.send_command(&commands::SetUpnpCommand {
            enable_upnp: if enabled { 1 } else { 0 },
        })
        .await
        .context("Failed to set UPnP")?;
        Ok(())
    }

    async fn set_dmz(&self, ip_address: Option<String>) -> Result<()> {
        self.send_command(&commands::SetDmzCommand {
            dmz_enable: if ip_address.is_some() { 1 } else { 0 },
            dmz_ip: ip_address,
        })
        .await
        .context("Failed to set DMZ")?;
        Ok(())
    }

    async fn select_lte_band(&self, lte_band: Option<HashSet<LteBand>>) -> Result<()> {
        let _ = lte_band;
        bail!("select_lte_band is not supported on GT5S")
    }

    async fn set_dns(&self, _manual: Option<[String; 2]>) -> Result<()> {
        bail!("set_dns is not directly supported on GT5S (DNS is managed by the connection profile)")
    }

    async fn get_status(&self) -> Result<Value> {
        let sim_info = self
            .send_command(&commands::GetSimInfoCommand::default())
            .await
            .unwrap_or(serde_json::json!({}));

        let wwan = self
            .send_command(&commands::GetWwanIfaceCommand::default())
            .await
            .unwrap_or(serde_json::json!({}));

        let device_info = self
            .uci_get("zwrt_zte_mdm", Some("device_info"))
            .await
            .unwrap_or(serde_json::json!({}));

        let common_config = self
            .uci_get("zwrt_common_info", Some("common_config"))
            .await
            .unwrap_or(serde_json::json!({}));

        let router_status = self
            .send_command(&commands::GetRouterStatusCommand::default())
            .await
            .unwrap_or(serde_json::json!({}));

        let device_values = device_info
            .get("values")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let common_values = common_config
            .get("values")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        Ok(serde_json::json!({
            "sim_info": sim_info,
            "wwan": wwan,
            "device_info": device_values,
            "common_config": common_values,
            "router_status": router_status,
        }))
    }

    async fn get_apn_mode(&self) -> Result<bool> {
        let resp = self
            .send_command(&commands::GetApnModeCommand::default())
            .await
            .context("Failed to get APN mode")?;
        let mode = resp
            .get("apn_mode")
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .unwrap_or(0);
        Ok(mode == 1)
    }

    async fn set_apn_mode(&self, manual: bool) -> Result<()> {
        self.send_command(&commands::SetApnModeCommand {
            apn_mode: if manual { 1 } else { 0 },
        })
        .await
        .context("Failed to set APN mode")?;
        Ok(())
    }

    async fn get_apn_profiles(&self) -> Result<Vec<ApnProfile>> {
        let resp = self
            .send_command(&commands::GetManuApnListCommand::default())
            .await
            .context("Failed to get manual APN list")?;

        let profiles = resp
            .get("apnListArray")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut result = Vec::new();
        for p in profiles {
            let auth_str = p
                .get("pppAuthMode")
                .and_then(|v| v.as_str())
                .unwrap_or("0");
            let auth_mode = match auth_str {
                "0" | "NONE" => ApnAuthMode::None,
                "1" | "PAP" => ApnAuthMode::Pap,
                "2" | "CHAP" => ApnAuthMode::Chap,
                "3" | "PAP_CHAP" => ApnAuthMode::PapChap,
                _ => ApnAuthMode::None,
            };
            let pdp_str = p
                .get("pdpType")
                .and_then(|v| v.as_str())
                .unwrap_or("IPv4v6");
            let pdp_type = match pdp_str {
                "IPv4" | "IP" => PdpType::IPv4,
                "IPv6" => PdpType::IPv6,
                _ => PdpType::IPv4v6,
            };

            result.push(ApnProfile {
                profile_id: p
                    .get("profileId")
                    .and_then(|v| v.as_str().or_else(|| v.as_i64().map(|_| "")).map(|s| s.to_string()))
                    .or_else(|| p.get("profileId").and_then(|v| v.as_i64()).map(|n| n.to_string())),
                profile_name: p
                    .get("profilename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                apn: p
                    .get("wanapn")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                pdp_type,
                auth_mode,
                username: p
                    .get("username")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                password: String::new(), // password is encrypted, don't expose
            });
        }
        Ok(result)
    }

    async fn set_apn_profile(&self, profile: &ApnProfile) -> Result<()> {
        let profile_id = profile
            .profile_id
            .as_deref()
            .ok_or_else(|| anyhow!("profile_id is required for modifying an APN profile"))?;

        let auth_mode = match profile.auth_mode {
            ApnAuthMode::None => "0",
            ApnAuthMode::Pap => "1",
            ApnAuthMode::Chap => "2",
            ApnAuthMode::PapChap => "3",
        };
        let pdp_type = match profile.pdp_type {
            PdpType::IPv4 => "IPv4",
            PdpType::IPv6 => "IPv6",
            PdpType::IPv4v6 => "IPv4v6",
        };

        let encrypted_password = if profile.password.is_empty() {
            self.aes_encrypt("")?  
        } else {
            self.aes_encrypt(&profile.password)?
        };

        self.send_command(&commands::ModifyManuApnCommand {
            profile_name: profile.profile_name.clone(),
            pdp_type,
            apn: profile.apn.clone(),
            auth_mode,
            username: profile.username.clone(),
            password: encrypted_password,
            profile_id: profile_id.to_string(),
        })
        .await
        .context("Failed to modify APN profile")?;
        Ok(())
    }

    async fn enable_apn_profile(&self, profile_id: &str) -> Result<()> {
        self.send_command(&commands::EnableManuApnCommand {
            profile_id: profile_id.to_string(),
        })
        .await
        .context("Failed to enable APN profile")?;
        Ok(())
    }

    async fn get_dhcp_settings(&self) -> Result<DhcpSettings> {
        let lan = self
            .uci_get("network", Some("lan"))
            .await
            .context("Failed to get LAN settings")?;
        let dhcp = self
            .uci_get("dhcp", Some("lan"))
            .await
            .context("Failed to get DHCP settings")?;
        let router_dhcp = self
            .uci_get("zwrt_router", Some("dhcp"))
            .await
            .context("Failed to get router DHCP settings")?;

        let lan_vals = lan.get("values").unwrap_or(&Value::Null);
        let dhcp_vals = dhcp.get("values").unwrap_or(&Value::Null);
        let router_vals = router_dhcp.get("values").unwrap_or(&Value::Null);

        let ip_address = lan_vals
            .get("ipaddr")
            .and_then(|v| v.as_str())
            .unwrap_or("192.168.0.1")
            .to_string();
        let subnet_mask = lan_vals
            .get("netmask")
            .and_then(|v| v.as_str())
            .unwrap_or("255.255.255.0")
            .to_string();
        let ignore = dhcp_vals
            .get("ignore")
            .and_then(|v| v.as_str().or_else(|| v.as_i64().map(|_| "")))
            .unwrap_or("0");
        let dhcp_enabled = ignore != "1";
        let lease_time = router_vals
            .get("leasetime")
            .and_then(|v| v.as_str())
            .and_then(|s| s.trim_end_matches('h').parse::<u32>().ok())
            .unwrap_or(24);

        Ok(DhcpSettings {
            ip_address,
            subnet_mask,
            dhcp_enabled,
            lease_time,
        })
    }

    async fn set_dhcp_settings(&self, settings: &DhcpSettings) -> Result<()> {
        self.send_command(&commands::SetLanParaCommand {
            ipaddr: settings.ip_address.clone(),
            netmask: settings.subnet_mask.clone(),
            ignore: if settings.dhcp_enabled { "0" } else { "1" },
            leasetime: format!("{}h", settings.lease_time),
        })
        .await
        .context("Failed to set DHCP settings")?;
        Ok(())
    }

    async fn get_mtu_settings(&self) -> Result<MtuSettings> {
        let resp = self
            .uci_get("zwrt_router", Some("network"))
            .await
            .context("Failed to get MTU settings")?;

        let vals = resp.get("values").unwrap_or(&Value::Null);
        let mtu = vals
            .get("mtu")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1500);
        let mss = vals
            .get("mss")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1460);

        Ok(MtuSettings { mtu, mss })
    }

    async fn set_mtu_settings(&self, settings: &MtuSettings) -> Result<()> {
        self.send_command(&commands::SetWanMtuCommand {
            mtu: settings.mtu.to_string(),
            mss: settings.mss.to_string(),
        })
        .await
        .context("Failed to set MTU/MSS")?;
        Ok(())
    }

    async fn get_sms_settings(&self) -> Result<SmsSettings> {
        let resp = self
            .send_command(&commands::GetSmsParameterCommand::default())
            .await
            .context("Failed to get SMS settings")?;

        let validity = resp
            .get("tp_validity_period")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let center_number = resp
            .get("sca")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let delivery_report = resp
            .get("status_report_on")
            .and_then(|v| v.as_str().or_else(|| v.as_i64().map(|_| "")))
            .unwrap_or("0");

        Ok(SmsSettings {
            validity,
            center_number,
            delivery_report: delivery_report == "1",
        })
    }

    async fn get_network_info(&self) -> Result<Value> {
        self.send_command(&commands::GetNetInfoCommand::default())
            .await
            .context("Failed to get network info")
    }

    async fn get_sim_info(&self) -> Result<Value> {
        self.send_command(&commands::GetSimInfoCommand::default())
            .await
            .context("Failed to get SIM info")
    }

    async fn get_device_info(&self) -> Result<Value> {
        let device_info = self
            .uci_get("zwrt_zte_mdm", Some("device_info"))
            .await
            .unwrap_or(serde_json::json!({}));
        let common_config = self
            .uci_get("zwrt_common_info", Some("common_config"))
            .await
            .unwrap_or(serde_json::json!({}));

        let device_values = device_info
            .get("values")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let common_values = common_config
            .get("values")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        Ok(serde_json::json!({
            "device_info": device_values,
            "common_config": common_values,
        }))
    }

    async fn get_connected_devices(&self) -> Result<Value> {
        let user_count = self
            .send_command(&commands::GetUserListNumCommand::default())
            .await
            .context("Failed to get user list count")?;

        let total = user_count
            .get("access_total_num")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        if total == 0 {
            return Ok(serde_json::json!({"devices": []}));
        }

        let lan_list = self
            .send_command(&commands::GetLanAccessListCommand::default())
            .await
            .context("Failed to get LAN access list")?;

        Ok(lan_list)
    }
}

/// Compute the GT5S password hash: `SHA256(SHA256(password).UPPER + salt).UPPER`
pub fn gt5s_password_hash(password: &str, salt: &str) -> String {
    let hash1 = sha256::digest(password.as_bytes()).to_uppercase();
    let concat = format!("{}{}", hash1, salt);
    sha256::digest(concat.as_bytes()).to_uppercase()
}
