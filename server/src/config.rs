use envy;
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::Deserialize;

/// A model used for deserializing raw values out of the environment.
#[derive(Deserialize)]
struct EnvConfig {
    pub rust_log: String,
    pub port: u16,
    pub database_url: String,
    pub idp_private_key: String,
    pub idp_public_key: String,
    pub giphy_api_key: String,
}

/// Config defines this API's runtime configuration.
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey<'static>,
    pub giphy_api_key: String,
}

impl Config {
    /// Create a new config instance populated from the runtime environment.
    pub fn new() -> Config {
        // Deserialize config from environment.
        let config = match envy::from_env::<EnvConfig>() {
            Ok(config) => config,
            Err(err) => panic!("{:#?}", err),
        };
        // Decode public & private keys.
        let decoded_private_key = base64::decode(&config.idp_private_key).expect("Expected private key to be base64 encoded.");
        let encoding_key = EncodingKey::from_rsa_pem(&decoded_private_key).expect("Failed to parse IDP private key.");
        let decoded_public_key: Vec<u8> = base64::decode(&config.idp_public_key).expect("Expected public key to be base64 encoded.");
        let decoding_key = DecodingKey::from_rsa_pem(&decoded_public_key).expect("Failed to parse IDP public key.").into_static();
        // All is good. No panic. Return config instance.
        Self{
            port: config.port,
            database_url: config.database_url,
            encoding_key: encoding_key,
            decoding_key: decoding_key,
            giphy_api_key: config.giphy_api_key,
        }
    }
}
