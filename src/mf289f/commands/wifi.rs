use super::GoformCommand;
use serde::Serialize;

#[derive(Serialize)]
pub enum WiFiCoverage {
    #[serde(rename = "short_mode")]
    Short,
    #[serde(rename = "medium_mode")]
    Medium,
    #[serde(rename = "long_mode")]
    Long,
}

impl Default for WiFiCoverage {
    fn default() -> Self {
        WiFiCoverage::Long
    }
}

#[derive(Serialize, Default)]
pub struct WiFiCoverageCommand {
    #[serde(rename = "WiFiCoverage")]
    pub wifi_coverage: WiFiCoverage,
}

impl GoformCommand for WiFiCoverageCommand {
    fn goform_id(&self) -> &'static str {
        "setWiFiCoverage"
    }

    fn authenticated(&self) -> bool {
        true
    }
}
