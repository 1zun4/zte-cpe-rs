use crate::g5ts::{G5tsClient, g5ts_password_hash};
use crate::RouterClient;

#[tokio::test]
async fn test_g5ts_password_hash() {
    let password = "ZTEPASSWORD";
    let salt = "D537DFE05F21D78962E8996209E6ECA2CCCC7BE985DC55FC47F5EC38F460EBA6";

    let hash = g5ts_password_hash(password, salt);

    let expected_inner = "D2989352E891805206B7DD0072EE7718EF00403FFE563998880E60B082392728";
    assert_eq!(
        sha256::digest(password.as_bytes()).to_uppercase(),
        expected_inner
    );

    let concat = format!("{}{}", expected_inner, salt);
    let expected_hash = sha256::digest(concat.as_bytes()).to_uppercase();
    assert_eq!(hash, expected_hash);
}

#[tokio::test]
async fn test_g5ts_password_hash_uppercase_output() {
    let hash = g5ts_password_hash("test", "ABCDEF");
    assert_eq!(hash, hash.to_uppercase());
    assert_eq!(hash.len(), 64);
}

#[test]
fn test_g5ts_client_new() {
    let client = G5tsClient::new("https://192.168.0.1");
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_g5ts_unsupported_set_dns() {
    let client = G5tsClient::new("https://192.168.0.1").unwrap();
    let result = client.set_dns(None).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not directly supported"));
}

#[tokio::test]
async fn test_g5ts_select_lte_band_not_supported() {
    let client = G5tsClient::new("https://192.168.0.1").unwrap();
    let result = client.select_lte_band(None).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not supported"));
}
