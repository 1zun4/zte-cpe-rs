use std::collections::HashSet;

use crate::bands::{select_lte_band, LteBand, ALL_LTE_BANDS};

#[tokio::test]
async fn test_login_hash() {
    let password = "ZTEPASSWORD";
    let ld = "91CF42608863DB6DC767F5B8E3D6E2F8656016D53B6AAF68E833373587C73BD2";

    let hash_password = sha256::digest(password.as_bytes()).to_uppercase();
    let zte_pass = sha256::digest(&(hash_password + &ld)).to_uppercase();

    let expected_zte_pass = "77675C36BEE89745EA52CCEFE3D862F9D1CDDF8DE304E2FD1776E2A4FDFE1A9D";
    assert_eq!(zte_pass, expected_zte_pass);
}

#[tokio::test]
async fn test_ad_hash() {
    let rd = "4079016d940210b4ae9ae7d41c4a2065";
    let wa_inner_version = "BD_VDFDEMF289FV1.0.0B08 [Jun 18 2022 05:39:38]";
    let cr_version = "CR_VDFDEMF289FV1.0.0B08";

    let expected_ad = "a8dc582d399e07ffe8ce4d61573543a1";

    // a = wa_inner_version + cr_version
    let a = md5::compute(format!("{}{}", wa_inner_version, cr_version));
    let a = format!("{:x}", a);
    let result = md5::compute(&(a + &rd));
    let result = format!("{:x}", result);
    assert_eq!(result, expected_ad);
}

#[tokio::test]
async fn test_select_lte_band_all() {
    // Test when None is passed (should use ALL_LTE_BANDS)
    let bitmask = select_lte_band(None).await;

    assert_eq!(bitmask, ALL_LTE_BANDS);
}

#[tokio::test]
async fn test_select_lte_band_specific() {
    let mut bands = HashSet::new();

    // 1+3+20 = lte_band_lock: 0x80005
    bands.insert(LteBand::Band1);
    bands.insert(LteBand::Band3);
    bands.insert(LteBand::Band20);

    // Expected bitmask: 1+3+20 -> 0x80005
    let expected_bitmask = "0x80005";
    let bitmask = select_lte_band(Some(bands)).await;

    assert_eq!(bitmask, expected_bitmask);
}
