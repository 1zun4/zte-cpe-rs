# zte-cpe-rs

A Rust library for interacting with ZTE devices, such as the GigaCube ZTE MF289F and ZTE GT5S.

## Supported Devices

- ZTE GT5S
- GigaCube ZTE MF289F (Last tested: https://github.com/1zun4/zte-cpe-rs/commit/bdd76f850785e76be45a149a8e7d72c7eb99da11)

## Features
| Feature | MF289F | GT5S |
| --- | --- | --- |
| Device reboot | Yes | Yes |
| Status information / monitoring | Yes | Yes |
| Connect and disconnect network | Yes | Yes |
| Set connection mode | Yes | Yes |
| Set bearer preference | Yes | Yes |
| Set LTE band lock | Yes | No |
| Set DNS mode configuration | Yes | No |
| Configure UPnP | Yes | Yes |
| Configure DMZ | Yes | Yes |

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
    let mut router = Mf289fClient::new("giga.cube")
        .context("Failed to create MF289F client")?;
    // For a GT5S, use `zte_cpe_rs::gt5s::Gt5sClient` instead.
    
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

Build the project:

```sh
cargo build
```

Build the CLI for a specific router family:

```sh
# GT5S over HTTPS with rustls
cargo build --no-default-features --features cli,gt5s,tls-rustls

# MF289F over HTTP
cargo build --no-default-features --features cli,mf289f
```

Use the CLI:

```sh
# When both model features are compiled in, pass --model.
cargo run --features cli -- --model gt5s --host 192.168.0.1 --password YOURPASSWORD status

# Pretty-print JSON output.
cargo run --features cli -- --model gt5s --host 192.168.0.1 --password YOURPASSWORD status --pretty

# Model-specific builds can omit --model when only one family is enabled.
cargo run --no-default-features --features cli,gt5s,tls-rustls -- \
    --host 192.168.0.1 --password YOURPASSWORD version
```

Run tests:

```sh
cargo test
```

## Acknowledgements

This project was inspired by and uses code from:

- [ZTE-MC-Home-assistant](https://github.com/Kajkac/ZTE-MC-Home-assistant/blob/master/python_scripts/zte_tool.py)
- [zte-cpe](https://github.com/SpeckyYT/zte-cpe)
- [zte-v3.0b.min.txt](https://miononno.it/files/zte-v3.0b.min.txt)

## License

This project is licensed under the GNU GENERAL PUBLIC LICENSE.
