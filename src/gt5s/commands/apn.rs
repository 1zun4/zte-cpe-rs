use super::UbusCommand;
use serde::Serialize;

/// Get current APN mode: 0 = auto, 1 = manual.
#[derive(Serialize, Default)]
pub struct GetApnModeCommand {}

impl UbusCommand for GetApnModeCommand {
    fn module(&self) -> &'static str {
        "zwrt_apn_object"
    }
    fn method(&self) -> &'static str {
        "get_apn_mode"
    }
}

/// Set APN mode: 0 = auto, 1 = manual.
#[derive(Serialize)]
pub struct SetApnModeCommand {
    pub apn_mode: i32,
}

impl UbusCommand for SetApnModeCommand {
    fn module(&self) -> &'static str {
        "zwrt_apn_object"
    }
    fn method(&self) -> &'static str {
        "set_apn_mode"
    }
}

/// Get list of manually configured APN profiles.
#[derive(Serialize, Default)]
pub struct GetManuApnListCommand {}

impl UbusCommand for GetManuApnListCommand {
    fn module(&self) -> &'static str {
        "zwrt_apn_object"
    }
    fn method(&self) -> &'static str {
        "getManuApnList"
    }
}

/// Modify an existing manual APN profile.
#[derive(Serialize)]
pub struct ModifyManuApnCommand {
    #[serde(rename = "profilename")]
    pub profile_name: String,
    #[serde(rename = "pdpType")]
    pub pdp_type: &'static str,
    #[serde(rename = "wanapn")]
    pub apn: String,
    #[serde(rename = "pppAuthMode")]
    pub auth_mode: &'static str,
    pub username: String,
    /// AES-GCM encrypted password.
    pub password: String,
    #[serde(rename = "profileId")]
    pub profile_id: String,
}

impl UbusCommand for ModifyManuApnCommand {
    fn module(&self) -> &'static str {
        "zwrt_apn_object"
    }
    fn method(&self) -> &'static str {
        "modifyManuApn"
    }
}

/// Enable/activate a manual APN profile by its ID.
#[derive(Serialize)]
pub struct EnableManuApnCommand {
    #[serde(rename = "profileId")]
    pub profile_id: String,
}

impl UbusCommand for EnableManuApnCommand {
    fn module(&self) -> &'static str {
        "zwrt_apn_object"
    }
    fn method(&self) -> &'static str {
        "enable_manu_apn_id"
    }
}
