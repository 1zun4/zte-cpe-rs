use std::collections::HashSet;

use crate::bands::{select_lte_band, LteBand, ALL_LTE_BANDS};
use crate::g5ts::g5ts_password_hash;
use crate::{Model, Router};

#[tokio::test]
async fn test_mf289f_login_hash() {
    let password = "ZTEPASSWORD";
    let ld = "91CF42608863DB6DC767F5B8E3D6E2F8656016D53B6AAF68E833373587C73BD2";

    let hash_password = sha256::digest(password.as_bytes()).to_uppercase();
    let zte_pass = sha256::digest(&(hash_password + &ld)).to_uppercase();

    let expected_zte_pass = "77675C36BEE89745EA52CCEFE3D862F9D1CDDF8DE304E2FD1776E2A4FDFE1A9D";
    assert_eq!(zte_pass, expected_zte_pass);
}

#[tokio::test]
async fn test_mf289f_ad_hash() {
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

#[tokio::test]
async fn test_gt5s_password_hash() {
    let password = "ZTEPASSWORD";
    let salt = "D537DFE05F21D78962E8996209E6ECA2CCCC7BE985DC55FC47F5EC38F460EBA6";

    let hash = g5ts_password_hash(password, salt);

    // SHA256("ZTEPASSWORD") = D2989352E891805206B7DD0072EE7718EF00403FFE563998880E60B082392728
    let expected_inner = "D2989352E891805206B7DD0072EE7718EF00403FFE563998880E60B082392728";
    assert_eq!(
        sha256::digest(password.as_bytes()).to_uppercase(),
        expected_inner
    );

    // SHA256(inner + salt) = final hash (uppercase)
    let concat = format!("{}{}", expected_inner, salt);
    let expected_hash = sha256::digest(concat.as_bytes()).to_uppercase();
    assert_eq!(hash, expected_hash);
}

#[tokio::test]
async fn test_gt5s_password_hash_uppercase_output() {
    // Verify the hash is always uppercase hex
    let hash = g5ts_password_hash("test", "ABCDEF");
    assert_eq!(hash, hash.to_uppercase());
    assert_eq!(hash.len(), 64);
}

#[test]
fn test_router_new_mf289f() {
    let router = Router::new(Model::MF289F, "192.168.0.1");
    assert!(router.is_ok());
}

#[test]
fn test_router_new_g5ts() {
    let router = Router::new(Model::G5TS, "192.168.0.1");
    assert!(router.is_ok());
}

#[tokio::test]
async fn test_g5ts_unsupported_reboot() {
    let router = Router::new(Model::G5TS, "192.168.0.1").unwrap();
    let result = router.reboot().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not supported"));
}

#[tokio::test]
async fn test_g5ts_unsupported_set_upnp() {
    let router = Router::new(Model::G5TS, "192.168.0.1").unwrap();
    let result = router.set_upnp(true).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not supported"));
}

#[tokio::test]
async fn test_g5ts_unsupported_set_dmz() {
    let router = Router::new(Model::G5TS, "192.168.0.1").unwrap();
    let result = router.set_dmz(None).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not supported"));
}

#[tokio::test]
async fn test_g5ts_unsupported_select_lte_band() {
    let router = Router::new(Model::G5TS, "192.168.0.1").unwrap();
    let result = router.select_lte_band(None).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not supported"));
}

#[tokio::test]
async fn test_g5ts_unsupported_get_status() {
    let router = Router::new(Model::G5TS, "192.168.0.1").unwrap();
    let result = router.get_status().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not supported"));
}
