use actix_web::{http::StatusCode, post, web, HttpResponse, Responder, ResponseError};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::utils::error_response::AppErrorResponse;

#[derive(Serialize, Debug, Display)]
pub enum LoginError {
    InvalidEmailOrPassword = 10011,
}

#[derive(Deserialize, Debug)]
pub enum PlayMode {
    FUN = 0,
    REAL,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LoginRequestData {
    email: String,
    password: String,
    play_mode: PlayMode,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginSuccessResponse {
    jwt_token: String,
    uuid: String,
    display_name: String,
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::InvalidEmailOrPassword => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        match self {
            LoginError::InvalidEmailOrPassword => HttpResponse::build(status)
                .json(AppErrorResponse::from(LoginError::InvalidEmailOrPassword)),
        }
    }
}

#[post("/login")]
async fn auth_login(param_obj: web::Json<LoginRequestData>) -> Result<impl Responder, LoginError> {
    let payload = param_obj.into_inner();
    log::trace!("/auth {:?}", payload);

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();

    let response_data = LoginSuccessResponse {
        jwt_token: "2121".to_string(),
        uuid: uuid_str,
        display_name: "foo".to_string(),
    };

    return Ok(web::Json(response_data));
}
