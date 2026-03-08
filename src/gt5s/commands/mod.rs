pub mod apn;
pub mod auth;
pub mod device;
pub mod network;
pub mod router;
pub mod sms;

use serde::Serialize;
pub use {apn::*, auth::*, device::*, network::*, router::*, sms::*};

/// Trait for GT5S ubus JSON-RPC commands.
///
/// Each command maps to a ubus `call` with `(session, module, method, params)`.
pub trait UbusCommand: Serialize {
    /// The ubus module name (e.g. `"zwrt_web"`, `"zwrt_router.api"`).
    fn module(&self) -> &'static str;

    /// The ubus method name (e.g. `"web_login"`, `"router_set_dmz"`).
    fn method(&self) -> &'static str;

    /// Whether this command requires an authenticated session.
    /// Commands that use `NULL_SESSION` (like login info) return false.
    fn authenticated(&self) -> bool {
        true
    }
}
