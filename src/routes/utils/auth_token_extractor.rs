use actix_web::dev::Payload;
use actix_web::error::ErrorUnauthorized;
use actix_web::http::header::HeaderValue;
use actix_web::{web, Error as ActixWebError, FromRequest, HttpRequest};
use jsonwebtoken::{
    decode, errors::Error as JwtError, Algorithm, DecodingKey, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

use crate::services::env_settings::EnvSettings;

use super::user_claims::UserClaims;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuthentication {
    pub authentication_token: String,
    pub uuid: String,
}

impl FromRequest for UserAuthentication {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        let env_settings = &req.app_data::<web::Data<EnvSettings>>().unwrap();

        let authorization_header_option: Option<&HeaderValue> =
            req.headers().get(actix_web::http::header::AUTHORIZATION);
        // No Header was sent
        if authorization_header_option.is_none() {
            return ready(Err(ErrorUnauthorized("No authentication token sent!")));
        }

        let authentication_token: String = authorization_header_option
            .unwrap()
            .to_str()
            .unwrap_or("")
            .to_string();
        // Couldn't convert Header::Authorization to String
        if authentication_token.is_empty() {
            return ready(Err(ErrorUnauthorized("Invalid authentication token sent!")));
        }
        let client_auth_token = authentication_token[6..authentication_token.len()].trim();

        let token_result: Result<TokenData<UserClaims>, JwtError> = decode::<UserClaims>(
            client_auth_token,
            &DecodingKey::from_secret(env_settings.user_jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        );
        match token_result {
            Ok(token) => {
                let user_claims = token.claims;
                ready(Ok(UserAuthentication {
                    authentication_token,
                    uuid: user_claims.uuid,
                }))
            }
            Err(_) => {
                // log::error!("token_result Error: {:?}", e);
                ready(Err(ErrorUnauthorized("Invalid authentication token sent!")))
            }
        }
    }
}
