use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use actix::spawn;
use actix_web::rt::time;
use log::info;
use map_range::MapRange;

#[derive(Debug)]
pub enum GameState {
    IDLE,
    BETTING_IN_PROGRESS,
    GAME_IN_PROGRESS
}

#[derive(Debug, Clone)]
pub struct CrashGame {
    is_betting_in_progress: Arc<AtomicBool>,
    is_game_round_in_progress: Arc<AtomicBool>,
    betting_time_left: Arc<AtomicU32>,
}

const BETTING_TIMER_MAX_VAL: u32 = 5;

struct RoundResult {
    multiplier: u64,
    /// in seconds
    animation_duration: u64,
}

impl CrashGame {
    pub fn new() -> Self {
        Self {
            is_betting_in_progress: Arc::new(AtomicBool::new(false)),
            is_game_round_in_progress: Arc::new(AtomicBool::new(false)),
            betting_time_left: Arc::new(AtomicU32::new(0)),
        }
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

        self.reset_betting_time_left();
        self.set_betting_in_progress(true);

        let game = Arc::new(self.clone()); // or self.clone()

        let mut interval = time::interval(Duration::from_secs(1));
        let mut time_left = BETTING_TIMER_MAX_VAL;
        spawn(async move {
            loop {
                if time_left <= 0 {
                    info!("betting timer is over, no more bets!");
                    game.on_betting_timer_finished();
                    game.start_game();
                    return;
                }
                interval.tick().await;
                time_left = game.decrement_betting_time_left();
                info!("time left: {:?}", time_left);
            }
        });
    }

    fn start_game(&self) {
        let valid = match self.get_game_state() {
            GameState::GAME_IN_PROGRESS => true,
            _ => false,
        };

        if !valid {
            info!("game is already running!");
            return;
        }

        self.set_game_in_progress(true);

        let round_result = self.get_round_result();

        let mut current_second: u64 = 0;
        let mut current_multiplier: u64 = 0;
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

                current_multiplier = current_second.map_range(
                    0..round_result.animation_duration,
                    0..round_result.multiplier,
                );
                info!("current_multiplier {:?}", current_multiplier);
                // todo: send updates peers
            }
        });
    }

    pub fn get_game_state(&self) -> GameState {
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
        self.set_betting_in_progress(false);
        self.set_game_in_progress(true);
    }

    fn on_game_finished(&self) {
        info!("Game finished!");
        self.set_game_in_progress(false);
        self.reset_betting_time_left();
    }

    fn set_game_in_progress(&self, val: bool) {
        self.is_game_round_in_progress.store(val, Ordering::SeqCst);
    }

    fn set_betting_in_progress(&self, val: bool) {
        self.is_betting_in_progress.store(val, Ordering::SeqCst);
    }

    fn decrement_betting_time_left(&self) -> u32 {
        return self.betting_time_left.fetch_sub(1, Ordering::SeqCst);
    }

    fn reset_betting_time_left(&self) {
        // todo: move timer value to config
        self.betting_time_left.store(BETTING_TIMER_MAX_VAL, Ordering::SeqCst);
    }
}
