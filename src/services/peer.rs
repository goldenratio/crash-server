use std::time::Instant;

use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler,
    Running, StreamHandler, WrapFuture,
};
use actix_web::web;
use actix_web_actors::ws::{self, CloseReason};
use log::info;

use crate::{
    routes::utils::auth_token_extractor::UserAuthentication,
    services::message_types::{BetRequest, CrashOutRequest, PlayerJoined},
    utils::flatbuffer_utils::{
        create_betting_timer_started_response, create_betting_timer_update_response,
        create_crash_out_response, create_game_finished_response, create_game_started_response,
        create_game_update_response, create_join_game_response_success,
        create_remote_player_crash_out_response, create_remote_player_joined_response,
        create_remote_player_left_response, parse_gameplay_data,
    },
};

use super::{
    env_settings::EnvSettings,
    game_server::GameServer,
    message_types::{Connect, Disconnect, GameEvent},
};

#[derive(Debug)]
pub enum ClientData {
    JoinGameRequest {
        player_uuid: String,
        jwt_token: String,
    },
    BetRequest {
        /// in cents
        bet_amount: u64,
    },
    CrashOutRequest {},
    Unknown,
}

pub struct Peer {
    // unique session id
    pub session_id: usize,

    pub heart_beat: Instant,

    // game server actor address
    pub game_server_addr: Addr<GameServer>,

    pub env_settings: web::Data<EnvSettings>,
}

impl Peer {
    pub fn new(game_server_addr: Addr<GameServer>, env_settings: web::Data<EnvSettings>) -> Self {
        Self {
            // session_id is re-assigned when connection is established
            session_id: 0,
            heart_beat: Instant::now(),
            game_server_addr,
            env_settings,
        }
    }
}

impl Actor for Peer {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.game_server_addr
            .send(Connect {})
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        act.session_id = res;
                    }
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .then(|_, act, _| {
                info!("peer actor connected! session_id: {:?}", act.session_id);
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        // notify game server
        self.game_server_addr.do_send(Disconnect {
            session_id: self.session_id,
        });
        Running::Stop
    }
}

impl Handler<GameEvent> for Peer {
    type Result = ();

    fn handle(&mut self, msg: GameEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            GameEvent::PlayerJoinedResponse {
                game_state,
                betting_time_left_ms,
                multiplier,
                round_time_elapsed_ms,
                display_name,
            } => {
                let response_data = create_join_game_response_success(
                    game_state,
                    betting_time_left_ms,
                    multiplier,
                    round_time_elapsed_ms,
                    display_name,
                );
                ctx.binary(response_data);
            }
            GameEvent::BettingTimerStarted {
                betting_time_left_ms,
            } => {
                let response_data = create_betting_timer_started_response(betting_time_left_ms);
                ctx.binary(response_data);
            }
            GameEvent::BettingTimerUpdate {
                betting_time_left_ms,
            } => {
                let response_data = create_betting_timer_update_response(betting_time_left_ms);
                ctx.binary(response_data);
            }
            GameEvent::GameStarted {} => {
                let response_data = create_game_started_response();
                ctx.binary(response_data);
            }
            GameEvent::GameFinished {} => {
                let response_data = create_game_finished_response();
                ctx.binary(response_data);
            }
            GameEvent::GameRoundUpdate { multiplier } => {
                let response_data = create_game_update_response(multiplier);
                ctx.binary(response_data);
            }
            GameEvent::CrashOutResponse {
                win_amount,
                multiplier,
            } => {
                let response_data = create_crash_out_response(win_amount, multiplier);
                ctx.binary(response_data);
            }
            GameEvent::RemotePlayerJoined { display_name } => {
                let response_data = create_remote_player_joined_response(display_name);
                ctx.binary(response_data);
            }
            GameEvent::RemotePlayerLeft { display_name } => {
                let response_data = create_remote_player_left_response(display_name);
                ctx.binary(response_data);
            }
            GameEvent::RemotePlayerCrashOut {
                display_name,
                win_amount,
            } => {
                let response_data =
                    create_remote_player_crash_out_response(display_name, win_amount);
                ctx.binary(response_data);
            }
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Peer {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match item {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Text(text) => {
                info!("received from client (text), {:?}", text);
            }
            ws::Message::Binary(bytes) => {
                info!("received from client (bytes) {:?}", bytes.len());
                let gameplay_data = parse_gameplay_data(&bytes);
                info!("gameplay_data: {:?}", &gameplay_data);
                match gameplay_data {
                    ClientData::JoinGameRequest {
                        jwt_token,
                        player_uuid,
                    } => {
                        // todo: check for already logged in
                        match UserAuthentication::validate_auth(
                            &player_uuid,
                            &jwt_token,
                            &self.env_settings,
                        ) {
                            Ok(_) => {
                                let peer_addr = ctx.address();
                                self.game_server_addr.do_send(PlayerJoined {
                                    session_id: self.session_id,
                                    uuid: player_uuid.clone(),
                                    peer_addr: peer_addr.recipient(),
                                });
                            }
                            Err(_) => {
                                ctx.close(Option::from(CloseReason {
                                    code: ws::CloseCode::Invalid,
                                    description: Option::from(
                                        "Invalid authentication token sent!".to_owned(),
                                    ),
                                }));
                                ctx.stop();
                            }
                        };
                    }
                    ClientData::BetRequest { bet_amount } => {
                        // info!("bet request {:?} {:?}", bet_amount, self.session_id);
                        self.game_server_addr.do_send(BetRequest {
                            session_id: self.session_id,
                            bet_amount: bet_amount,
                        });
                    }
                    ClientData::CrashOutRequest {} => {
                        // info!("crash out {:?}", self.session_id);
                        self.game_server_addr.do_send(CrashOutRequest {
                            session_id: self.session_id,
                        });
                    }
                    ClientData::Unknown => {}
                }
            }
            ws::Message::Ping(msg) => {
                self.heart_beat = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.heart_beat = Instant::now();
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => {}
        }
    }
}
