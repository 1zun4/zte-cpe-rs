use std::collections::HashSet;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum LteBand {
    /// Band 1: 2100 MHz (IMT)
    Band1 = 1, // lte_band_lock: 0x1
    /// Band 3: 1800 MHz (DCS)
    Band3 = 3, //
    /// Band 7: 2600 MHz (IMT-E)
    Band7 = 7,
    /// Band 8: 900 MHz (Extended GSM)
    Band8 = 8,
    /// Band 20: 800 MHz (Digital Dividend, EU)
    Band20 = 20,
    /// Band 28: 700 MHz (APT)
    Band28 = 28,
    /// Band 32: 1500 MHz (L-Band, SDL)
    Band32 = 32,
    /// Band 38: 2600 MHz (TDD, IMT-E)
    Band38 = 38,
}

// Predefined bitmask representing all supported LTE bands
pub const ALL_LTE_BANDS: &str = "0x20080800C5";

// Convert the selected bands to a hexadecimal bitmask
fn calculate_bitmask(selected_bands: HashSet<LteBand>) -> String {
    let mut bitmask: u64 = 0;
    for band in selected_bands {
        bitmask |= 1 << ((band as u64) - 1);
    }
    format!("0x{:X}", bitmask)
}

pub async fn select_lte_band(lte_band: Option<HashSet<LteBand>>) -> String {
    match lte_band {
        Some(bands) => calculate_bitmask(bands),
        None => ALL_LTE_BANDS.to_string(),
    }
}
