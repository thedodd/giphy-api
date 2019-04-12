use base64;
use envy;
use serde_derive::Deserialize;

/// Config defines this API's runtime configuration.
#[derive(Deserialize)]
pub struct Config {
    pub port: u16,

    // Database Config //
    pub backend_connection_string: String,
    pub backend_database: String,

    // Auth Config //
    pub idp_private_key: String,
    pub idp_public_key: String,
    #[serde(skip)]
    pub raw_idp_private_key: String,
    #[serde(skip)]
    pub raw_idp_public_key: String,

    // Giphy API Config //
    pub giphy_api_key: String,
}

impl Config {
    /// Create a new config instance populated from the runtime environment.
    pub fn new() -> Config {
        // Deserialize config from environment.
        let mut config = match envy::from_env::<Config>() {
            Ok(config) => config,
            Err(err) => panic!("{:#?}", err),
        };

        // Decode public & private keys.
        config.raw_idp_private_key = String::from_utf8(
            base64::decode(&config.idp_private_key).expect("Expected private key to be base64 encoded.")
        ).expect("Expected private key to be valid UTF8.");
        config.raw_idp_public_key = String::from_utf8(
            base64::decode(&config.idp_public_key).expect("Expected public key to be base64 encoded.")
        ).expect("Expected public key to be valid UTF8.");

        // All is good. No panic. Return config instance.
        config
    }
}
