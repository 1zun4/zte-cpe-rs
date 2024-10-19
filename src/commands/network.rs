use super::GoformCommand;
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct DisconnectNetworkCommand;

impl GoformCommand for DisconnectNetworkCommand {
    fn goform_id(&self) -> &'static str {
        "DISCONNECT_NETWORK"
    }

    fn authenticated(&self) -> bool {
        true
    }
}

#[derive(Serialize, Default)]
pub struct ConnectNetworkCommand;

impl GoformCommand for ConnectNetworkCommand {
    fn goform_id(&self) -> &'static str {
        "CONNECT_NETWORK"
    }

    fn authenticated(&self) -> bool {
        true
    }
}

#[derive(Serialize)]
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

// goformId=SET_CONNECTION_MODE&isTest=false&ConnectionMode=manual_dial&roam_setting_option=on&AD=083a022656ae93cf730854fa5d292fff
// goformId=SET_CONNECTION_MODE&isTest=false&ConnectionMode=auto_dial&roam_setting_option=on&AD=335ac27b5ed34b37ec6a5683f9665dfb
#[derive(Serialize, Default)]
pub struct ConnectionModeCommand {
    #[serde(rename = "ConnectionMode")]
    pub connection_mode: ConnectionMode,
    #[serde(rename = "roam_setting_option")]
    #[serde(serialize_with = "crate::util::bool_to_str")]
    pub roam_setting_option: bool,
}

impl GoformCommand for ConnectionModeCommand {
    fn goform_id(&self) -> &'static str {
        "SET_CONNECTION_MODE"
    }

    fn authenticated(&self) -> bool {
        true
    }
}

// isTest=false&goformId=SET_BEARER_PREFERENCE&BearerPreference=NETWORK_auto&AD=0a7f52c55cb7c4d3cc99690780702e3a
// isTest=false&goformId=SET_BEARER_PREFERENCE&BearerPreference=Only_LTE&AD=ccf4ff80adf777e9c6ee1ab50216d9d0
// isTest=false&goformId=SET_BEARER_PREFERENCE&BearerPreference=Only_GSM&AD=945b088500a4bc851a07647ff967d6c9
// isTest=false&goformId=SET_BEARER_PREFERENCE&BearerPreference=Only_WCDMA&AD=b47f01a5a2e331713cf8497391211246
#[derive(Serialize)]
pub enum BearerPreference {
    #[serde(rename = "NETWORK_auto")]
    Auto,
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

#[derive(Serialize, Default)]
pub struct BearerPreferenceCommand {
    #[serde(rename = "BearerPreference")]
    pub bearer_preference: BearerPreference,
}

impl GoformCommand for BearerPreferenceCommand {
    fn goform_id(&self) -> &'static str {
        "SET_BEARER_PREFERENCE"
    }

    fn authenticated(&self) -> bool {
        true
    }
}

#[derive(Serialize, Default)]
pub struct LockLteBandCommand {
    #[serde(rename = "lte_band_lock")]
    pub lte_band_lock: String,
}

impl GoformCommand for LockLteBandCommand {
    fn goform_id(&self) -> &'static str {
        "SET_LTE_BAND_LOCK"
    }

    fn authenticated(&self) -> bool {
        true
    }
}
