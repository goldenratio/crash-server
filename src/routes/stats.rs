use std::sync::atomic::Ordering;

use actix_web::{get, web, Responder};
use serde::Serialize;

use crate::services::game_stats::GameStats;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatsResponseData {
    players_online: usize,
}

#[get("/stats")]
pub async fn get_stats(game_stats: web::Data<GameStats>) -> impl Responder {
    let players_online = game_stats.players_online.load(Ordering::SeqCst);
    let response_data = StatsResponseData { players_online };
    return web::Json(response_data);
}
