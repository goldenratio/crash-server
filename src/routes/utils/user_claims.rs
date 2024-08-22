use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

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
