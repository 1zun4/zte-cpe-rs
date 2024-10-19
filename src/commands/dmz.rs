// on: goformId=DMZ_SETTING&isTest=false&DMZEnabled=1&DMZIPAddress=192.168.1.1&AD=47bd367d64cca07bac17b798d8bfa2d5
// off: goformId=DMZ_SETTING&isTest=false&DMZEnabled=0&AD=93629140ea995b60d5a83cdcc9d2f5ed

use super::GoformCommand;
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct DmzCommand {
    #[serde(rename = "DMZEnabled")]
    #[serde(serialize_with = "crate::util::bool_to_int")]
    pub dmz_enabled: bool,
    #[serde(rename = "DMZIPAddress")]
    pub dmz_ip_address: Option<String>,
}

impl GoformCommand for DmzCommand {
    fn goform_id(&self) -> &'static str {
        "DMZ_SETTING"
    }

    fn authenticated(&self) -> bool {
        true
    }
}
