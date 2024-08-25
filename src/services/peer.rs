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
    utils::flatbuffer_utils::{create_auth_response_success, parse_gameplay_data},
};

use super::{
    env_settings::EnvSettings,
    game_server::GameServer,
    message_types::{Connect, Disconnect, PeerPlayerData},
};

#[derive(Debug)]
pub enum ClientData {
    JoinGameRequest {
        player_uuid: String,
        jwt_token: String,
    },
    BetRequest {
        bet_amount: u32,
    },
    Unknown,
}

pub struct Peer {
    // unique session id
    pub id: usize,

    pub heart_beat: Instant,

    // game server actor address
    pub game_server_addr: Addr<GameServer>,

    pub env_settings: web::Data<EnvSettings>,
}

impl Peer {
    pub fn new(game_server_addr: Addr<GameServer>, env_settings: web::Data<EnvSettings>) -> Self {
        Self {
            // id is re-assigned when connection is established
            id: 0,
            heart_beat: Instant::now(),
            game_server_addr,
            env_settings,
        }
    }
}

impl Actor for Peer {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let peer_addr = ctx.address();
        self.game_server_addr
            .send(Connect {
                peer_addr: peer_addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        act.id = res;
                    }
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .then(|_, act, _| {
                info!("peer actor connected! id: {:?}", act.id);
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        // notify game server
        self.game_server_addr.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl Handler<PeerPlayerData> for Peer {
    type Result = ();

    fn handle(&mut self, msg: PeerPlayerData, ctx: &mut Self::Context) -> Self::Result {
        todo!()
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
                        match UserAuthentication::validate_auth(
                            &player_uuid,
                            &jwt_token,
                            &self.env_settings,
                        ) {
                            Ok(_) => {
                                let success_data = create_auth_response_success();
                                info!(
                                    "token valid! sending response bytes of len {:?}",
                                    &success_data.len()
                                );
                                ctx.binary(success_data);
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
                        // self.game_server_addr.do_send(PeerPlayerPositionUpdate {
                        //     player_position: player_position,
                        //     player_id: self.id
                        // });
                    }
                    ClientData::BetRequest { bet_amount } => {
                        info!("bet request {:?}", bet_amount);
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
