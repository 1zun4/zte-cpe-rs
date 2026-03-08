use std::collections::HashSet;

use anyhow::Result;
use bands::LteBand;
use serde_json::Value;

pub mod bands;
#[cfg(feature = "mf289f")]
pub(crate) mod util;

#[cfg(feature = "mf289f")]
pub mod mf289f;

#[derive(serde::Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMode {
    #[serde(rename = "auto_dial")]
    Auto,
    #[serde(rename = "manual_dial")]
    Manual,
}

impl Default for ConnectionMode {
    fn default() -> Self {
        ConnectionMode::Auto
    }
}

#[derive(serde::Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BearerPreference {
    #[serde(rename = "NETWORK_auto")]
    Auto,
    #[serde(rename = "4G_AND_5G")]
    LteAndNr5g,
    #[serde(rename = "LTE_AND_5G")]
    Nr5gNsa,
    #[serde(rename = "Only_5G")]
    OnlyNr5g,
    #[serde(rename = "Only_LTE")]
    OnlyLte,
    #[serde(rename = "Only_GSM")]
    OnlyGsm,
    #[serde(rename = "Only_WCDMA")]
    OnlyWcdma,
}

impl Default for BearerPreference {
    fn default() -> Self {
        BearerPreference::Auto
    }
}

#[async_trait::async_trait]
pub trait RouterClient {
    async fn login(&mut self, password: &str) -> Result<()>;

    async fn logout(&mut self) -> Result<()>;

    async fn disconnect_network(&self) -> Result<()>;

    async fn connect_network(&self) -> Result<()>;

    async fn reboot(&self) -> Result<()>;

    async fn get_version(&self) -> Result<(String, String)>;

    async fn set_connection_mode(
        &self,
        connection_mode: ConnectionMode,
        roam: bool,
    ) -> Result<()>;

    async fn set_network_bearer_preference(
        &self,
        bearer_preference: BearerPreference,
    ) -> Result<()>;

    async fn set_upnp(&self, enabled: bool) -> Result<()>;

    async fn set_dmz(&self, ip_address: Option<String>) -> Result<()>;

    async fn select_lte_band(&self, lte_band: Option<HashSet<LteBand>>) -> Result<()>;

    async fn set_dns(&self, manual: Option<[String; 2]>) -> Result<()>;

    async fn get_status(&self) -> Result<Value>;
}
