use actix::{Message, Recipient};

// messages sent between peer and gameServer

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub peer_addr: Recipient<PeerPlayerData>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub enum PeerPlayerData {
    PeerJoined { player_id: usize },
    PeerLeft { player_id: usize },
}
