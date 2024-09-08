use std::sync::{atomic::AtomicU32, Arc};

#[derive(Debug, Clone)]
pub struct GameStats {
    pub players_online: Arc<AtomicU32>,
}

impl GameStats {
    pub fn new() -> Self {
        Self {
            players_online: Arc::new(AtomicU32::new(0)),
        }
    }
}
