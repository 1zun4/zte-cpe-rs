use std::collections::HashSet;

use anyhow::{anyhow, bail, Context, Result};
use bands::{select_lte_band, LteBand};
use commands::{AdCommand, BearerPreference, BearerPreferenceCommand, ConnectNetworkCommand, ConnectionMode, ConnectionModeCommand, DisconnectNetworkCommand, DnsModeCommand, GoformCommand, LockLteBandCommand, LoginCommand, LogoutCommand, RebootCommand, UpnpCommand};
use log::{debug, info};
use reqwest::header::{CONTENT_TYPE, REFERER};
use serde::Serialize;
use serde_json::Value;

#[cfg(test)]
mod tests;

pub mod commands;
pub mod bands;

pub(crate) mod util;

/// ZTE Client
/// 
/// Tested for ZTE MF289F Gigacube
/// 
pub struct ZteClient {
    referer: String,
    client: reqwest::Client
}

impl ZteClient {

    pub fn new(ip: &str) -> Result<Self> {
        let client = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .build()?;
        
        #[cfg(any(feature = "ssl-native", feature = "ssl-rustls"))]
        let referer = format!("https://{}/", ip);
        #[cfg(not(any(feature = "ssl-native", feature = "ssl-rustls")))]
        let referer = format!("http://{}/", ip);

        Ok(ZteClient {
            referer,
            client
        })
    }

    pub async fn login(&mut self, password: String) -> Result<()> {
        let ld = self.get_ld().await
            .context("Failed to fetch LD")?;

        // Hash password to SHA256 and then to uppercase
        let hash_password = sha256::digest(password.as_bytes()).to_uppercase();

        // Hash the password and LD together and then to uppercase
        let zte_pass = sha256::digest(&(hash_password + &ld)).to_uppercase();

        let code = self.send_command(LoginCommand {
            password: zte_pass.clone(),
            ..Default::default()
        }).await.context("Failed to login")?;

        return match code.parse::<i32>().context("Failed to parse login response")? {
            0 => Ok(()),
            3 => bail!("Invalid password"),
            _ => bail!(format!("Unknown error code: {}", code))
        };
    }

    pub async fn logout(&mut self) -> Result<()> {
        self.send_command(LogoutCommand {}).await
            .context("Failed to logout")?;
        Ok(())
    }

    // http://giga.cube/goform/goform_get_cmd_process?isTest=false&cmd=wa_inner_version
    // {"wa_inner_version":"BD_VDFDEMF289FV1.0.0B08 [Jun 18 2022 05:39:38]"}
    pub async fn get_version(&self) -> Result<(String, String)> {
        let response = self.get_command("cr_version,wa_inner_version").await?;
        info!("Response: {}", response);
        
        // cr_version, wa_inner_version
        let cr_version = response.get("cr_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Missing cr_version in response"))?;
        let wa_inner_version = response.get("wa_inner_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Missing wa_inner_version in response"))?;

        Ok((cr_version, wa_inner_version))
    }

    // REBOOT_DEVICE -> {result: "success"}
    pub async fn reboot(&self) -> Result<()> {
        match self.send_command(RebootCommand {}).await?.as_str() {
            "success" => Ok(()),
            _ => bail!("Failed to reboot"),
        }
    }

    pub async fn disconnect_network(&self) -> Result<()> {
        match self.send_command(DisconnectNetworkCommand {}).await?.as_str() {
            "success" => Ok(()),
            _ => bail!("Failed to disconnect network"),
        }
    }

    pub async fn connect_network(&self) -> Result<()> {
        match self.send_command(ConnectNetworkCommand {}).await?.as_str() {
            "success" => Ok(()),
            _ => bail!("Failed to connect network"),
        }
    }

    pub async fn set_connection_mode(&self, connection_mode: ConnectionMode, roam: bool) -> Result<()> {
        let command = ConnectionModeCommand {
            connection_mode,
            roam_setting_option: roam,
        };

        match self.send_command(command).await?.as_str() {
            "success" => Ok(()),
            _ => bail!("Failed to set connection mode"),
        }
    }

    pub async fn set_network_bearer_preference(&self, bearer_preference: BearerPreference) -> Result<()> {
        let command = BearerPreferenceCommand {
            bearer_preference,
        };

        match self.send_command(command).await?.as_str() {
            "success" => Ok(()),
            _ => bail!("Failed to set network bearer preference"),
        }
    }

    pub async fn set_upnp(&self, enabled: bool) -> Result<()> {
        let command = UpnpCommand {
            upnp_setting_option: enabled,
        };

        match self.send_command(command).await?.as_str() {
            "success" => Ok(()),
            _ => bail!("Failed to set UPnP"),
        }
    }

    pub async fn set_dmz(&self, ip_address: Option<String>) -> Result<()> {
        let command = commands::DmzCommand {
            dmz_enabled: ip_address.is_some(),
            dmz_ip_address: ip_address,
        };

        match self.send_command(command).await?.as_str() {
            "success" => Ok(()),
            _ => bail!("Failed to set DMZ"),
        }
    }
    
    // Lock LTE band
    // Inspired by https://miononno.it/files/zte-v3.0b.min.txt
    pub async fn select_lte_band(&self, lte_band: Option<HashSet<LteBand>>) -> Result<()> {
        // SET_LTE_BAND_LOCK
        // lte_band_lock:n

        let lte_band_lock = select_lte_band(lte_band).await;
        debug!("Selected LTE band: {}", lte_band_lock);

        let command = LockLteBandCommand {
            lte_band_lock,
        };

        match self.send_command(command).await?.as_str() {
            "success" => Ok(()),
            v => {
                println!("Response: {}", v);
                bail!("Failed to select LTE band")
            },
        }
    }

    // Set DNS to manual or automatic
    // Inspired by https://miononno.it/files/zte-v3.0b.min.txt
    pub async fn set_dns(&self, manual: Option<[String; 2]>) -> Result<()> {
        // ROUTER_DNS_SETTING
        // dns_mode:dns_mode,prefer_dns_manual:a[0],standby_dns_manual:a[1]

        let dns_mode = if manual.is_some() { "manual" } else { "auto" };
        let prefer_dns_manual = manual.as_ref()
            .map(|a| a[0].clone()).unwrap_or("".to_string());
        let standby_dns_manual = manual.as_ref()
            .map(|a| a[1].clone()).unwrap_or("".to_string());

        let command = DnsModeCommand {
            dns_mode: dns_mode.to_string(),
            prefer_dns_manual,
            standby_dns_manual,
        };

        match self.send_command(command).await?.as_str() {
            "success" => Ok(()),
            _ => bail!("Failed to set DNS"),
        }
    }


    // gets a list of pre-defined useful information
    pub async fn get_status(&self) -> Result<Value> {
        const COMMAND_SET: &str = "dns_mode,prefer_dns_manual,standby_dns_manual,network_type,mcc,mnc,rssi,rsrq,lte_rsrp,wan_lte_ca,lte_ca_pcell_band,lte_ca_pcell_bandwidth,lte_ca_scell_band,lte_ca_scell_bandwidth,lte_ca_pcell_arfcn,lte_ca_scell_arfcn,Z_SINR,Z_CELL_ID,Z_eNB_id,Z_rsrq,lte_ca_scell_info,wan_ipaddr,ipv6_wan_ipaddr,static_wan_ipaddr,opms_wan_mode,opms_wan_auto_mode,ppp_status,loginfo";
        let response = self.get_command(COMMAND_SET).await?;

        Ok(response)
    }

    // isTest=false&cmd=LD
    async fn get_ld(&self) -> Result<String> {
        self.get_command("LD").await
            .context("Failed to fetch LD")
            .map(|response| response.get("LD")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow!("Missing LD in response")))
            .and_then(|r| r)
    }

    // isTest=false&cmd=RD
    async fn get_rd(&self) -> Result<String> {
        self.get_command("RD").await
            .context("Failed to fetch RD")
            .map(|response| response.get("RD")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow!("Missing RD in response")))
            .and_then(|r| r)
    }

    async fn get_ad(&self) -> Result<String> {
        let (cr_version, wa_inner_version) = self.get_version().await
            .context("Failed to fetch version")?;

        let a = format!("{:x}", md5::compute(format!("{}{}", wa_inner_version, cr_version)));
        let u = self.get_rd().await
            .context("Failed to fetch RD")?;

        Ok(format!("{:x}", md5::compute(&(a + &u))))
    }

    pub async fn send_command<T>(&self, command: T) -> Result<String>
    where
        T: GoformCommand + Serialize + Default,
    {
        let goform_id = command.goform_id();
        
        let ad  = if command.authenticated() {
            // Fetch AD and wrap the command with AD
            let ad = self.get_ad().await
                .context("Failed to fetch AD")?;
            
            Some(ad)
        } else {
            None
        };

        let wrapped_command = AdCommand {
            ad,
            command,
            ..Default::default()
        };

        let form_data = serde_urlencoded::to_string(&wrapped_command)
            .context(format!("Failed to serialize command: {}", goform_id))?;

        let url = format!("{}goform/goform_set_cmd_process", self.referer);
        let request = self.client.post(&url)
            .header(REFERER, &self.referer)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded; charset=UTF-8")
            .body(form_data);

        let response = request.send()
            .await
            .context(format!("Failed to send {} command", goform_id))?
            .json::<Value>()
            .await
            .context(format!("Failed to parse JSON for {} command", goform_id))?;

        let result = response.get("result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing result in response"))?;

        Ok(result.to_string())
    }
    
    // Get data from command or multiple commands
    // TODO: Use a struct containing ALL commands to fetch and their data type
    pub async fn get_command(&self, cmd: &str) -> Result<Value> {
        let multi_data = cmd.contains(",");
        let url = format!("{}goform/goform_get_cmd_process?isTest=false&cmd={}{}", self.referer, cmd, if multi_data { "&multi_data=1" } else { "" });

        let response = self.client.get(&url)
            .header(REFERER, &self.referer)
            .send()
            .await
            .context(format!("Failed to fetch command {}", cmd))?
            .json::<Value>()
            .await
            .context(format!("Failed to parse JSON for command {}", cmd))?;
        Ok(response)
    }

}
