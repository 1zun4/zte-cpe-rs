// on: goformId=UPNP_SETTING&isTest=false&upnp_setting_option=1&AD=0af55b396e012b08cd433ffd7a6ee6ce
// off: goformId=UPNP_SETTING&isTest=false&upnp_setting_option=0&AD=5e53a8fafcf3530e40550394a31b5e3b

use serde::Serialize;
use super::GoformCommand;

#[derive(Serialize, Default)]
pub struct UpnpCommand {
    #[serde(rename = "upnp_setting_option")]
    #[serde(serialize_with = "crate::util::bool_to_int")]
    pub upnp_setting_option: bool,
}

impl GoformCommand for UpnpCommand {
    fn goform_id(&self) -> &'static str {
        "UPNP_SETTING"
    }
    
    fn authenticated(&self) -> bool {
        true
    }
}
