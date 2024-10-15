use crate::settings::settings::Auth;
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::request::{FromRequest, Outcome, Request};

pub struct BasicAuthorizer {
    username: String,
    password: String,
}

impl BasicAuthorizer {
    fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    fn authorize(&self, username: &str, password: &str) -> Result<AdminUser, String> {
        if self.username == username && self.password == password {
            Ok(AdminUser {
                username: self.username.clone(),
                password: self.password.clone(),
            })
        } else {
            Err("Invalid credentials".to_string())
        }
    }

    pub fn managed(settings: Auth) -> rocket::fairing::AdHoc {
        rocket::fairing::AdHoc::on_ignite("Basic Authorizer", move |rocket| async move {
            // let username = rocket.figment().get_str("admin.username").unwrap();
            // let password = rocket.figment().get_str("admin.password").unwrap();
            let authorizer = BasicAuthorizer::new(settings.username.to_string(), settings.password.to_string());
            rocket.manage(authorizer)
        })
    }
}

pub struct AdminUser {
    pub username: String,
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(user) = headers::extract_auth_from_request(request) {
            request
                .rocket()
                .state::<BasicAuthorizer>()
                .map(|authorizer| {
                    authorizer
                        .authorize(user.username.as_str(), user.password.as_str())
                        .expect("provided credentials are valid")
                })
                .or_forward(Status::InternalServerError)
        } else {
            Outcome::Error((Status::Unauthorized, "Invalid data in 'authorization' header".to_string()))
        }
    }
}

mod headers {
    use crate::services::auth::AdminUser;
    use rocket::Request;

    pub fn extract_auth_from_request(request: &Request) -> Option<AdminUser> {
        request
            .headers()
            .get_one("authorization")
            .and_then(extract_payload_from_header)
            .and_then(|token| decode_token(token))
    }

    fn extract_payload_from_header(header: &str) -> Option<&str> {
        let prefix = "Basic ";
        if header.starts_with(prefix) {
            Some(&header[prefix.len()..])
        } else {
            None
        }
    }

    fn decode_token(token: &str) -> Option<AdminUser> {
        use base64::{engine::general_purpose, Engine as _};

        let bytes = general_purpose::STANDARD.decode(token).unwrap();
        let decoded = String::from_utf8(bytes).ok()?;
        let parts: Vec<&str> = decoded.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let username = parts[0].to_string();
        let password = parts[1].to_string();
        Some(AdminUser { username, password })
    }
}
