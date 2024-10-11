#[macro_use]
extern crate rocket;
mod services;
mod settings;

mod api;

use crate::services::auth::BasicAuthorizer;
use crate::services::s3::Downloader;
use crate::settings::settings::Settings;
use clap::{Arg, Command};
use rocket::fs::FileServer;
use rocket::Config;
use std::sync::Arc;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

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

    let config = Config {
        port: settings.server.port as u16,
        temp_dir: "/tmp".into(),
        ..Config::debug_default()
    };

    // get current directory
    let local_dir = std::env::current_dir().expect("Failed to get current directory").join("www/public");

    rocket::build()
        // .configure(config)
        .mount("/", FileServer::from(local_dir.clone()))
        .mount("/api/admin", routes![api::admin::update])
        .attach(Downloader::managed(settings.s3.bucket.to_string(), local_dir.clone().to_str().expect("Failed to get static directory").to_string()))
        .attach(BasicAuthorizer::managed(settings.auth.clone()))
        .attach(Settings::manage(settings.clone()))
    // .attach(Template::fairing())
}