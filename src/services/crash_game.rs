use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use actix::{spawn, Addr};
use actix_web::rt::time;
use log::info;
use map_range::MapRange;

use crate::services::message_types::{BettingTimerUpdate, GameRoundUpdate};

use super::game_server::GameServer;

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    IDLE,
    BETTING_IN_PROGRESS,
    GAME_IN_PROGRESS,
}

impl Into<u8> for GameState {
    fn into(self) -> u8 {
        match self {
            GameState::IDLE => 0,
            GameState::BETTING_IN_PROGRESS => 1,
            GameState::GAME_IN_PROGRESS => 2,
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
}

const BETTING_TIMER_MAX_VAL: u32 = 5;

struct RoundResult {
    multiplier: u32,
    /// in seconds
    animation_duration: u32,
}

impl CrashGame {
    pub fn new() -> Self {
        Self {
            is_betting_in_progress: Arc::new(AtomicBool::new(false)),
            is_game_round_in_progress: Arc::new(AtomicBool::new(false)),
            betting_time_left: Arc::new(AtomicU32::new(0)),
            round_time_elapsed: Arc::new(AtomicU32::new(0)),
            current_multiplier: Arc::new(AtomicU32::new(0)),
            game_server_addr: None,
        }
    }

    pub fn set_game_server_addr(&mut self, addr: Addr<GameServer>) {
        self.game_server_addr = Option::from(addr);
    }

    pub fn start_betting_timer(&self) {
        let valid = match self.get_game_state() {
            GameState::IDLE => true,
            _ => false,
        };

        if !valid {
            // game is not idle, can't start new round
            info!("betting timer is already running!");
            return;
        }

        self.reset_game_data();
        self.set_betting_in_progress(true);

        let game = Arc::new(self.clone()); // or self.clone()

        let mut interval = time::interval(Duration::from_secs(1));
        let mut time_left = BETTING_TIMER_MAX_VAL;
        spawn(async move {
            loop {
                // note: can't be 0, u64 out of range
                if time_left <= 0 {
                    info!("betting timer is over, no more bets!");
                    game.on_betting_timer_finished();
                    game.start_game();
                    return;
                }
                interval.tick().await;
                time_left = game.betting_time_left.fetch_sub(1, Ordering::SeqCst);
                // info!("time left brrr: {:?}", time_left);
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
            GameState::GAME_IN_PROGRESS => true,
            _ => false,
        };

        if !valid {
            info!("game is in-progress already!");
            return;
        }

        let round_result = self.get_round_result();

        let mut current_second: u32 = 0;
        let game = Arc::new(self.clone()); // or self.clone()

        let mut interval = time::interval(Duration::from_secs(1));
        spawn(async move {
            loop {
                if current_second >= round_result.animation_duration {
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

    pub fn place_bets(&self, player_uuid: &String, bet_amount: u64) {
        //
    }

    pub fn crash_out(&self, player_uuid: &String) {
        //
    }

    fn get_game_state(&self) -> GameState {
        if self.is_betting_in_progress.load(Ordering::SeqCst) {
            return GameState::BETTING_IN_PROGRESS;
        }

        if self.is_game_round_in_progress.load(Ordering::SeqCst) {
            return GameState::GAME_IN_PROGRESS;
        }

        GameState::IDLE
    }

    fn get_round_result(&self) -> RoundResult {
        RoundResult {
            multiplier: 142,
            animation_duration: 10,
        }
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
        // todo: move timer value to config
        self.betting_time_left
            .store(BETTING_TIMER_MAX_VAL, Ordering::SeqCst);
        self.round_time_elapsed.store(0, Ordering::SeqCst);
    }
}
