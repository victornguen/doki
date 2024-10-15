use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Server {
    /// Listening address, must be 0.0.0.0 if you want to listen all incoming connections
    pub host: String,
    /// Listening port
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
    /// URL of the S3 endpoint
    pub endpoint: String,
    /// Name of the bucket
    pub bucket: String,
    /// Use path style for S3 requests
    pub force_path_style: bool,
    /// Whether to load the documentation on start
    pub load_on_start: bool,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct FS {
    /// Local directory where the documentation is stored
    pub local_dir: String,
    /// Temporary directory for storing files
    pub temp_dir: String,
    /// Directory where the static files are stored
    pub statics: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Settings {
    pub server: Server,
    pub logging: Logging,
    pub auth: Auth,
    pub s3: S3,
    pub fs: FS,
}

impl Settings {
    pub fn new(location: &str, env_prefix: &str) -> anyhow::Result<Self> {
        let mut builder = Config::builder();

        if Path::new(location).exists() {
            builder = builder.add_source(File::with_name(location));
        }

        builder = builder.add_source(
            Environment::with_prefix(env_prefix)
                .separator("__")
                .prefix_separator("__"),
        );

        let settings = builder.build()?.try_deserialize()?;
        Ok(settings)
    }
}
