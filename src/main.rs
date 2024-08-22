mod generated;
mod routes;
mod services;

use dotenv::dotenv;
use actix::Actor;
use actix_web::{error, middleware, web, App, HttpResponse, HttpServer};
use generated::hello::get_num;
use log::info;
use routes::{
    auth::auth_login,
    create_ws::create_ws,
    stats::get_stats,
    utils::error_response::{AppError, AppErrorResponse},
};
use services::{env_settings::EnvSettings, game_server::GameServer, game_stats::GameStats};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    info!("answer to everything! {:?}", get_num());
    let env_settings = EnvSettings::new();

    let port = 8090;
    let game_stats = web::Data::new(GameStats::new());
    let game_server = GameServer::new(game_stats.clone()).start();

    info!("running server in port {:?}", 8090);
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
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
            .service(web::scope("/ws").service(create_ws))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
