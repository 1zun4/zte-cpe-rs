use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use zte_cpe_rs::RouterClient;

use crate::cli::args::{Cli, Command, DnsModeArg, Model};

pub async fn run(cli: Cli) -> Result<()> {
    let model = cli
        .model
        .or(default_model())
        .ok_or_else(|| anyhow!("--model is required when multiple router model features are compiled in"))?;
    let mut client = build_client(model, &cli.host)?;

    client
        .login(&cli.password)
        .await
        .with_context(|| format!("failed to login to {} at {}", model, cli.host))?;

    let result = execute_command(client.as_mut(), cli.command).await;
    let logout_result = client.logout().await;

    result?;
    logout_result.context("command completed, but logout failed")?;
    Ok(())
}

async fn execute_command(client: &mut dyn RouterClient, command: Command) -> Result<()> {
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

fn build_client(model: Model, host: &str) -> Result<Box<dyn RouterClient>> {
    match model {
        #[cfg(feature = "mf289f")]
        Model::Mf289f => Ok(Box::new(zte_cpe_rs::mf289f::Mf289fClient::new(host)?)),
        #[cfg(feature = "gt5s")]
        Model::Gt5s => Ok(Box::new(zte_cpe_rs::gt5s::Gt5sClient::new(host)?)),
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