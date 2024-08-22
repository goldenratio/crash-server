use actix_web::{http::StatusCode, post, web, HttpResponse, Responder, ResponseError};
use derive_more::Display;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{routes::utils::user_claims::UserClaims, services::env_settings::EnvSettings};

use super::utils::error_response::AppErrorResponse;

#[derive(Serialize, Debug, Display)]
pub enum LoginError {
    GenericError = 10011,
    InvalidEmailOrPassword,
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
            LoginError::GenericError => StatusCode::INTERNAL_SERVER_ERROR,
            LoginError::InvalidEmailOrPassword => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        match self {
            LoginError::GenericError => {
                HttpResponse::build(status).json(AppErrorResponse::from(LoginError::GenericError))
            }
            LoginError::InvalidEmailOrPassword => HttpResponse::build(status)
                .json(AppErrorResponse::from(LoginError::InvalidEmailOrPassword)),
        }
    }
}

#[post("/login")]
async fn auth_login(
    param_obj: web::Json<LoginRequestData>,
    env_settings: web::Data<EnvSettings>,
) -> Result<impl Responder, LoginError> {
    let payload = param_obj.into_inner();
    log::info!("/auth {:?}", payload);

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();

    let claims = UserClaims::new(env_settings.user_jwt_expiration_minutes, uuid_str.clone());

    let jwt_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(env_settings.user_jwt_secret.as_ref()),
    )
    .map_err(|_| LoginError::GenericError)?;

    let response_data = LoginSuccessResponse {
        jwt_token,
        uuid: uuid_str,
        display_name: "foo".to_string(),
    };

    Ok(web::Json(response_data))
}
