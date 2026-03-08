use std::collections::HashSet;
use std::fmt::{self, Display};
use std::str::FromStr;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use zte_cpe_rs::bands::LteBand;
use zte_cpe_rs::{BearerPreference, ConnectionMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Model {
    #[cfg(feature = "mf289f")]
    Mf289f,
    #[cfg(feature = "gt5s")]
    Gt5s,
}

impl Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "mf289f")]
            Self::Mf289f => f.write_str("mf289f"),
            #[cfg(feature = "gt5s")]
            Self::Gt5s => f.write_str("gt5s"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ConnectionModeArg {
    Auto,
    Manual,
}

impl From<ConnectionModeArg> for ConnectionMode {
    fn from(value: ConnectionModeArg) -> Self {
        match value {
            ConnectionModeArg::Auto => ConnectionMode::Auto,
            ConnectionModeArg::Manual => ConnectionMode::Manual,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum BearerPreferenceArg {
    Auto,
    LteAndNr5g,
    Nr5gNsa,
    OnlyNr5g,
    OnlyLte,
    OnlyGsm,
    OnlyWcdma,
}

impl From<BearerPreferenceArg> for BearerPreference {
    fn from(value: BearerPreferenceArg) -> Self {
        match value {
            BearerPreferenceArg::Auto => BearerPreference::Auto,
            BearerPreferenceArg::LteAndNr5g => BearerPreference::LteAndNr5g,
            BearerPreferenceArg::Nr5gNsa => BearerPreference::Nr5gNsa,
            BearerPreferenceArg::OnlyNr5g => BearerPreference::OnlyNr5g,
            BearerPreferenceArg::OnlyLte => BearerPreference::OnlyLte,
            BearerPreferenceArg::OnlyGsm => BearerPreference::OnlyGsm,
            BearerPreferenceArg::OnlyWcdma => BearerPreference::OnlyWcdma,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SwitchState {
    On,
    Off,
}

impl SwitchState {
    pub fn is_on(self) -> bool {
        matches!(self, Self::On)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DnsModeArg {
    Auto,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DmzTarget {
    Off,
    Ip(String),
}

impl DmzTarget {
    pub fn into_option(self) -> Option<String> {
        match self {
            Self::Off => None,
            Self::Ip(ip) => Some(ip),
        }
    }
}

impl FromStr for DmzTarget {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        if value == "off" {
            Ok(Self::Off)
        } else {
            Ok(Self::Ip(value.to_string()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LteBandSelection {
    All,
    Some(HashSet<LteBand>),
}

impl LteBandSelection {
    pub fn into_option(self) -> Option<HashSet<LteBand>> {
        match self {
            Self::All => None,
            Self::Some(bands) => Some(bands),
        }
    }
}

impl FromStr for LteBandSelection {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        if value == "all" {
            return Ok(Self::All);
        }

        let mut bands = HashSet::new();
        for item in value.split(',') {
            bands.insert(parse_lte_band(item).map_err(|err| err.to_string())?);
        }

        Ok(Self::Some(bands))
    }
}

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(name = "zte-cpe-rs")]
pub struct Cli {
    #[arg(long)]
    pub model: Option<Model>,
    #[arg(long)]
    pub host: String,
    #[arg(long)]
    pub password: String,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Status {
        #[arg(long)]
        pretty: bool,
    },
    Version,
    Connect,
    Disconnect,
    Reboot,
    SetConnectionMode {
        #[arg(value_enum)]
        mode: ConnectionModeArg,
        #[arg(long, conflicts_with = "no_roam")]
        roam: bool,
        #[arg(long)]
        no_roam: bool,
    },
    SetBearer {
        #[arg(value_enum)]
        preference: BearerPreferenceArg,
    },
    SetUpnp {
        #[arg(value_enum)]
        enabled: SwitchState,
    },
    SetDmz {
        target: DmzTarget,
    },
    SetDns {
        #[arg(value_enum)]
        mode: DnsModeArg,
        primary: Option<String>,
        secondary: Option<String>,
    },
    SelectLteBand {
        selection: LteBandSelection,
    },
}

fn parse_lte_band(value: &str) -> Result<LteBand> {
    match value {
        "1" | "band1" => Ok(LteBand::Band1),
        "3" | "band3" => Ok(LteBand::Band3),
        "7" | "band7" => Ok(LteBand::Band7),
        "8" | "band8" => Ok(LteBand::Band8),
        "20" | "band20" => Ok(LteBand::Band20),
        "28" | "band28" => Ok(LteBand::Band28),
        "32" | "band32" => Ok(LteBand::Band32),
        "38" | "band38" => Ok(LteBand::Band38),
        other => bail!("unsupported LTE band: {other}"),
    }
}