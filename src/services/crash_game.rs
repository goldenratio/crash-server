use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use actix::{spawn, Addr};
use actix_web::rt::time;
use log::{info, warn};
use map_range::MapRange;

use crate::services::message_types::{BettingTimerUpdate, GameRoundUpdate};

use super::{
    crash_game_math::{sha256, CrashGameMath},
    game_server::GameServer,
    message_types::{BettingTimerStarted, GameError, GameFinished, GameStarted},
};

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    Idle,
    BettingInProgress,
    GameInProgress,
}

impl Into<u8> for GameState {
    fn into(self) -> u8 {
        match self {
            GameState::Idle => 0,
            GameState::BettingInProgress => 1,
            GameState::GameInProgress => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub game_state: GameState,
    pub multiplier: u32,
    /// in milliseconds
    pub betting_time_left_ms: u32,
    /// in milliseconds
    pub round_time_elapsed_ms: u32,
}

#[derive(Debug, Clone)]
pub struct CrashGame {
    is_betting_in_progress: Arc<AtomicBool>,
    is_game_round_in_progress: Arc<AtomicBool>,
    /// in seconds
    betting_time_left: Arc<AtomicU32>,
    /// in seconds
    round_time_elapsed: Arc<AtomicU32>,
    current_multiplier: Arc<AtomicU32>,
    game_server_addr: Option<Addr<GameServer>>,
    max_betting_time_duration: u32,
    server_seed: String,
    next_round_server_seed: String,
    client_seed: Arc<Mutex<String>>,
    house_edge_pct: f32,
    round_id: u32,
}

struct RoundResult {
    multiplier: u32,
    /// in seconds
    animation_duration: u32,
}

impl CrashGame {
    pub fn new(betting_time_duration: u32, house_edge_pct: f32) -> Self {
        Self {
            is_betting_in_progress: Arc::new(AtomicBool::new(false)),
            is_game_round_in_progress: Arc::new(AtomicBool::new(false)),
            round_id: 32314,
            betting_time_left: Arc::new(AtomicU32::new(0)),
            round_time_elapsed: Arc::new(AtomicU32::new(0)),
            current_multiplier: Arc::new(AtomicU32::new(0)),
            game_server_addr: None,
            max_betting_time_duration: betting_time_duration,
            server_seed: Default::default(),
            next_round_server_seed: CrashGameMath::generate_seed(),
            client_seed: Default::default(),
            house_edge_pct: house_edge_pct,
        }
    }

    pub fn set_game_server_addr(&mut self, addr: Addr<GameServer>) {
        self.game_server_addr = Option::from(addr);
    }

    pub fn start_betting_timer(&mut self) {
        let valid = match self.get_game_state() {
            GameState::Idle => true,
            _ => false,
        };

        if !valid {
            // game is not idle, can't start new round
            info!("betting timer is already running!");
            return;
        }

        self.reset_game_data();
        self.set_betting_in_progress(true);

        self.round_id += 1;
        self.server_seed = self.next_round_server_seed.clone();
        self.next_round_server_seed = CrashGameMath::generate_seed();

        let game = Arc::new(self.clone());

        let mut interval = time::interval(Duration::from_secs(1));
        let mut time_left = self.max_betting_time_duration;

        game.game_server_addr
            .as_ref()
            .unwrap()
            .do_send(BettingTimerStarted {
                betting_time_left_ms: time_left * 1000,
                round_id: game.round_id,
                server_seed_hash: sha256(&game.server_seed),
                next_round_server_seed_hash: sha256(&game.next_round_server_seed),
            });

        spawn(async move {
            loop {
                if time_left <= 0 {
                    info!("betting timer is over, no more bets!");
                    game.on_betting_timer_finished();

                    // todo: only start game, if at-least 3 players has placed bets

                    // todo: get this from actual client
                    let mut client_seed = game.client_seed.lock().unwrap();
                    *client_seed = CrashGameMath::generate_seed();
                    drop(client_seed);

                    game.start_game();

                    return;
                }
                interval.tick().await;

                time_left = game.betting_time_left.fetch_sub(1, Ordering::SeqCst);

                game.game_server_addr
                    .as_ref()
                    .unwrap()
                    .do_send(BettingTimerUpdate {
                        betting_time_left_ms: time_left * 1000,
                    });
            }
        });
    }

    fn start_game(&self) {
        let valid = match self.get_game_state() {
            GameState::GameInProgress => true,
            _ => false,
        };

        if !valid {
            info!("game is in-progress already!");
            return;
        }

        if let Some(round_result) = self.get_round_result() {
            let mut current_second: u32 = 0;
            let game = Arc::new(self.clone()); // or self.clone()

            game.game_server_addr
                .as_ref()
                .unwrap()
                .do_send(GameStarted {});

            let mut interval = time::interval(Duration::from_secs(1));
            spawn(async move {
                loop {
                    if current_second >= round_result.animation_duration {
                        // send updates to peers
                        game.game_server_addr
                            .as_ref()
                            .unwrap()
                            .do_send(GameFinished {});
                        game.on_game_finished();
                        return;
                    }
                    interval.tick().await;
                    current_second += 1;

                    let current_multiplier = current_second.map_range(
                        0..round_result.animation_duration,
                        0..round_result.multiplier,
                    );

                    // info!("current_multiplier {:?}", current_multiplier);
                    game.current_multiplier
                        .store(current_multiplier, Ordering::SeqCst);
                    game.round_time_elapsed
                        .store(current_second, Ordering::SeqCst);

                    // send updates to peers
                    game.game_server_addr
                        .as_ref()
                        .unwrap()
                        .do_send(GameRoundUpdate {
                            multiplier: current_multiplier,
                        });
                }
            });
        } else {
            warn!("error in round result!");
            // error
            self.game_server_addr
                .as_ref()
                .unwrap()
                .do_send(GameError {});
            self.game_server_addr
                .as_ref()
                .unwrap()
                .do_send(GameFinished {});
            self.on_game_finished();
        }
    }

    pub fn get_game_data(&self) -> GameData {
        let game_state = self.get_game_state();
        GameData {
            game_state,
            multiplier: self.current_multiplier.load(Ordering::Relaxed),
            betting_time_left_ms: self.betting_time_left.load(Ordering::Relaxed) * 1000,
            round_time_elapsed_ms: self.round_time_elapsed.load(Ordering::Relaxed) * 1000,
        }
    }

    fn get_game_state(&self) -> GameState {
        if self.is_betting_in_progress.load(Ordering::SeqCst) {
            return GameState::BettingInProgress;
        }

        if self.is_game_round_in_progress.load(Ordering::SeqCst) {
            return GameState::GameInProgress;
        }

        GameState::Idle
    }

    fn get_round_result(&self) -> Option<RoundResult> {
        let client_seed = self.client_seed.lock().unwrap();

        if let Some(crash_point) = CrashGameMath::generate_crash_point(
            &self.server_seed,
            &client_seed,
            &self.house_edge_pct,
            &self.round_id,
        ) {
            let multiplier_u32 = (crash_point * 100.0) as u32;
            return Some(RoundResult {
                multiplier: multiplier_u32,
                animation_duration: 10,
            });
        }

        None
    }

    fn on_betting_timer_finished(&self) {
        info!("Betting timer finished!");
        // nb! this is needed
        self.betting_time_left.store(0, Ordering::SeqCst);
        self.set_betting_in_progress(false);
        self.set_game_in_progress(true);
    }

    fn on_game_finished(&self) {
        info!("Game finished!");

        // reset game states
        self.set_game_in_progress(false);
        self.set_betting_in_progress(false);

        self.reset_game_data();
    }

    fn set_game_in_progress(&self, val: bool) {
        self.is_game_round_in_progress.store(val, Ordering::SeqCst);
    }

    fn set_betting_in_progress(&self, val: bool) {
        self.is_betting_in_progress.store(val, Ordering::SeqCst);
    }

    fn reset_game_data(&self) {
        self.betting_time_left
            .store(self.max_betting_time_duration, Ordering::SeqCst);
        self.round_time_elapsed.store(0, Ordering::SeqCst);
        self.current_multiplier.store(0, Ordering::SeqCst);
    }
}
