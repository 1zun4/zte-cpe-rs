use super::UbusCommand;
use serde::Serialize;

/// Reboot the router.
#[derive(Serialize)]
pub struct RebootCommand {
    #[serde(rename = "moduleName")]
    pub module_name: &'static str,
}

impl Default for RebootCommand {
    fn default() -> Self {
        Self {
            module_name: "web",
        }
    }
}

impl UbusCommand for RebootCommand {
    fn module(&self) -> &'static str {
        "zwrt_mc.device.manager"
    }
    fn method(&self) -> &'static str {
        "device_reboot"
    }
}

/// Get the router status summary.
#[derive(Serialize, Default)]
pub struct GetRouterStatusCommand {}

impl UbusCommand for GetRouterStatusCommand {
    fn module(&self) -> &'static str {
        "zwrt_router.api"
    }
    fn method(&self) -> &'static str {
        "router_get_status"
    }
}

/// Get the number of connected users.
#[derive(Serialize, Default)]
pub struct GetUserListNumCommand {}

impl UbusCommand for GetUserListNumCommand {
    fn module(&self) -> &'static str {
        "zwrt_router.api"
    }
    fn method(&self) -> &'static str {
        "router_get_user_list_num"
    }
}

/// Get the LAN access list (connected devices).
#[derive(Serialize, Default)]
pub struct GetLanAccessListCommand {}

impl UbusCommand for GetLanAccessListCommand {
    fn module(&self) -> &'static str {
        "zwrt_router.api"
    }
    fn method(&self) -> &'static str {
        "router_lan_access_list"
    }
}

/// Read a UCI config section.
#[derive(Serialize)]
pub struct UciGetCommand {
    pub config: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

impl UbusCommand for UciGetCommand {
    fn module(&self) -> &'static str {
        "uci"
    }
    fn method(&self) -> &'static str {
        "get"
    }
}
