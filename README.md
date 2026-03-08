# zte-cpe-rs

A Rust library for interacting with ZTE devices, such as the GigaCube ZTE MF289F and ZTE G5TS.

## Supported Devices

- ZTE G5TS
- GigaCube ZTE MF289F (Last tested: https://github.com/1zun4/zte-cpe-rs/commit/bdd76f850785e76be45a149a8e7d72c7eb99da11)

## Features
| Feature | MF289F | G5TS |
| --- | --- | --- |
| Device reboot | Yes | Yes |
| Get status info | Yes | Yes |
| Get device info | No | Yes |
| Get network/signal information | No | Yes |
| Get SIM card info | No | Yes |
| Connect and disconnect network | Yes | Yes |
| Set connection mode | Yes | Yes |
| Set bearer preference | Yes | Yes |
| Set LTE band lock | Yes | No |
| Set DNS mode | Yes | No |
| Configure UPnP | Yes | Yes |
| Configure DMZ | Yes | Yes |
| Get APN profiles | No | Yes |
| Modify an APN profile | No | Yes |
| Get DHCP settings | No | Yes |
| Set DHCP settings | No | Yes |
| Get MTU/MSS settings | No | Yes |
| Set MTU/MSS settings | No | Yes |
| Get SMS settings | No | Yes |
| Get connected devices | No | Yes |

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zte-cpe-rs = "0.2.1"
```

The library builds by default. The CLI is optional and is only compiled when you enable the `cli` feature.

## Usage

Here's a basic example of how to use `zte-cpe-rs`:

```rust
use std::collections::HashSet;

use anyhow::{Context, Result};
use zte_cpe_rs::{bands::LteBand, mf289f::Mf289fClient, RouterClient};

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = Mf289fClient::new("http://giga.cube")
        .context("Failed to create MF289F client")?;
    // For a G5TS, use `zte_cpe_rs::g5ts::G5tsClient` instead.
    
    // Login
    router.login("YOURPASSWORD")
        .await
        .context("Failed to login")?;

    // Disconnect network
    router.disconnect_network().await?;

    // Connect network
    router.connect_network().await?;

    // Get status
    println!("{}", router.get_status().await?);

    // Set LTE band
    let mut bands = HashSet::new();
    bands.insert(LteBand::Band1);
    bands.insert(LteBand::Band3);
    bands.insert(LteBand::Band7);

    router.select_lte_band(Some(bands))
        .await?;

    // Logout
    router.logout().await?;

    Ok(())
}
```

## Setup

Clone the repository:

```sh
git clone https://github.com/1zun4/zte-cpe-rs.git
cd zte-cpe-rs
```

Use the CLI:

```sh
cargo run --features cli -- --model g5ts --url https://192.168.0.1 --password YOURPASSWORD status
cargo run --features cli -- --model g5ts --url https://192.168.0.1 --password YOURPASSWORD status --pretty
```

## Acknowledgements

This project was inspired by and uses code from:

- [ZTE-MC-Home-assistant](https://github.com/Kajkac/ZTE-MC-Home-assistant/blob/master/python_scripts/zte_tool.py)
- [zte-cpe](https://github.com/SpeckyYT/zte-cpe)
- [zte-v3.0b.min.txt](https://miononno.it/files/zte-v3.0b.min.txt)

## License

This project is licensed under the GNU GENERAL PUBLIC LICENSE.
