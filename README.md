# zte-cpe-rs

🚀 A Rust library for interacting with ZTE devices, such as the GigaCube ZTE MF289F.

## Supported Devices

- GigaCube ZTE MF289F

## Features
- 🔄 Device Reboot
- 📊 Device Status Information / Monitoring
- 📡 Connect and Disconnect Network
- 🔀 Set Connection Mode
- 🛡️ Set Bearer Preference
- 🔒 Set LTE Band Lock
- 🌐 Set DNS mode configuration
- 📶 Set WiFi Coverage
- 🔌 Configure UPnP
- 🌐 Configure DMZ
- ♻️ Manage Auto Update

More features coming soon...

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zte-cpe-rs = "0.2.0"
```

## Usage

Here's a basic example of how to use `zte-cpe-rs`:

```rust
use std::collections::HashSet;

use anyhow::{Context, Result};
use zte_cpe_rs::{bands::LteBand, ZteClient};

#[tokio::main]
async fn main() -> Result<()> {
    let mut zte_client = ZteClient::new("giga.cube")
        .context("Failed to create ZteClient")?;

    // Login
    zte_client.login("YOURPASSWORD".to_string())
        .await
        .context("Failed to login")?;

    // Disconnect network
    zte_client.disconnect_network().await?;

    // Connect network
    zte_client.connect_network().await?;

    // Get status
    println!("{}", zte_client.get_status().await?);

    // Set LTE band
    let mut bands = HashSet::new();
    bands.insert(LteBand::Band1);
    bands.insert(LteBand::Band3);
    bands.insert(LteBand::Band7);
    
    zte_client.select_lte_band(Some(bands))
        .await?;

    // Logout
    zte_client.logout().await?;

    Ok(())
}
```

## Contributing

We welcome contributions! To get started, follow these steps:

1. **Fork the repository**: Click the "Fork" button at the top right of this page.
2. **Clone your fork**: 
    ```sh
    git clone https://github.com/yourusername/zte-cpe-rs.git
    cd zte-cpe-rs
    ```
3. **Create a new branch**: 
    ```sh
    git checkout -b feature/your-feature-name
    ```
4. **Make your changes**: Implement your feature or fix the bug.
5. **Commit your changes**: 
    ```sh
    git commit -am 'Add a meaningful commit message'
    ```
6. **Push to your branch**: 
    ```sh
    git push origin feature/your-feature-name
    ```
7. **Open a Pull Request**: Go to the original repository and click the "New Pull Request" button.

Please ensure your code adheres to the project's coding standards and includes appropriate tests.

Thank you for your contributions!

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
