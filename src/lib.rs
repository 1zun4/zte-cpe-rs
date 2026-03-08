use std::collections::HashSet;

use anyhow::{Context, Result, bail};
use bands::LteBand;
use serde_json::Value;

pub mod bands;
#[cfg(feature = "mf289f")]
pub(crate) mod util;

#[cfg(feature = "gt5s")]
pub mod gt5s;

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
    /// Prefer 4G+5G automatic selection (GT5S).
    #[serde(rename = "4G_AND_5G")]
    LteAndNr5g,
    /// Prefer 5G NSA mode (GT5S).
    #[serde(rename = "LTE_AND_5G")]
    Nr5gNsa,
    /// Prefer 5G SA / 5G only mode (GT5S).
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
    /// GT5S returns `(hardware_version, wa_inner_version)`.
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
    /// GT5S-specific 5G values (`LteAndNr5g`, `Nr5gNsa`, `OnlyNr5g`) return an
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
}

