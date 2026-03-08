use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use zte_cpe_rs::{ApnAuthMode, ApnProfile, PdpType, RouterClient};

use crate::cli::args::{Cli, Command, DnsModeArg, Model};

pub async fn run(cli: Cli) -> Result<()> {
    let model = cli
        .model
        .or(default_model())
        .ok_or_else(|| anyhow!("--model is required when multiple router model features are compiled in"))?;
    let mut client = build_client(model, &cli.url)?;

    client
        .login(&cli.password)
        .await
        .with_context(|| format!("failed to login to {} at {}", model, cli.url))?;

    let result = execute_command(client.as_mut(), cli.command).await;
    let logout_result = client.logout().await;

    result?;
    logout_result.context("command completed, but logout failed")?;
    Ok(())
}

async fn execute_command(client: &mut (dyn RouterClient + Send + Sync), command: Command) -> Result<()> {
    match command {
        Command::Status { pretty } => {
            let status = client.get_status().await?;
            print_json(&status, pretty)?;
        }
        Command::Version => {
            let (primary, secondary) = client.get_version().await?;
            println!("primary={primary}");
            println!("secondary={secondary}");
        }
        Command::Connect => {
            client.connect_network().await?;
            println!("ok");
        }
        Command::Disconnect => {
            client.disconnect_network().await?;
            println!("ok");
        }
        Command::Reboot => {
            client.reboot().await?;
            println!("ok");
        }
        Command::SetConnectionMode {
            mode,
            roam,
            no_roam,
        } => {
            let roam = if no_roam { false } else { roam };
            client.set_connection_mode(mode.into(), roam).await?;
            println!("ok");
        }
        Command::SetBearer { preference } => {
            client
                .set_network_bearer_preference(preference.into())
                .await?;
            println!("ok");
        }
        Command::SetUpnp { enabled } => {
            client.set_upnp(enabled.is_on()).await?;
            println!("ok");
        }
        Command::SetDmz { target } => {
            client.set_dmz(target.into_option()).await?;
            println!("ok");
        }
        Command::SetDns {
            mode,
            primary,
            secondary,
        } => {
            let manual = match mode {
                DnsModeArg::Auto => {
                    if primary.is_some() || secondary.is_some() {
                        bail!("set-dns auto does not accept DNS server arguments")
                    }
                    None
                }
                DnsModeArg::Manual => Some([
                    primary.ok_or_else(|| anyhow!("set-dns manual requires <primary> <secondary>"))?,
                    secondary.ok_or_else(|| anyhow!("set-dns manual requires <primary> <secondary>"))?,
                ]),
            };

            client.set_dns(manual).await?;
            println!("ok");
        }
        Command::SelectLteBand { selection } => {
            client.select_lte_band(selection.into_option()).await?;
            println!("ok");
        }
        Command::GetApn => {
            let profiles = client.get_apn_profiles().await?;
            let mode = client.get_apn_mode().await?;
            println!("APN mode: {}", if mode { "manual" } else { "auto" });
            let json = serde_json::to_string_pretty(&profiles)?;
            println!("{json}");
        }
        Command::SetApn {
            id,
            name,
            apn,
            pdp_type,
            auth,
            username,
            password,
        } => {
            // Fetch current profile to use as defaults
            let profiles = client.get_apn_profiles().await?;
            let current = profiles
                .iter()
                .find(|p| p.profile_id.as_deref() == Some(&id))
                .ok_or_else(|| anyhow!("APN profile with id '{}' not found", id))?;

            let new_pdp_type = match pdp_type.as_deref() {
                Some("ipv4") | Some("IPv4") => PdpType::IPv4,
                Some("ipv6") | Some("IPv6") => PdpType::IPv6,
                Some("ipv4v6") | Some("IPv4v6") => PdpType::IPv4v6,
                Some(other) => bail!("Unknown PDP type: {}", other),
                None => current.pdp_type,
            };
            let new_auth = match auth.as_deref() {
                Some("none") | Some("NONE") => ApnAuthMode::None,
                Some("pap") | Some("PAP") => ApnAuthMode::Pap,
                Some("chap") | Some("CHAP") => ApnAuthMode::Chap,
                Some("pap-chap") | Some("PAP_CHAP") => ApnAuthMode::PapChap,
                Some(other) => bail!("Unknown auth mode: {}", other),
                None => current.auth_mode,
            };

            let profile = ApnProfile {
                profile_id: Some(id),
                profile_name: name.unwrap_or_else(|| current.profile_name.clone()),
                apn: apn.unwrap_or_else(|| current.apn.clone()),
                pdp_type: new_pdp_type,
                auth_mode: new_auth,
                username: username.unwrap_or_else(|| current.username.clone()),
                password: password.unwrap_or_default(),
            };

            client.set_apn_profile(&profile).await?;
            println!("ok");
        }
        Command::GetDhcp => {
            let settings = client.get_dhcp_settings().await?;
            let json = serde_json::to_string_pretty(&settings)?;
            println!("{json}");
        }
        Command::SetDhcp {
            ip,
            subnet,
            enabled,
            lease_time,
        } => {
            let current = client.get_dhcp_settings().await?;
            let settings = zte_cpe_rs::DhcpSettings {
                ip_address: ip.unwrap_or(current.ip_address),
                subnet_mask: subnet.unwrap_or(current.subnet_mask),
                dhcp_enabled: enabled.map(|s| s.is_on()).unwrap_or(current.dhcp_enabled),
                lease_time: lease_time.unwrap_or(current.lease_time),
            };
            client.set_dhcp_settings(&settings).await?;
            println!("ok");
        }
        Command::GetMtu => {
            let settings = client.get_mtu_settings().await?;
            let json = serde_json::to_string_pretty(&settings)?;
            println!("{json}");
        }
        Command::SetMtu { mtu, mss } => {
            let current = client.get_mtu_settings().await?;
            let settings = zte_cpe_rs::MtuSettings {
                mtu: mtu.unwrap_or(current.mtu),
                mss: mss.unwrap_or(current.mss),
            };
            client.set_mtu_settings(&settings).await?;
            println!("ok");
        }
        Command::GetSmsSettings => {
            let settings = client.get_sms_settings().await?;
            let json = serde_json::to_string_pretty(&settings)?;
            println!("{json}");
        }
        Command::NetworkInfo { pretty } => {
            let info = client.get_network_info().await?;
            print_json(&info, pretty)?;
        }
        Command::SimInfo { pretty } => {
            let info = client.get_sim_info().await?;
            print_json(&info, pretty)?;
        }
        Command::DeviceInfo { pretty } => {
            let info = client.get_device_info().await?;
            print_json(&info, pretty)?;
        }
        Command::ConnectedDevices { pretty } => {
            let info = client.get_connected_devices().await?;
            print_json(&info, pretty)?;
        }
    }

    Ok(())
}

fn print_json(value: &Value, pretty: bool) -> Result<()> {
    let text = if pretty {
        serde_json::to_string_pretty(value)?
    } else {
        serde_json::to_string(value)?
    };

    println!("{text}");
    Ok(())
}

fn build_client(model: Model, url: &str) -> Result<Box<dyn RouterClient + Send + Sync>> {
    match model {
        #[cfg(feature = "mf289f")]
        Model::Mf289f => Ok(Box::new(zte_cpe_rs::mf289f::Mf289fClient::new(url)?)),
        #[cfg(feature = "gt5s")]
        Model::Gt5s => Ok(Box::new(zte_cpe_rs::gt5s::Gt5sClient::new(url)?)),
    }
}

fn default_model() -> Option<Model> {
    #[cfg(all(feature = "mf289f", not(feature = "gt5s")))]
    {
        return Some(Model::Mf289f);
    }

    #[cfg(all(feature = "gt5s", not(feature = "mf289f")))]
    {
        return Some(Model::Gt5s);
    }

    #[cfg(any(
        all(feature = "mf289f", feature = "gt5s"),
        not(any(feature = "mf289f", feature = "gt5s"))
    ))]
    {
        None
    }
}