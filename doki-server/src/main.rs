extern crate log;
#[macro_use]
extern crate rocket;
mod services;
mod settings;

mod api;
mod state;

use crate::services::auth::BasicAuthorizer;
use crate::services::s3::Downloader;
use crate::settings::settings::Settings;
use clap::{Arg, Command};
use log::LevelFilter;
use rocket::fs::FileServer;
use rocket::Config;
use std::path::Path;
use std::str::FromStr;

#[launch]
async fn rocket() -> _ {
    let command = Command::new("Documentation hosting")
        .version("1.0")
        .about("Documentation hosting service, that downloads documentation from S3 and serves it")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .help("Configuration file location")
            .default_value("config.yaml"));

    let matches = command.get_matches();
    let config_location = matches.get_one::<String>("config").unwrap_or(&"".to_string()).to_string();
    let settings = Settings::new(&config_location, "DOKI").expect("Failed to load settings");

    log::set_max_level(LevelFilter::from_str(settings.logging.log_level.as_str()).expect("Failed to set log level"));

    let config = Config {
        port: settings.server.port as u16,
        temp_dir: settings.fs.temp_dir.clone().into(),
        ..Config::default()
    };

    let local_dir = settings.fs.local_dir.clone();
    let temp_dir = settings.fs.temp_dir.clone();
    let statics = Path::new(local_dir.as_str()).join(settings.fs.statics.as_str());


    rocket::build()
        .configure(config)
        .mount("/", FileServer::from(statics.clone()))
        .mount("/api/admin", api::admin::Api::routes())
        .attach(Downloader::managed(settings.s3.bucket.to_string(), statics.clone().to_str().expect("Failed to convert path to string").to_string()))
        .attach(BasicAuthorizer::managed(settings.auth.clone()))
        .attach(state::AppState::managed(Path::new(&local_dir).into(), Path::new(&temp_dir).into()))
        .attach(Settings::manage(settings.clone()))
}