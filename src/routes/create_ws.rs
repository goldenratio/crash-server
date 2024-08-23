use actix::Addr;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::services::{env_settings::EnvSettings, game_server::GameServer, peer::Peer};

#[get("/crash-game")]
pub async fn create_crash_game(
    req: HttpRequest,
    stream: web::Payload,
    env_settings: web::Data<EnvSettings>,
    game_server_addr: web::Data<Addr<GameServer>>,
) -> Result<HttpResponse, Error> {
    let game_server_addr_ref = game_server_addr.get_ref().clone();
    ws::start(Peer::new(game_server_addr_ref, env_settings), &req, stream)
}
