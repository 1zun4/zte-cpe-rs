pub mod auth;
pub mod device;
pub mod network;
pub mod dhcp;
pub mod wifi;
pub mod dmz;
pub mod upnp;
pub mod update;

use serde::Serialize;
pub use {
    auth::*,
    device::*,
    network::*,
    dhcp::*,
    wifi::*,
    dmz::*,
    upnp::*,
    update::*,
};

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
    fn authenticated(&self) -> bool { false }
}