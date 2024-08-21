use actix::Addr;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::services::{game_server::GameServer, peer::Peer};

#[get("/game")]
pub async fn create_ws(
    req: HttpRequest,
    stream: web::Payload,
    game_server_addr: web::Data<Addr<GameServer>>,
) -> Result<HttpResponse, Error> {
    let game_server_addr_ref = game_server_addr.get_ref().clone();
    ws::start(Peer::new(game_server_addr_ref), &req, stream)
}
