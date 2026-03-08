use std::collections::HashSet;

use anyhow::{Context, Result, bail};
use bands::LteBand;
use serde_json::Value;

pub mod bands;
#[cfg(feature = "mf289f")]
pub(crate) mod util;

#[cfg(feature = "g5ts")]
pub mod g5ts;

#[cfg(feature = "mf289f")]
pub mod mf289f;

pub(crate) fn normalize_router_url(url: &str) -> Result<String> {
    let mut target = reqwest::Url::parse(url)
        .with_context(|| format!("Invalid router URL: {url}"))?;

    if target.cannot_be_a_base() {
        bail!("Router URL must be an absolute base URL: {url}");
    }

    if target.query().is_some() || target.fragment().is_some() {
        bail!("Router URL must not include a query string or fragment: {url}");
    }

    if !target.path().ends_with('/') {
        let normalized_path = format!("{}/", target.path().trim_end_matches('/'));
        target.set_path(&normalized_path);
    }

    Ok(target.into())
}

#[derive(serde::Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMode {
    #[serde(rename = "auto_dial")]
    Auto,
    #[serde(rename = "manual_dial")]
    Manual,
}

impl Default for ConnectionMode {
    fn default() -> Self {
        ConnectionMode::Auto
    }
}

#[derive(serde::Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BearerPreference {
    #[serde(rename = "NETWORK_auto")]
    Auto,
    /// Prefer 4G+5G automatic selection (G5TS).
    #[serde(rename = "4G_AND_5G")]
    LteAndNr5g,
    /// Prefer 5G NSA mode (G5TS).
    #[serde(rename = "LTE_AND_5G")]
    Nr5gNsa,
    /// Prefer 5G SA / 5G only mode (G5TS).
    #[serde(rename = "Only_5G")]
    OnlyNr5g,
    #[serde(rename = "Only_LTE")]
    OnlyLte,
    #[serde(rename = "Only_GSM")]
    OnlyGsm,
    #[serde(rename = "Only_WCDMA")]
    OnlyWcdma,
}

impl Default for BearerPreference {
    fn default() -> Self {
        BearerPreference::Auto
    }
}

// --- APN types ---

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApnAuthMode {
    None,
    Pap,
    Chap,
    PapChap,
}

impl std::fmt::Display for ApnAuthMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApnAuthMode::None => write!(f, "NONE"),
            ApnAuthMode::Pap => write!(f, "PAP"),
            ApnAuthMode::Chap => write!(f, "CHAP"),
            ApnAuthMode::PapChap => write!(f, "PAP_CHAP"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdpType {
    IPv4,
    IPv6,
    IPv4v6,
}

impl std::fmt::Display for PdpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdpType::IPv4 => write!(f, "IPv4"),
            PdpType::IPv6 => write!(f, "IPv6"),
            PdpType::IPv4v6 => write!(f, "IPv4v6"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApnProfile {
    pub profile_id: Option<String>,
    pub profile_name: String,
    pub apn: String,
    pub pdp_type: PdpType,
    pub auth_mode: ApnAuthMode,
    pub username: String,
    pub password: String,
}

// --- DHCP types ---

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DhcpSettings {
    pub ip_address: String,
    pub subnet_mask: String,
    pub dhcp_enabled: bool,
    pub lease_time: u32,
}

// --- MTU types ---

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MtuSettings {
    pub mtu: u32,
    pub mss: u32,
}

// --- SMS types ---

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SmsSettings {
    pub validity: String,
    pub center_number: String,
    pub delivery_report: bool,
}

/// Common trait implemented by all router clients.
///
/// Each router model provides its own implementation. Methods that are not
/// supported by a particular model return an error.
///
/// ```no_run
/// # async fn example() -> anyhow::Result<()> {
/// use zte_cpe_rs::RouterClient;
///
/// // Use the concrete type for the router you have:
/// #[cfg(feature = "gt5s")]
/// let mut router = zte_cpe_rs::gt5s::Gt5sClient::new("https://192.168.0.1")?;
/// #[cfg(feature = "mf289f")]
/// let mut router = zte_cpe_rs::mf289f::Mf289fClient::new("http://192.168.0.1")?;
///
/// router.login("password").await?;
/// router.disconnect_network().await?;
/// router.logout().await?;
/// # Ok(())
/// # }
/// ```
#[async_trait::async_trait]
pub trait RouterClient {
    /// Authenticate with the router.
    async fn login(&mut self, password: &str) -> Result<()>;

    /// Log out from the router.
    async fn logout(&mut self) -> Result<()>;

    /// Disconnect the mobile/WAN network.
    async fn disconnect_network(&self) -> Result<()>;

    /// Connect the mobile/WAN network.
    async fn connect_network(&self) -> Result<()>;

    /// Reboot the router.
    async fn reboot(&self) -> Result<()>;

    /// Get firmware/hardware version as `(version_a, version_b)`.
    ///
    /// MF289F returns `(cr_version, wa_inner_version)`.
    /// G5TS returns `(hardware_version, wa_inner_version)`.
    async fn get_version(&self) -> Result<(String, String)>;

    /// Set connection mode (auto/manual) and roaming.
    async fn set_connection_mode(
        &self,
        connection_mode: ConnectionMode,
        roam: bool,
    ) -> Result<()>;

    /// Set network bearer preference.
    ///
    /// Common values (`Auto`, `OnlyLte`, `OnlyGsm`, `OnlyWcdma`) work on both
    /// families where supported by firmware.
    /// G5TS-specific 5G values (`LteAndNr5g`, `Nr5gNsa`, `OnlyNr5g`) return an
    /// unsupported error on MF289F.
    async fn set_network_bearer_preference(
        &self,
        bearer_preference: BearerPreference,
    ) -> Result<()>;

    /// Enable or disable UPnP.
    async fn set_upnp(&self, enabled: bool) -> Result<()>;

    /// Set DMZ host IP, or disable DMZ with `None`.
    async fn set_dmz(&self, ip_address: Option<String>) -> Result<()>;

    /// Lock specific LTE bands, or unlock all with `None`.
    async fn select_lte_band(&self, lte_band: Option<HashSet<LteBand>>) -> Result<()>;

    /// Set DNS mode: `None` for auto, `Some([primary, secondary])` for manual.
    async fn set_dns(&self, manual: Option<[String; 2]>) -> Result<()>;

    /// Get router status info as JSON.
    async fn get_status(&self) -> Result<Value>;

    /// Get the current APN mode: true = manual, false = auto.
    async fn get_apn_mode(&self) -> Result<bool> {
        bail!("get_apn_mode is not supported on this model")
    }

    /// Set APN mode: true = manual, false = auto.
    async fn set_apn_mode(&self, _manual: bool) -> Result<()> {
        bail!("set_apn_mode is not supported on this model")
    }

    /// List manual APN profiles.
    async fn get_apn_profiles(&self) -> Result<Vec<ApnProfile>> {
        bail!("get_apn_profiles is not supported on this model")
    }

    /// Modify an existing manual APN profile.
    async fn set_apn_profile(&self, _profile: &ApnProfile) -> Result<()> {
        bail!("set_apn_profile is not supported on this model")
    }

    /// Set a manual APN profile as the active/default one.
    async fn enable_apn_profile(&self, _profile_id: &str) -> Result<()> {
        bail!("enable_apn_profile is not supported on this model")
    }

    /// Get current DHCP settings.
    async fn get_dhcp_settings(&self) -> Result<DhcpSettings> {
        bail!("get_dhcp_settings is not supported on this model")
    }

    /// Set DHCP settings.
    async fn set_dhcp_settings(&self, _settings: &DhcpSettings) -> Result<()> {
        bail!("set_dhcp_settings is not supported on this model")
    }

    /// Get current MTU/MSS settings.
    async fn get_mtu_settings(&self) -> Result<MtuSettings> {
        bail!("get_mtu_settings is not supported on this model")
    }

    /// Set MTU/MSS settings.
    async fn set_mtu_settings(&self, _settings: &MtuSettings) -> Result<()> {
        bail!("set_mtu_settings is not supported on this model")
    }

    /// Get SMS settings.
    async fn get_sms_settings(&self) -> Result<SmsSettings> {
        bail!("get_sms_settings is not supported on this model")
    }

    /// Get network/signal information.
    async fn get_network_info(&self) -> Result<Value> {
        bail!("get_network_info is not supported on this model")
    }

    /// Get SIM card information.
    async fn get_sim_info(&self) -> Result<Value> {
        bail!("get_sim_info is not supported on this model")
    }

    /// Get device information (IMEI, versions, etc).
    async fn get_device_info(&self) -> Result<Value> {
        bail!("get_device_info is not supported on this model")
    }

    /// Get connected device list.
    async fn get_connected_devices(&self) -> Result<Value> {
        bail!("get_connected_devices is not supported on this model")
    }
}

