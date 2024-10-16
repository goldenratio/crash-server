use flatbuffers::FlatBufferBuilder;

use crate::{
    generated::game_schema_generated::gameplay_fbdata::{
        root_as_game_request_event, BetResponse, BetResponseArgs, BettingTimerStarted, BettingTimerStartedArgs, BettingTimerUpdate, BettingTimerUpdateArgs, CrashOutResponse, CrashOutResponseArgs, GameFinished, GameFinishedArgs, GameResponseEvent, GameResponseEventArgs, GameStarted, GameStartedArgs, GameUpdate, GameUpdateArgs, JoinGameResponse, JoinGameResponseArgs, RemotePlayerBetsPlaced, RemotePlayerBetsPlacedArgs, RemotePlayerCrashOut, RemotePlayerCrashOutArgs, RemotePlayerJoined, RemotePlayerJoinedArgs, RemotePlayerLeft, RemotePlayerLeftArgs, RequestMessages, ResponseMessage
    },
    services::peer::ClientData,
};

pub fn parse_gameplay_data(buf: &[u8]) -> ClientData {
    let gameplay = root_as_game_request_event(buf).unwrap();
    let event_type = gameplay.msg_type();

    match event_type {
        RequestMessages::JoinGameRequest => {
            if let Some(auth_data) = gameplay.msg_as_join_game_request() {
                let player_uuid = auth_data.player_uuid().unwrap_or_else(|| "");
                let jwt_token = auth_data.jwt_token().unwrap_or_else(|| "");

                return ClientData::JoinGameRequest {
                    jwt_token: jwt_token.to_string(),
                    player_uuid: player_uuid.to_string(),
                };
            }
        }
        RequestMessages::BetRequest => {
            if let Some(bet_data) = gameplay.msg_as_bet_request() {
                let bet_amount = bet_data.bet_amount();
                return ClientData::BetRequest { bet_amount };
            }
        }
        RequestMessages::CrashOutRequest => {
            return ClientData::CrashOutRequest {};
        }
        _ => {
            return ClientData::Unknown;
        }
    }

    ClientData::Unknown
}

pub fn create_join_game_response_success(
    game_state: u8,
    betting_time_left: u32,
    multiplier: u32,
    round_time_elapsed_ms: u32,
    display_name: String,
    balance: u64,
) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    // Reset the `bytes` Vec to a clean state.
    bytes.clear();

    // Reset the `FlatBufferBuilder` to a clean state.
    bldr.reset();

    // Create a temporary `UserArgs` object to build a `User` object.
    // (Note how we call `bldr.create_string` to create the UTF-8 string
    // ergonomically.)
    let display_name_str = bldr.create_string(&display_name);

    let msg = JoinGameResponse::create(
        &mut bldr,
        &JoinGameResponseArgs {
            game_state: game_state,
            betting_time_left: betting_time_left,
            multiplier: multiplier,
            round_time_elapsed: round_time_elapsed_ms,
            display_name: Option::from(display_name_str),
            balance: balance,
        },
    )
    .as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::JoinGameResponse,
        msg: Option::from(msg),
    };

    // Call the `User::create` function with the `FlatBufferBuilder` and our
    // UserArgs object, to serialize the data to the FlatBuffer. The returned
    // value is an offset used to track the location of this serializaed data.
    let user_offset = GameResponseEvent::create(&mut bldr, &args);

    // Finish the write operation by calling the generated function
    // `finish_user_buffer` with the `user_offset` created by `User::create`.
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}

pub fn create_bet_response(balance: u64) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let msg = BetResponse::create(
        &mut bldr,
        &BetResponseArgs {
            balance: balance,
        },
    )
    .as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::BetResponse,
        msg: Option::from(msg),
    };

    let user_offset = GameResponseEvent::create(&mut bldr, &args);
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}

pub fn create_crash_out_response(win_amount: u64, multiplier: u32, balance: u64) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let msg = CrashOutResponse::create(
        &mut bldr,
        &CrashOutResponseArgs {
            win_amount: win_amount,
            multiplier: multiplier,
            balance: balance,
        },
    )
    .as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::CrashOutResponse,
        msg: Option::from(msg),
    };

    let user_offset = GameResponseEvent::create(&mut bldr, &args);
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}

pub fn create_game_update_response(multiplier: u32) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let msg = GameUpdate::create(
        &mut bldr,
        &GameUpdateArgs {
            multiplier: multiplier,
        },
    )
    .as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::GameUpdate,
        msg: Option::from(msg),
    };

    let user_offset = GameResponseEvent::create(&mut bldr, &args);
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}

pub fn create_game_started_response() -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let msg = GameStarted::create(&mut bldr, &GameStartedArgs {}).as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::GameStarted,
        msg: Option::from(msg),
    };

    let user_offset = GameResponseEvent::create(&mut bldr, &args);
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}

pub fn create_game_finished_response() -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let msg = GameFinished::create(&mut bldr, &GameFinishedArgs {}).as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::GameFinished,
        msg: Option::from(msg),
    };

    let user_offset = GameResponseEvent::create(&mut bldr, &args);
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}

pub fn create_betting_timer_update_response(betting_time_left: u32) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let msg = BettingTimerUpdate::create(
        &mut bldr,
        &BettingTimerUpdateArgs {
            betting_time_left: betting_time_left,
        },
    )
    .as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::BettingTimerUpdate,
        msg: Option::from(msg),
    };

    let user_offset = GameResponseEvent::create(&mut bldr, &args);
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}

pub fn create_betting_timer_started_response(
    betting_time_left: u32,
    round_id: u32,
    server_seed_hash: String,
    next_round_server_seed_hash: String,
) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let server_seed_hash_str = bldr.create_string(&server_seed_hash);
    let next_round_server_seed_hash_str = bldr.create_string(&next_round_server_seed_hash);

    let msg = BettingTimerStarted::create(
        &mut bldr,
        &BettingTimerStartedArgs {
            betting_time_left: betting_time_left,
            round_id,
            server_seed_hash: Option::from(server_seed_hash_str),
            next_round_server_seed_hash: Option::from(next_round_server_seed_hash_str),
        },
    )
    .as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::BettingTimerStarted,
        msg: Option::from(msg),
    };

    let user_offset = GameResponseEvent::create(&mut bldr, &args);
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}

pub fn create_remote_player_joined_response(display_name: String, players_online: u32) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let display_name_str = bldr.create_string(&display_name);

    let msg = RemotePlayerJoined::create(
        &mut bldr,
        &RemotePlayerJoinedArgs {
            display_name: Option::from(display_name_str),
            players_online,
        },
    )
    .as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::RemotePlayerJoined,
        msg: Option::from(msg),
    };

    let user_offset = GameResponseEvent::create(&mut bldr, &args);
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}

pub fn create_remote_player_left_response(display_name: String, players_online: u32) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let display_name_str = bldr.create_string(&display_name);

    let msg = RemotePlayerLeft::create(
        &mut bldr,
        &RemotePlayerLeftArgs {
            display_name: Option::from(display_name_str),
            players_online,
        },
    )
    .as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::RemotePlayerLeft,
        msg: Option::from(msg),
    };

    let user_offset = GameResponseEvent::create(&mut bldr, &args);
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}

pub fn create_remote_player_bets_placed_response(display_name: String, bet_amount: u64) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let display_name_str = bldr.create_string(&display_name);

    let msg = RemotePlayerBetsPlaced::create(
        &mut bldr,
        &RemotePlayerBetsPlacedArgs {
            display_name: Option::from(display_name_str),
            bet_amount: bet_amount,
        },
    )
    .as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::RemotePlayerBetsPlaced,
        msg: Option::from(msg),
    };

    let user_offset = GameResponseEvent::create(&mut bldr, &args);
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}

pub fn create_remote_player_crash_out_response(display_name: String, win_amount: u64) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let display_name_str = bldr.create_string(&display_name);

    let msg = RemotePlayerCrashOut::create(
        &mut bldr,
        &RemotePlayerCrashOutArgs {
            display_name: Option::from(display_name_str),
            win_amount: win_amount,
        },
    )
    .as_union_value();

    let args = GameResponseEventArgs {
        msg_type: ResponseMessage::RemotePlayerCrashOut,
        msg: Option::from(msg),
    };

    let user_offset = GameResponseEvent::create(&mut bldr, &args);
    bldr.finish(user_offset, None);

    // Copy the serialized FlatBuffers data to our own byte buffer.
    let finished_data = bldr.finished_data();
    bytes.extend_from_slice(finished_data);

    bytes
}
