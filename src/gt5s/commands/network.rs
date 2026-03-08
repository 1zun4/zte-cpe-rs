use super::UbusCommand;
use serde::Serialize;

/// Get WAN interface status.
#[derive(Serialize)]
pub struct GetWwanIfaceCommand {
    pub source_module: &'static str,
    pub cid: i32,
}

impl Default for GetWwanIfaceCommand {
    fn default() -> Self {
        Self {
            source_module: "web",
            cid: 1,
        }
    }
}

impl UbusCommand for GetWwanIfaceCommand {
    fn module(&self) -> &'static str {
        "zwrt_data"
    }
    fn method(&self) -> &'static str {
        "get_wwaniface"
    }
}

/// Set WAN interface parameters (enable/disable, connection mode, roaming).
#[derive(Serialize)]
pub struct SetWwanIfaceCommand {
    pub source_module: &'static str,
    pub cid: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_mode: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roam_enable: Option<i32>,
}

impl Default for SetWwanIfaceCommand {
    fn default() -> Self {
        Self {
            source_module: "web",
            cid: 1,
            enable: None,
            connect_mode: None,
            roam_enable: None,
        }
    }
}

impl UbusCommand for SetWwanIfaceCommand {
    fn module(&self) -> &'static str {
        "zwrt_data"
    }
    fn method(&self) -> &'static str {
        "set_wwaniface"
    }
}

/// Set network bearer preference (4G/5G mode selection).
#[derive(Serialize)]
pub struct SetNetSelectCommand {
    pub net_select: &'static str,
}

impl UbusCommand for SetNetSelectCommand {
    fn module(&self) -> &'static str {
        "zte_nwinfo_api"
    }
    fn method(&self) -> &'static str {
        "nwinfo_set_netselect"
    }
}

/// Get network/signal information.
#[derive(Serialize, Default)]
pub struct GetNetInfoCommand {}

impl UbusCommand for GetNetInfoCommand {
    fn module(&self) -> &'static str {
        "zte_nwinfo_api"
    }
    fn method(&self) -> &'static str {
        "nwinfo_get_netinfo"
    }
}

/// Get SIM card information.
#[derive(Serialize, Default)]
pub struct GetSimInfoCommand {}

impl UbusCommand for GetSimInfoCommand {
    fn module(&self) -> &'static str {
        "zwrt_zte_mdm.api"
    }
    fn method(&self) -> &'static str {
        "get_sim_info"
    }
}
