use actix_web::dev::Payload;
use actix_web::error::ErrorUnauthorized;
use actix_web::http::header::HeaderValue;
use actix_web::{web, Error as ActixWebError, FromRequest, HttpRequest};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, errors::Error as JwtError, Algorithm, DecodingKey, TokenData, Validation,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use uuid::Uuid;

use crate::services::env_settings::EnvSettings;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserClaims {
    pub exp: usize,
    pub uuid: String,
}

impl UserClaims {
    pub fn new(user_jwt_expiration_minutes: i64, uuid: String) -> Self {
        let token_expiry_date =
            (Utc::now() + Duration::minutes(user_jwt_expiration_minutes)).timestamp() as usize;
        Self {
            exp: token_expiry_date,
            uuid: uuid,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuthentication {
    pub authentication_token: String,
    pub uuid: String,
}

impl UserAuthentication {
    pub fn validate_auth(
        uuid: &str,
        jwt_token: &str,
        env_settings: &EnvSettings,
    ) -> Result<UserAuthentication, ()> {
        let token_result: Result<TokenData<UserClaims>, JwtError> = decode::<UserClaims>(
            jwt_token,
            &DecodingKey::from_secret(env_settings.user_jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        );
        match token_result {
            Ok(token) => {
                let user_claims = token.claims;
                let user_uuid = user_claims.uuid;
                if user_uuid == uuid {
                    Ok(UserAuthentication {
                        authentication_token: jwt_token.to_owned(),
                        uuid: user_uuid,
                    })
                } else {
                    Err(())
                }
            }
            Err(_) => Err(()),
        }
    }

    pub fn create_guest_auth(env_settings: &EnvSettings) -> Result<UserAuthentication, ()> {
        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string();

        let claims = UserClaims::new(env_settings.user_jwt_expiration_minutes, uuid_str.clone());

        let jwt_token_result = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(env_settings.user_jwt_secret.as_ref()),
        );

        match jwt_token_result {
            Ok(jwt_token) => Ok(UserAuthentication {
                authentication_token: jwt_token,
                uuid: uuid_str,
            }),
            Err(_) => Err(()),
        }
    }
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
