use derive_more::Display;
use serde::Serialize;

use crate::routes::auth::LoginError;

#[derive(Serialize, Debug, Display)]
pub enum AppError {
    InvalidRequestPayload = 10001,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorResponse {
    pub error_code: u16,
    pub error_message: String,
}

impl From<AppError> for AppErrorResponse {
    fn from(value: AppError) -> AppErrorResponse {
        match value {
            AppError::InvalidRequestPayload => {
                return AppErrorResponse {
                    error_code: AppError::InvalidRequestPayload as u16,
                    error_message: "Invalid request payload".to_string(),
                };
            }
        }
    }
}

impl From<LoginError> for AppErrorResponse {
    fn from(value: LoginError) -> AppErrorResponse {
        match value {
            LoginError::GenericError => {
                return AppErrorResponse {
                    error_code: LoginError::GenericError as u16,
                    error_message: "Generic login error".to_string(),
                };
            }
            LoginError::InvalidEmailOrPassword => {
                return AppErrorResponse {
                    error_code: LoginError::InvalidEmailOrPassword as u16,
                    error_message: "Invalid email or password".to_string(),
                };
            }
        }
    }
}
