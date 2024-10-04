use actix::{Message, Recipient};

// messages sent between peer and gameServer

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {}

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
        players_online: u32,
    },
    RemotePlayerLeft {
        display_name: String,
        players_online: u32,
    },
    RemotePlayerBetsPlaced {
        display_name: String,
        bet_amount: u64,
    },
    RemotePlayerCrashOut {
        display_name: String,
        win_amount: u64,
    },
    PlayerJoinedResponse {
        game_state: u8,
        multiplier: u32,
        /// in milliseconds
        betting_time_left_ms: u32,
        /// in milliseconds
        round_time_elapsed_ms: u32,
        display_name: String,
        balance: u64,
    },
    BetResponse {
        balance: u64,
    },
    CrashOutResponse {
        win_amount: u64,
        multiplier: u32,
        balance: u64,
    },
    BettingTimerStarted {
        /// in milliseconds
        betting_time_left_ms: u32,
        round_id: u32,
        server_seed_hash: String,
        next_round_server_seed_hash: String,
    },
    BettingTimerUpdate {
        /// in milliseconds
        betting_time_left_ms: u32,
    },
    GameStarted {},
    GameRoundUpdate {
        multiplier: u32,
    },
    GameFinished {},
    GameError {},
}

// messages between gameServer and CrashGame
#[derive(Message)]
#[rtype(result = "()")]
pub struct BettingTimerStarted {
    /// in milliseconds
    pub betting_time_left_ms: u32,
    pub round_id: u32,
    pub server_seed_hash: String,
    pub next_round_server_seed_hash: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BettingTimerUpdate {
    /// in milliseconds
    pub betting_time_left_ms: u32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameStarted {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameRoundUpdate {
    /// in milliseconds
    pub multiplier: u32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameFinished {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameError {}
