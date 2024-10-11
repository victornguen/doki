use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Server {
    pub host: String,
    pub port: i32,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Logging {
    pub log_level: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct S3 {
    pub bucket: String,
    pub local_dir: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Settings {
    pub server: Server,
    pub logging: Logging,
    pub auth: Auth,
    pub s3: S3,
}

impl Settings {
    pub fn new(location: &str, env_prefix: &str) -> anyhow::Result<Self> {
        let s = Config::builder()
            .add_source(File::with_name(location))
            .add_source(Environment::with_prefix(env_prefix).separator("__").prefix_separator("__"))
            .build()?;
        let settings = s.try_deserialize()?;
        Ok(settings)
    }

    pub fn manage(settings: Settings) -> rocket::fairing::AdHoc {
        rocket::fairing::AdHoc::on_ignite("Settings", move |rocket| async move {
            rocket.manage(settings)
        })
    }
}
