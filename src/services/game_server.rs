use actix::{Actor, AsyncContext, Context, Handler, Recipient};
use log::{info, warn};
use rand::{rngs::ThreadRng, Rng};
use std::{collections::HashMap, sync::atomic::Ordering};

use crate::services::{crash_game::GameState, generate_username::generate_guest_username};

use super::{
    balance_system::BalanceSystem,
    crash_game::CrashGame,
    env_settings::EnvSettings,
    game_stats::GameStats,
    message_types::{
        BetRequest, BettingTimerStarted, BettingTimerUpdate, Connect, CrashOutRequest, Disconnect,
        GameError, GameEvent, GameFinished, GameRoundUpdate, GameStarted, PlayerJoined,
    },
};

#[derive(Debug)]
pub struct GameServer {
    peers: HashMap<String, PeerInfo>,
    session_to_uuid: HashMap<usize, String>,
    bet_map: HashMap<String, u64>,
    rng: ThreadRng,
    game_stats: GameStats,
    crash_game: CrashGame,
    balance_system: BalanceSystem,
}

#[derive(Debug)]
struct PeerInfo {
    addr: Recipient<GameEvent>,
    display_name: String,
}

impl GameServer {
    pub fn new(
        game_stats: GameStats,
        env_settings: EnvSettings,
        balance_system: BalanceSystem,
    ) -> Self {
        Self {
            peers: HashMap::new(),
            session_to_uuid: HashMap::new(),
            bet_map: HashMap::new(),
            rng: rand::thread_rng(),
            game_stats: game_stats,
            crash_game: CrashGame::new(
                env_settings.betting_time_duration,
                env_settings.house_edge_pct,
            ),
            balance_system: balance_system,
        }
    }

    fn broadcast(&self, event: GameEvent, exclude_uuid: Option<&str>) {
        for (uuid, peer) in &self.peers {
            if Some(uuid.as_str()) != exclude_uuid {
                peer.addr.do_send(event.clone());
            }
        }
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.crash_game.set_game_server_addr(ctx.address());
    }
}

impl Handler<Connect> for GameServer {
    type Result = usize;

    fn handle(&mut self, _: Connect, _: &mut Self::Context) -> Self::Result {
        info!("peer connected!");
        self.rng.gen()
    }
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        info!("peer disconnected!");

        let players_online = self
            .game_stats
            .players_online
            .fetch_sub(1, Ordering::SeqCst);

        if let Some(uuid) = self.session_to_uuid.remove(&msg.session_id) {
            if let Some(peer) = self.peers.remove(&uuid) {
                self.broadcast(
                    GameEvent::RemotePlayerLeft {
                        display_name: peer.display_name,
                        players_online,
                    },
                    Some(&uuid),
                );
            }
        }
    }
}

impl Handler<PlayerJoined> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: PlayerJoined, _: &mut Self::Context) -> Self::Result {
        info!("peer joined the game! {:?}", msg.uuid);

        let display_name = generate_guest_username();

        let peer_info = PeerInfo {
            addr: msg.peer_addr.clone(),
            display_name: display_name.clone(),
        };

        self.peers.insert(msg.uuid.clone(), peer_info);
        self.session_to_uuid
            .insert(msg.session_id, msg.uuid.clone());

        let players_online = self
            .game_stats
            .players_online
            .fetch_add(1, Ordering::SeqCst);

        let game_data = self.crash_game.get_game_data();

        msg.peer_addr.do_send(GameEvent::PlayerJoinedResponse {
            betting_time_left_ms: game_data.betting_time_left_ms,
            game_state: game_data.game_state.into(),
            multiplier: game_data.multiplier,
            round_time_elapsed_ms: game_data.round_time_elapsed_ms,
            display_name: display_name.clone(),
            balance: 0,
        });

        self.broadcast(
            GameEvent::RemotePlayerJoined {
                display_name,
                players_online,
            },
            Some(&msg.uuid),
        );

        if matches!(game_data.game_state, GameState::Idle) {
            self.crash_game.start_betting_timer();
        }
    }
}

impl Handler<BetRequest> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: BetRequest, _: &mut Self::Context) -> Self::Result {
        // get uuid from session_id
        if let Some(uuid) = self.session_to_uuid.get(&msg.session_id) {
            let game_data = self.crash_game.get_game_data();

            if matches!(game_data.game_state, GameState::BettingInProgress) {
                if !self.bet_map.contains_key(uuid) {
                    info!("bets placed! {:?} {:?}", uuid, msg.bet_amount);
                    self.bet_map.insert(uuid.clone(), msg.bet_amount);

                    if let Some(peer) = self.peers.get(uuid) {
                        self.broadcast(
                            GameEvent::RemotePlayerBetsPlaced {
                                display_name: peer.display_name.clone(),
                                bet_amount: msg.bet_amount,
                            },
                            Some(uuid),
                        );
                    }
                } else {
                    warn!("bets already placed for uuid {:?}", uuid);
                }
            } else {
                warn!("bets received when state is not in BETTING_IN_PROGRESS");
            }
        } else {
            warn!("BetRequest: unknown session id {:?}", msg.session_id);
        }
    }
}

impl Handler<CrashOutRequest> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: CrashOutRequest, _: &mut Self::Context) -> Self::Result {
        if let Some(uuid) = self.session_to_uuid.get(&msg.session_id) {
            let game_data = self.crash_game.get_game_data();

            if matches!(game_data.game_state, GameState::GameInProgress) {
                if let Some(bet_amount) = self.bet_map.remove(uuid) {
                    let win_amount = bet_amount * game_data.multiplier as u64;
                    info!(
                        "player crashed out! {:?}, winAmount: {:?}",
                        uuid, win_amount
                    );

                    if let Some(peer) = self.peers.get(uuid) {
                        peer.addr.do_send(GameEvent::CrashOutResponse {
                            win_amount,
                            multiplier: game_data.multiplier,
                            balance: 0,
                        });

                        self.broadcast(
                            GameEvent::RemotePlayerCrashOut {
                                display_name: peer.display_name.clone(),
                                win_amount,
                            },
                            Some(uuid),
                        );
                    }
                }
            } else {
                warn!("crashOut received when state is not in GAME_IN_PROGRESS");
            }
        } else {
            warn!("CrashOutRequest: unknown session id {:?}", msg.session_id);
        }
    }
}

// Implement handlers for BettingTimerStarted, BettingTimerUpdate, GameRoundUpdate, GameStarted, and GameFinished
// using the broadcast method

impl Handler<BettingTimerStarted> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: BettingTimerStarted, _: &mut Self::Context) -> Self::Result {
        self.broadcast(
            GameEvent::BettingTimerStarted {
                betting_time_left_ms: msg.betting_time_left_ms,
                round_id: msg.round_id,
                server_seed_hash: msg.server_seed_hash,
                next_round_server_seed_hash: msg.next_round_server_seed_hash,
            },
            None,
        );
    }
}

impl Handler<BettingTimerUpdate> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: BettingTimerUpdate, _: &mut Self::Context) -> Self::Result {
        // info!("time left: {:?}", msg.betting_time_left_ms);
        self.broadcast(
            GameEvent::BettingTimerUpdate {
                betting_time_left_ms: msg.betting_time_left_ms,
            },
            None,
        );
    }
}

impl Handler<GameRoundUpdate> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: GameRoundUpdate, _: &mut Self::Context) -> Self::Result {
        // info!("multiplier: {:?}", msg.multiplier);
        self.broadcast(
            GameEvent::GameRoundUpdate {
                multiplier: msg.multiplier,
            },
            None,
        );
    }
}

impl Handler<GameStarted> for GameServer {
    type Result = ();

    fn handle(&mut self, _: GameStarted, _: &mut Self::Context) -> Self::Result {
        self.broadcast(GameEvent::GameStarted {}, None);
    }
}

impl Handler<GameFinished> for GameServer {
    type Result = ();

    fn handle(&mut self, _: GameFinished, _: &mut Self::Context) -> Self::Result {
        self.broadcast(GameEvent::GameFinished {}, None);
        self.bet_map.clear();

        if !self.peers.is_empty() {
            // self.crash_game.start_betting_timer();
        }
    }
}

impl Handler<GameError> for GameServer {
    type Result = ();

    fn handle(&mut self, _: GameError, _: &mut Self::Context) -> Self::Result {
        self.broadcast(GameEvent::GameError {}, None);
        self.bet_map.clear();
    }
}
