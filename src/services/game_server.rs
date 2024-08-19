use std::sync::atomic::Ordering;

use actix::{Actor, Context, Handler};
use actix_web::web;
use log::info;
use rand::{rngs::ThreadRng, Rng};

use super::{
    game_stats::GameStats,
    message_types::{Connect, Disconnect},
};

#[derive(Debug)]
pub struct GameServer {
    rng: ThreadRng,
    game_stats: web::Data<GameStats>,
}

impl GameServer {
    pub fn new(game_stats: web::Data<GameStats>) -> GameServer {
        Self {
            rng: rand::thread_rng(),
            game_stats,
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
        info!("peer joined!");
        // register session with random id
        let id = self.rng.gen::<usize>();

        self.game_stats
            .players_online
            .fetch_add(1, Ordering::SeqCst);
        id
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
