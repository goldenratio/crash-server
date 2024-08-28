use std::{collections::HashMap, sync::atomic::Ordering};

use actix::{Actor, Context, Handler, Recipient};
use actix_web::web;
use log::info;
use rand::{rngs::ThreadRng, Rng};

use super::{
    crash_game::CrashGame,
    game_stats::GameStats,
    message_types::{Connect, Disconnect, PeerPlayerData, PlayerJoined},
};

#[derive(Debug)]
pub struct GameServer {
    peer_addr_map: HashMap<String, Recipient<PeerPlayerData>>,
    rng: ThreadRng,
    game_stats: web::Data<GameStats>,
    crash_game: CrashGame,
}

impl GameServer {
    pub fn new(game_stats: web::Data<GameStats>) -> GameServer {
        Self {
            peer_addr_map: Default::default(),
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
        self.game_stats
            .players_online
            .fetch_sub(1, Ordering::SeqCst);
    }
}

impl Handler<PlayerJoined> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: PlayerJoined, _: &mut Self::Context) -> Self::Result {
        info!("peer joined the game! {:?}", msg.uuid);
        self.peer_addr_map.insert(msg.uuid, msg.peer_addr);
        self.game_stats
            .players_online
            .fetch_add(1, Ordering::SeqCst);

        // todo: notify other players
        self.crash_game.start();
    }
}
