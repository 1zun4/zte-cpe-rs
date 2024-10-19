use super::GoformCommand;
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct DnsModeCommand {
    #[serde(rename = "dns_mode")]
    pub dns_mode: String,
    #[serde(rename = "prefer_dns_manual")]
    pub prefer_dns_manual: String,
    #[serde(rename = "standby_dns_manual")]
    pub standby_dns_manual: String,
}

impl GoformCommand for DnsModeCommand {
    fn goform_id(&self) -> &'static str {
        "ROUTER_DNS_SETTING"
    }

    fn authenticated(&self) -> bool {
        true
    }
}
