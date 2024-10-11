use crate::services::auth::AdminUser;
use crate::services::s3::Downloader;
use crate::settings::settings::Settings;
use log::{log, Level};
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};

#[post("/update")]
pub async fn update(user: AdminUser, downloader: &State<Downloader>) -> Result<(), rocket::http::Status> {
    let result = downloader.clean_download().await.map_err(|e| {
        log!(Level::Error, "Failed to update documentation: {}", e);
        rocket::http::Status::InternalServerError
    });
    log!(Level::Info, "Documentation updated");
    result
}


