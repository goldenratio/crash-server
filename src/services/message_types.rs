use actix::{Message, Recipient};

// messages sent between peer and gameServer

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub peer_addr: Recipient<GameEvent>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub session_id: usize,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct PlayerJoined {
    pub session_id: usize,
    pub uuid: String,
    pub peer_addr: Recipient<GameEvent>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BetRequest {
    pub session_id: usize,
    pub bet_amount: u64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CrashOutRequest {
    pub session_id: usize,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub enum GameEvent {
    RemotePlayerJoined {
        display_name: String,
    },
    RemotePlayerLeft {
        display_name: String,
    },
    PlayerJoinedResponse {
        game_state: u8,
        multiplier: u32,
        /// in milliseconds
        betting_time_left_ms: u32,
        /// in milliseconds
        round_time_elapsed_ms: u32,
    },
    BettingTimerUpdate {
        /// in milliseconds
        betting_time_left_ms: u32,
    },
    GameRoundUpdate {
        multiplier: u32,
    },
}

// messages between gameServer and CrashGame
#[derive(Message)]
#[rtype(result = "()")]
pub struct BettingTimerUpdate {
    /// in milliseconds
    pub betting_time_left_ms: u32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameRoundUpdate {
    /// in milliseconds
    pub multiplier: u32,
}
