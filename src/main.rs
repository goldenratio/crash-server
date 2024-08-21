mod routes;
mod services;
mod generated;

use actix::Actor;
use actix_web::{middleware, web, App, HttpServer};
use generated::hello::get_num;
use log::info;
use routes::{create_ws::create_ws, stats::get_stats};
use services::{game_server::GameServer, game_stats::GameStats};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("answer to everything! {:?}", get_num());

    let port = 8090;
    let game_stats = web::Data::new(GameStats::new());
    let game_server = GameServer::new(game_stats.clone()).start();

    info!("running server in port {:?}", 8090);
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(game_server.clone()))
            .app_data(game_stats.clone())
            .service(web::scope("/api").service(get_stats))
            .service(web::scope("/ws").service(create_ws))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
