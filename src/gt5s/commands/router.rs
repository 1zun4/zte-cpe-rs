use super::UbusCommand;
use serde::Serialize;

/// Set DHCP / LAN parameters.
#[derive(Serialize)]
pub struct SetLanParaCommand {
    pub ipaddr: String,
    pub netmask: String,
    /// "0" = DHCP enabled, "1" = DHCP disabled.
    pub ignore: &'static str,
    pub leasetime: String,
}

impl UbusCommand for SetLanParaCommand {
    fn module(&self) -> &'static str {
        "zwrt_router.api"
    }
    fn method(&self) -> &'static str {
        "router_set_lan_para"
    }
}

/// Set MTU/MSS values.
#[derive(Serialize)]
pub struct SetWanMtuCommand {
    pub mtu: String,
    pub mss: String,
}

impl UbusCommand for SetWanMtuCommand {
    fn module(&self) -> &'static str {
        "zwrt_router.api"
    }
    fn method(&self) -> &'static str {
        "router_set_wan_mtu"
    }
}

/// Enable or disable UPnP.
#[derive(Serialize)]
pub struct SetUpnpCommand {
    pub enable_upnp: i32,
}

impl UbusCommand for SetUpnpCommand {
    fn module(&self) -> &'static str {
        "zwrt_router.api"
    }
    fn method(&self) -> &'static str {
        "router_set_upnp_switch"
    }
}

/// Set or disable DMZ.
#[derive(Serialize)]
pub struct SetDmzCommand {
    pub dmz_enable: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dmz_ip: Option<String>,
}

impl UbusCommand for SetDmzCommand {
    fn module(&self) -> &'static str {
        "zwrt_router.api"
    }
    fn method(&self) -> &'static str {
        "router_set_dmz"
    }
}
