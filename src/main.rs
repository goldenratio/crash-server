mod generated;
mod routes;
mod services;
mod utils;

use actix::Actor;
use actix_cors::Cors;
use actix_web::{error, middleware, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use log::info;
use routes::{
    auth::auth_login,
    create_ws::create_crash_game,
    stats::get_stats,
    utils::error_response::{AppError, AppErrorResponse},
};
use services::{env_settings::EnvSettings, game_server::GameServer, game_stats::GameStats};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    let env_settings = EnvSettings::new();

    let port = 8090;
    let game_stats = web::Data::new(GameStats::new());
    let game_server = GameServer::new(game_stats.clone()).start();

    info!("running server in port {:?}", 8090);
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            // todo: during development, use feature config
            .wrap(Cors::permissive())
            .app_data(web::Data::new(env_settings.clone()))
            .app_data(web::Data::new(game_server.clone()))
            .app_data(game_stats.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(1024)
                    .error_handler(|err, _req| {
                        return error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest()
                                .json(AppErrorResponse::from(AppError::InvalidRequestPayload)),
                        )
                        .into();
                    }),
            )
            .service(web::scope("/api").service(get_stats).service(auth_login))
            .service(web::scope("/ws").service(create_crash_game))
    })
    .bind(("127.0.0.1", port))?
    .workers(2)
    .run()
    .await
}
