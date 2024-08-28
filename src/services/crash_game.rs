use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use actix::spawn;
use actix_web::rt::time;
use map_range::MapRange;

#[derive(Debug, Clone)]
pub struct CrashGame {
    is_game_running: Arc<AtomicBool>,
}

struct RoundResult {
    multiplier: u64,
    /// in seconds
    animation_duration: u64,
}

impl CrashGame {
    pub fn new() -> Self {
        Self {
            is_game_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) {
        if self.is_game_running.swap(true, Ordering::SeqCst) {
            println!("game is already running!");
            return;
        }

        let round_result = self.get_round_result();

        let mut current_second: u64 = 0;
        let mut current_multiplier = 0;
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
                println!("current_multiplier {:?}", current_multiplier);
            }
        });
    }

    fn get_round_result(&self) -> RoundResult {
        RoundResult {
            multiplier: 142,
            animation_duration: 10,
        }
    }

    fn on_game_finished(&self) {
        println!("Game finished!");
        self.is_game_running.store(false, Ordering::SeqCst);
    }
}
