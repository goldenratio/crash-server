use flatbuffers::FlatBufferBuilder;

use crate::{
    generated::game_schema_generated::gameplay_fbdata::{
        root_as_game_request_event, BettingTimerUpdate, BettingTimerUpdateArgs, GameResponseEvent,
        GameResponseEventArgs, GameUpdate, GameUpdateArgs, JoinGameResponse, JoinGameResponseArgs,
        RequestMessages, ResponseMessage,
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
        RequestMessages::CrashOut => {
            return ClientData::CrashOut {};
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

    let msg = JoinGameResponse::create(
        &mut bldr,
        &JoinGameResponseArgs {
            game_state: game_state,
            betting_time_left: betting_time_left,
            multiplier: multiplier,
            round_time_elapsed: round_time_elapsed_ms,
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

pub fn create_game_update_response(multiplier: u32) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let msg = GameUpdate::create(
        &mut bldr,
        &&GameUpdateArgs {
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

pub fn create_betting_timer_update_response(betting_time_left: u32) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    bytes.clear();
    bldr.reset();

    let msg = BettingTimerUpdate::create(
        &mut bldr,
        &&BettingTimerUpdateArgs {
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
