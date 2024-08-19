use std::time::Instant;

use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler,
    Running, StreamHandler, WrapFuture,
};
use actix_web_actors::ws;
use log::info;

use super::{
    game_server::GameServer,
    message_types::{Connect, Disconnect, PeerPlayerData},
};

pub struct Peer {
    // unique session id
    pub id: usize,

    pub heart_beat: Instant,

    // game server actor address
    pub game_server_addr: Addr<GameServer>,
}

impl Peer {
    pub fn new(game_server_addr: Addr<GameServer>) -> Self {
        Self {
            // id is re-assigned when connection is established
            id: 0,
            heart_beat: Instant::now(),
            game_server_addr,
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
                todo!()
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
