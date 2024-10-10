use crate::services::auth::AdminUser;
use crate::services::s3::Downloader;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};

#[post("/update")]
pub fn update(user: AdminUser, downloader: &State<Downloader>) {
    println!("Auth data={}:{}", user.username, user.password);
}


