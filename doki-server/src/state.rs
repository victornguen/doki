use std::path::Path;

pub struct AppState {
    pub local_dir: Box<Path>,
    pub temp_dir: Box<Path>,
}

impl AppState {
    pub fn new(local_dir: &Path, temp_dir: &Path) -> Self {
        let local_dir = Box::from(local_dir);
        let temp_dir = Box::from(temp_dir);
        AppState { local_dir, temp_dir }
    }

    pub fn managed(local_dir: Box<Path>, temp_dir: Box<Path>) -> rocket::fairing::AdHoc {
        rocket::fairing::AdHoc::on_ignite("AppState", move |rocket| async move {
            rocket.manage(AppState::new(&local_dir, &temp_dir))
        })
    }
}