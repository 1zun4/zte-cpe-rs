use super::GoformCommand;
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
pub struct LogoutCommand;

impl GoformCommand for LogoutCommand {
    fn goform_id(&self) -> &'static str {
        "LOGOUT"
    }

    fn authenticated(&self) -> bool {
        true
    }
}
