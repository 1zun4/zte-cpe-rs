use serde::Serialize;

#[derive(Serialize, Default)]
pub struct LoginCommand {
    #[serde(rename = "isTest")]
    pub is_test: bool,
    pub password: String,
}

impl GoformCommand for LoginCommand {
    fn goform_id(&self) -> &'static str {
        "LOGIN"
    }
}

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

#[derive(Serialize, Default)]
pub struct DnsModeCommand {
    #[serde(rename = "dns_mode")]
    pub dns_mode: String,
    #[serde(rename = "prefer_dns_manual")]
    pub prefer_dns_manual: String,
    #[serde(rename = "standby_dns_manual")]
    pub standby_dns_manual: String,
}

impl GoformCommand for DnsModeCommand {
    fn goform_id(&self) -> &'static str {
        "ROUTER_DNS_SETTING"
    }
    
    fn authenticated(&self) -> bool {
        true
    }
}

#[derive(Serialize, Default)]
pub struct RebootCommand { }

impl GoformCommand for RebootCommand {
    fn goform_id(&self) -> &'static str {
        "REBOOT_DEVICE"
    }
    
    fn authenticated(&self) -> bool {
        true
    }
}

#[derive(Serialize, Default)]
pub struct LogoutCommand { }

impl GoformCommand for LogoutCommand {
    fn goform_id(&self) -> &'static str {
        "LOGOUT"
    }
    
    fn authenticated(&self) -> bool {
        true
    }
}

#[derive(Serialize)]
pub struct AdCommand<T> {
    #[serde(rename = "isTest")]
    pub is_test: bool,
    #[serde(rename = "goformId")]
    pub goform_id: &'static str,
    #[serde(rename = "AD")]
    pub ad: Option<String>,
    #[serde(flatten)]
    pub command: T,
}

impl<T> Default for AdCommand<T>
where
    T: Default + GoformCommand,
{
    fn default() -> Self {
        AdCommand {
            is_test: false,
            goform_id: T::default().goform_id(),
            ad: None,
            command: T::default(),
        }
    }
}

pub trait GoformCommand {
    fn goform_id(&self) -> &'static str;
    fn authenticated(&self) -> bool {
        false
    }
}