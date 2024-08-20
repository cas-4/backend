use config::ConfigError;
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configuration {
    pub rust_log: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub allowed_host: String,
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        let builder = config::Config::builder().add_source(config::Environment::default());

        builder.build()?.try_deserialize()
    }
}

lazy_static! {
    pub static ref CONFIG: Configuration = Configuration::new().expect("Config can be loaded");
}
