use std::env;

#[derive(Debug, Clone)]
pub struct EnvSettings {
    pub user_jwt_secret: String,
    pub user_jwt_expiration_minutes: i64,
    pub server_port: u16,
    pub betting_time_duration: u32,
    pub house_edge_pct: f32,
}

impl EnvSettings {
    pub fn new() -> Self {
        Self {
            user_jwt_expiration_minutes: env::var("JWT_EXPIRATION_MINUTES")
                .expect("JWT_EXPIRATION_MINUTES in .env file is missing")
                .parse::<i64>()
                .expect("JWT_EXPIRATION_MINUTES must be a valid i64 number"),
            user_jwt_secret: env::var("USER_JWT_SECRET")
                .expect("USER_JWT_SECRET in .env file is missing"),
            server_port: env::var("PORT")
                .expect("PORT in .env file is missing")
                .parse::<u16>()
                .expect("PORT must be a valid U16 number"),
            betting_time_duration: env::var("BETTING_TIME_DURATION")
                .expect("BETTING_TIME_DURATION in .env file is missing")
                .parse::<u32>()
                .expect("BETTING_TIME_DURATION must be a valid u16 number"),
            house_edge_pct: env::var("HOUSE_EDGE_PERCENT")
                .expect("HOUSE_EDGE_PERCENT in .env file is missing")
                .parse::<f32>()
                .expect("HOUSE_EDGE_PERCENT must be a valid f32 number"),
        }
    }
}
