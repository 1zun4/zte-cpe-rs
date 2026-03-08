// on: isTest=false&goformId=SetUpgAutoSetting&UpgMode=1&UpgIntervalDay=7&UpgRoamPermission=0&AD=706ea33bd8322ca1eada95f75f878640
// off: isTest=false&goformId=SetUpgAutoSetting&UpgMode=0&UpgIntervalDay=7&UpgRoamPermission=0&AD=78abc373606c29f1eb3b05d38002f911

use super::GoformCommand;
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct AutoUpgradeCommand {
    #[serde(rename = "UpgMode")]
    #[serde(serialize_with = "crate::util::bool_to_int")]
    pub upg_mode: bool,
    #[serde(rename = "UpgIntervalDay")]
    pub upg_interval_day: i32,
    #[serde(rename = "UpgRoamPermission")]
    #[serde(serialize_with = "crate::util::bool_to_int")]
    pub upg_roam_permission: bool,
}

impl GoformCommand for AutoUpgradeCommand {
    fn goform_id(&self) -> &'static str {
        "SetUpgAutoSetting"
    }

    fn authenticated(&self) -> bool {
        true
    }
}
