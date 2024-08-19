use std::sync::{atomic::AtomicUsize, Arc, Mutex};

#[derive(Debug)]
pub struct GameStats {
    pub players_online: Arc<AtomicUsize>,
}

impl GameStats {
    pub fn new() -> Self {
        Self {
            players_online: Arc::new(AtomicUsize::new(0)),
        }
    }
}
