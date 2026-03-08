use super::UbusCommand;
use serde::Serialize;

/// Fetch login info (salt + remaining attempts). Uses null session.
#[derive(Serialize, Default)]
pub struct LoginInfoCommand {}

impl UbusCommand for LoginInfoCommand {
    fn module(&self) -> &'static str {
        "zwrt_web"
    }
    fn method(&self) -> &'static str {
        "web_login_info"
    }
    fn authenticated(&self) -> bool {
        false
    }
}

/// Login with a pre-hashed password.
#[derive(Serialize)]
pub struct LoginCommand {
    pub password: String,
}

impl UbusCommand for LoginCommand {
    fn module(&self) -> &'static str {
        "zwrt_web"
    }
    fn method(&self) -> &'static str {
        "web_login"
    }
    fn authenticated(&self) -> bool {
        false
    }
}

/// Logout from the router.
#[derive(Serialize, Default)]
pub struct LogoutCommand {}

impl UbusCommand for LogoutCommand {
    fn module(&self) -> &'static str {
        "zwrt_web"
    }
    fn method(&self) -> &'static str {
        "web_logout"
    }
}

/// Get RSA public key certificate for encryption key exchange.
#[derive(Serialize, Default)]
pub struct GetCertificateCommand {}

impl UbusCommand for GetCertificateCommand {
    fn module(&self) -> &'static str {
        "zwrt_web"
    }
    fn method(&self) -> &'static str {
        "web_crt_get"
    }
}

/// Send encrypted AES key to the router.
#[derive(Serialize)]
pub struct SetEncryptionKeyCommand {
    pub web_enstr: String,
}

impl UbusCommand for SetEncryptionKeyCommand {
    fn module(&self) -> &'static str {
        "zwrt_web"
    }
    fn method(&self) -> &'static str {
        "web_http_enstr_set"
    }
}
