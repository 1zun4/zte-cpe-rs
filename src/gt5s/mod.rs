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

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use reqwest::header::{CONTENT_TYPE, REFERER};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bands::LteBand;
use crate::{BearerPreference, ConnectionMode, RouterClient};

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
    result: Option<(i64, Value)>,
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
}

impl Gt5sClient {
    pub fn new(ip: &str) -> Result<Self> {
        #[allow(unused_mut)]
        let mut builder = reqwest::ClientBuilder::new().cookie_store(true);

        // Accept self-signed certificates when TLS is enabled
        #[cfg(any(feature = "tls-native", feature = "tls-rustls"))]
        {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build()?;

        let target = format!("https://{}/", ip);

        Ok(Gt5sClient {
            target,
            client,
            session: NULL_SESSION.to_string(),
            request_id: AtomicU64::new(1),
        })
    }

    /// Get the current WAN connection status.
    pub async fn get_connection_status(&self) -> Result<WwanIfaceStatus> {
        let resp = self
            .rpc_call(
                "zwrt_data",
                "get_wwaniface",
                serde_json::json!({"source_module": "web", "cid": 1}),
            )
            .await
            .context("Failed to get connection status")?;

        serde_json::from_value(resp).context("Failed to parse connection status")
    }

    /// Get login info including the salt and remaining login attempts.
    async fn get_login_info(&self) -> Result<LoginInfo> {
        let resp = self
            .rpc_call_with_session(
                NULL_SESSION,
                "zwrt_web",
                "web_login_info",
                serde_json::json!({}),
            )
            .await
            .context("Failed to get login info")?;

        serde_json::from_value(resp).context("Failed to parse login info")
    }

    async fn set_wwan_enable(&self, enable: bool) -> Result<()> {
        let resp = self
            .rpc_call(
                "zwrt_data",
                "set_wwaniface",
                serde_json::json!({
                    "source_module": "web",
                    "cid": 1,
                    "enable": if enable { 1 } else { 0 },
                }),
            )
            .await
            .context("Failed to set WWAN interface")?;

        if resp.get("enable").is_some() {
            Ok(())
        } else {
            bail!("Unexpected response: {}", resp)
        }
    }

    /// Set GT5S bearer preference including 5G-specific modes.
    ///
    /// Uses ubus only:
    /// `zte_nwinfo_api.nwinfo_set_netselect({ net_select: <mode> })`
    pub async fn set_bearer_preference(&self, preference: BearerPreference) -> Result<()> {
        let net_select = gt5s_net_select_value(preference)?;
        let resp = self
            .rpc_call(
                "zte_nwinfo_api",
                "nwinfo_set_netselect",
                serde_json::json!({"net_select": net_select}),
            )
            .await
            .context("Failed to set GT5S bearer preference")?;

        // ubus success is represented by top-level result code 0; payload may be empty.
        let _ = resp;
        Ok(())
    }

    /// Read a UCI config section via ubus.
    async fn uci_get(&self, config: &str, section: Option<&str>) -> Result<Value> {
        let mut params = serde_json::json!({"config": config});
        if let Some(s) = section {
            params["section"] = Value::String(s.to_string());
        }
        self.rpc_call("uci", "get", params).await
    }

    /// Make an RPC call using the current session token.
    async fn rpc_call(&self, module: &str, method: &str, params: Value) -> Result<Value> {
        self.rpc_call_with_session(&self.session, module, method, params)
            .await
    }

    /// Make an RPC call with a specific session token.
    async fn rpc_call_with_session(
        &self,
        session: &str,
        module: &str,
        method: &str,
        params: Value,
    ) -> Result<Value> {
        let id = self.request_id.fetch_add(1, Ordering::Relaxed);
        let request = UbusRequest {
            jsonrpc: "2.0",
            id,
            method: "call",
            params: (
                session.to_string(),
                module.to_string(),
                method.to_string(),
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

        let (code, data) = resp
            .result
            .ok_or_else(|| anyhow!("Missing result in ubus response"))?;

        if code != 0 {
            bail!("ubus error code {}: {}", code, data);
        }

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
            .rpc_call(
                "zwrt_web",
                "web_login",
                serde_json::json!({"password": hash}),
            )
            .await
            .context("Failed to send login request")?;

        let login: LoginResult =
            serde_json::from_value(resp).context("Failed to parse login response")?;

        match login.result {
            0 => {
                self.session = login
                    .ubus_rpc_session
                    .ok_or_else(|| anyhow!("Login succeeded but no session token returned"))?;
                Ok(())
            }
            3 => bail!("Another user is already logged in"),
            _ => bail!("Login failed (result={})", login.result),
        }
    }

    async fn logout(&mut self) -> Result<()> {
        self.rpc_call("zwrt_web", "web_logout", serde_json::json!({}))
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
        self.rpc_call(
            "zwrt_mc.device.manager",
            "device_reboot",
            serde_json::json!({"moduleName": "web"}),
        )
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
            .rpc_call(
                "zwrt_data",
                "set_wwaniface",
                serde_json::json!({
                    "source_module": "web",
                    "cid": 1,
                    "connect_mode": mode,
                    "roam_enable": if roam { 1 } else { 0 },
                }),
            )
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
        self.rpc_call(
            "zwrt_router.api",
            "router_set_upnp_switch",
            serde_json::json!({"enable_upnp": if enabled { 1 } else { 0 }}),
        )
        .await
        .context("Failed to set UPnP")?;
        Ok(())
    }

    async fn set_dmz(&self, ip_address: Option<String>) -> Result<()> {
        let mut params = serde_json::json!({
            "dmz_enable": if ip_address.is_some() { 1 } else { 0 },
        });
        if let Some(ip) = ip_address {
            params["dmz_ip"] = Value::String(ip);
        }

        self.rpc_call("zwrt_router.api", "router_set_dmz", params)
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
        // Gather data from multiple ubus sources, similar to the JS ut() function
        let sim_info = self
            .rpc_call("zwrt_zte_mdm.api", "get_sim_info", serde_json::json!({}))
            .await
            .unwrap_or(serde_json::json!({}));

        let wwan = self
            .rpc_call(
                "zwrt_data",
                "get_wwaniface",
                serde_json::json!({"source_module": "web", "cid": 1}),
            )
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
            .rpc_call(
                "zwrt_router.api",
                "router_get_status",
                serde_json::json!({}),
            )
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
}

/// Compute the GT5S password hash: `SHA256(SHA256(password).UPPER + salt).UPPER`
pub fn gt5s_password_hash(password: &str, salt: &str) -> String {
    let hash1 = sha256::digest(password.as_bytes()).to_uppercase();
    let concat = format!("{}{}", hash1, salt);
    sha256::digest(concat.as_bytes()).to_uppercase()
}
