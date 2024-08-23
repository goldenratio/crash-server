use crate::{
    generated::game_schema_generated::gameplay_fbdata::{
        root_as_game_request_event, RequestMessages,
    },
    services::peer::ClientData,
};

pub fn parse_gameplay_data(buf: &[u8]) -> ClientData {
    let gameplay = root_as_game_request_event(buf).unwrap();
    let event_type = gameplay.msg_type();

    if event_type == RequestMessages::Authenticate {
        if let Some(auth_data) = gameplay.msg_as_authenticate() {
            let player_uuid = auth_data.player_uuid().unwrap_or_else(|| "");
            let jwt_token = auth_data.jwt_token().unwrap_or_else(|| "");

            return ClientData::Authenticate {
                jwt_token: jwt_token.to_string(),
                player_uuid: player_uuid.to_string(),
            };
        }
    }
    ClientData::Unknown
}
