use std::{collections::HashMap, sync::atomic::Ordering};

use actix::{Actor, AsyncContext, Context, Handler, Recipient};
use actix_web::web;
use log::{info, warn};
use rand::{rngs::ThreadRng, Rng};

use crate::services::crash_game::GameState;

use super::{
    crash_game::CrashGame,
    game_stats::GameStats,
    message_types::{
        BetRequest, BettingTimerUpdate, Connect, CrashOutRequest, Disconnect, GameEvent,
        GameRoundUpdate, PlayerJoined,
    },
};

#[derive(Debug)]
pub struct GameServer {
    peer_addr_map: HashMap<String, Recipient<GameEvent>>,
    peer_session_uuid_map: HashMap<usize, String>,
    rng: ThreadRng,
    game_stats: web::Data<GameStats>,
    crash_game: CrashGame,
}

impl GameServer {
    pub fn new(game_stats: web::Data<GameStats>) -> GameServer {
        Self {
            peer_addr_map: Default::default(),
            peer_session_uuid_map: Default::default(),
            rng: rand::thread_rng(),
            game_stats,
            crash_game: CrashGame::new(),
        }
    }
}

impl Actor for GameServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.crash_game.set_game_server_addr(addr);
    }
}

impl Handler<Connect> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        info!("peer connected!");
        // register session with random id
        let session_id = self.rng.gen::<usize>();
        session_id
    }
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        info!("peer disconnected!");
        // remove peer address
        if let Some(uuid) = self.peer_session_uuid_map.get(&msg.session_id) {
            self.peer_addr_map.remove(uuid);
        }
        self.peer_session_uuid_map.remove(&msg.session_id);

        self.game_stats
            .players_online
            .fetch_sub(1, Ordering::SeqCst);
    }
}

impl Handler<PlayerJoined> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: PlayerJoined, _: &mut Self::Context) -> Self::Result {
        info!("peer joined the game! {:?}", msg.uuid);

        self.peer_addr_map
            .insert(msg.uuid.clone(), msg.peer_addr.clone());

        self.peer_session_uuid_map
            .insert(msg.session_id, msg.uuid.clone());

        self.game_stats
            .players_online
            .fetch_add(1, Ordering::SeqCst);

        let game_data = self.crash_game.get_game_data();
        // send player current game state
        msg.peer_addr.do_send(GameEvent::PlayerJoinedResponse {
            betting_time_left_ms: game_data.betting_time_left_ms,
            game_state: game_data.game_state.into(),
            multiplier: game_data.multiplier,
            round_time_elapsed_ms: game_data.round_time_elapsed_ms,
        });

        // todo: notify other players

        match game_data.game_state {
            GameState::IDLE => {
                // start betting timer, If game is idle
                self.crash_game.start_betting_timer();
            }
            _ => {
                // do nothing
            }
        };
    }
}

impl Handler<BetRequest> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: BetRequest, _: &mut Self::Context) -> Self::Result {
        if let Some(uuid) = self.peer_session_uuid_map.get(&msg.session_id) {
            info!("bets placed! {:?} {:?}", uuid, msg.bet_amount);
            self.crash_game.place_bets(uuid, msg.bet_amount)
        } else {
            warn!("BetRequest: unknown session id {:?}", msg.session_id);
        }
    }
}

impl Handler<CrashOutRequest> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: CrashOutRequest, _: &mut Self::Context) -> Self::Result {
        if let Some(uuid) = self.peer_session_uuid_map.get(&msg.session_id) {
            info!("player crashed out! {:?}", uuid);
            self.crash_game.crash_out(uuid);
        } else {
            warn!("CrashOutRequest: unknown session id {:?}", msg.session_id);
        }
    }
}

// --------------------------------
// messages from CrashGame
// --------------------------------

impl Handler<BettingTimerUpdate> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: BettingTimerUpdate, _: &mut Self::Context) -> Self::Result {
        info!("time left: {:?}", msg.betting_time_left_ms);
        for (_, client_addr) in &self.peer_addr_map {
            client_addr.do_send(GameEvent::BettingTimerUpdate {
                betting_time_left_ms: msg.betting_time_left_ms,
            });
        }
    }
}

impl Handler<GameRoundUpdate> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: GameRoundUpdate, _: &mut Self::Context) -> Self::Result {
        info!("multiplier: {:?}", msg.multiplier);
        for (_, client_addr) in &self.peer_addr_map {
            client_addr.do_send(GameEvent::GameRoundUpdate {
                multiplier: msg.multiplier,
            });
        }
    }
}
