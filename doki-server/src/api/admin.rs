use crate::services::archive;
use crate::services::auth::AdminUser;
use crate::services::s3::Downloader;
use crate::state::AppState;
use log::{log, Level};
use rocket::fs::TempFile;
use rocket::{Route, State};
use std::path::Path;
use std::rc::Rc;

pub struct Api;

impl Api {
    pub fn routes() -> Vec<Route> {
        routes![update, upload_tar_gz, upload_zip]
    }
}

#[post("/update")]
pub async fn update(_user: AdminUser, downloader: &State<Downloader>) -> Result<(), rocket::http::Status> {
    let result = downloader.clean_download().await.map_err(|e| {
        log!(Level::Error, "Failed to update documentation: {}", e);
        rocket::http::Status::InternalServerError
    });
    log!(Level::Info, "Documentation updated");
    result
}

/// Route that takes an archive file(tar) and extracts it to the local directory
#[post("/upload", data = "<file>", format = "application/gzip")]
pub async fn upload_tar_gz(_user: AdminUser, file: TempFile<'_>, state: &State<AppState>, downloader: &State<Downloader>) -> Result<(), rocket::http::Status> {
    handle_upload(file, state, downloader, "tar.gz", archive::unpack_tar_gz).await
}

/// Route that takes an archive file(zip) and extracts it to the local directory
#[post("/upload", data = "<file>", format = "application/zip", rank = 2)]
pub async fn upload_zip(_user: AdminUser, file: TempFile<'_>, state: &State<AppState>, downloader: &State<Downloader>) -> Result<(), rocket::http::Status> {
    handle_upload(file, state, downloader, "rar", archive::unpack_zip).await
}

async fn handle_upload(
    mut file: TempFile<'_>,
    state: &State<AppState>,
    downloader: &State<Downloader>,
    file_ext: &str,
    unpack_fn: fn(&Path, &Path) -> Result<(), anyhow::Error>,
) -> Result<(), rocket::http::Status> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let path = state.temp_dir.join(format!("temp_{}-{}.{}", file_ext, uuid, file_ext));
    file.persist_to(&path).await.map_err(|e| {
        log!(Level::Error, "Failed to persist file: {}", e);
        rocket::http::Status::InternalServerError
    })?;
    let backup_path = state.temp_dir.join(format!("backup-{}.tar.gz", uuid));
    let backup_path = Rc::from(backup_path.as_path());
    archive::pack_tar_gz(&state.local_dir, &backup_path, flate2::Compression::fast()).map_err(|e| {
        log!(Level::Error, "Failed to make backup: {}", e);
        rocket::http::Status::InternalServerError
    })?;
    downloader.clear_dir().map_err(|e| {
        log!(Level::Error, "Failed to clear local directory: {}", e);
        rocket::http::Status::InternalServerError
    })?;
    unpack_fn(&path, &state.local_dir).map_err(|e| {
        log!(Level::Error, "Failed to unpack archive: {}", e);
        archive::unpack_tar_gz(&backup_path, &state.local_dir).expect("Failed to restore backup");
        let backup_path = &backup_path.clone();
        let _ = std::fs::remove_file(&backup_path);
        rocket::http::Status::InternalServerError
    })?;
    let _ = std::fs::remove_file(backup_path);
    let _ = std::fs::remove_file(path);
    Ok(())
}