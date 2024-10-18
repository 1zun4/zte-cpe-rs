use serde::Serialize;
use super::GoformCommand;


#[derive(Serialize, Default)]
pub struct DisconnectNetworkCommand { }

impl GoformCommand for DisconnectNetworkCommand {
    fn goform_id(&self) -> &'static str {
        "DISCONNECT_NETWORK"
    }
    
    fn authenticated(&self) -> bool {
        true
    }
}

#[derive(Serialize, Default)]
pub struct ConnectNetworkCommand { }

impl GoformCommand for ConnectNetworkCommand {
    fn goform_id(&self) -> &'static str {
        "CONNECT_NETWORK"
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