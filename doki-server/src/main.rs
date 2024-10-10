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
// use rocket_dyn_templates::Template;

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

    let downloader = services::s3::Downloader::new().await;
    println!("Connected to S3");

    println!("Downloading directory");

    // get current directory
    let local_dir = std::env::current_dir().expect("Failed to get current directory");

    downloader.download_directory("aggapi-documentation", local_dir.join("static").to_str().unwrap()).await.expect("Failed to download directory");

    rocket::build()
        // .configure(config)
        .mount("/", FileServer::from(local_dir.join("static")))
        .mount("/api/admin", routes![api::admin::update])
        .attach(Downloader::manage(downloader))
        .attach(BasicAuthorizer::managed(settings.auth))
    // .attach(Template::fairing())
}