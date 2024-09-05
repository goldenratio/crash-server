use std::{collections::HashMap, sync::atomic::Ordering};

use actix::{Actor, AsyncContext, Context, Handler, Recipient};
use actix_web::web;
use log::{info, warn};
use rand::{rngs::ThreadRng, Rng};

use crate::services::{
    crash_game::{GameData, GameState},
    generate_username::generate_guest_username,
};

use super::{
    crash_game::CrashGame,
    game_stats::GameStats,
    message_types::{
        BetRequest, BettingTimerStarted, BettingTimerUpdate, Connect, CrashOutRequest, Disconnect,
        GameEvent, GameFinished, GameRoundUpdate, GameStarted, PlayerJoined,
    },
};

#[derive(Debug)]
pub struct GameServer {
    peer_addr_map: HashMap<String, Recipient<GameEvent>>,
    peer_session_uuid_map: HashMap<usize, String>,
    peer_display_name_map: HashMap<String, String>,
    bet_map: HashMap<String, u64>,
    rng: ThreadRng,
    game_stats: web::Data<GameStats>,
    crash_game: CrashGame,
}

impl GameServer {
    pub fn new(game_stats: web::Data<GameStats>) -> GameServer {
        Self {
            peer_addr_map: Default::default(),
            peer_session_uuid_map: Default::default(),
            peer_display_name_map: Default::default(),
            bet_map: Default::default(),
            rng: rand::thread_rng(),
            game_stats,
            crash_game: CrashGame::new(),
        }
    }
}

impl Actor for GameServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.crash_game.set_game_server_addr(addr);
    }
}

impl Handler<Connect> for GameServer {
    type Result = usize;

    fn handle(&mut self, _: Connect, _: &mut Self::Context) -> Self::Result {
        info!("peer connected!");
        // register session with random id
        let session_id = self.rng.gen::<usize>();
        session_id
    }
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        info!("peer disconnected!");

        // remove peer address
        if let Some(uuid) = self.peer_session_uuid_map.get(&msg.session_id) {
            self.peer_addr_map.remove(uuid);
            if let Some(guest_display_name) = self.peer_display_name_map.get(uuid) {
                // notify other players
                for (client_id, client_addr) in &self.peer_addr_map {
                    // skip the current player
                    if client_id != uuid {
                        client_addr.do_send(GameEvent::RemotePlayerLeft {
                            display_name: guest_display_name.clone(),
                        });
                    }
                }
            }

            self.peer_display_name_map.remove(uuid);
        }
        self.peer_session_uuid_map.remove(&msg.session_id);

        self.game_stats
            .players_online
            .fetch_sub(1, Ordering::SeqCst);
    }
}

impl Handler<PlayerJoined> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: PlayerJoined, _: &mut Self::Context) -> Self::Result {
        info!("peer joined the game! {:?}", msg.uuid);

        let guest_display_name = generate_guest_username();

        self.peer_addr_map
            .insert(msg.uuid.clone(), msg.peer_addr.clone());

        self.peer_display_name_map
            .insert(msg.uuid.clone(), guest_display_name.clone());

        self.peer_session_uuid_map
            .insert(msg.session_id, msg.uuid.clone());

        self.game_stats
            .players_online
            .fetch_add(1, Ordering::SeqCst);

        let game_data = self.crash_game.get_game_data();

        // send player current game state
        msg.peer_addr.do_send(GameEvent::PlayerJoinedResponse {
            betting_time_left_ms: game_data.betting_time_left_ms,
            game_state: game_data.game_state.into(),
            multiplier: game_data.multiplier,
            round_time_elapsed_ms: game_data.round_time_elapsed_ms,
            display_name: guest_display_name.clone(),
        });

        // notify other players
        for (client_id, client_addr) in &self.peer_addr_map {
            // skip the current player
            if *client_id != msg.uuid {
                client_addr.do_send(GameEvent::RemotePlayerJoined {
                    display_name: guest_display_name.clone(),
                });
            }
        }

        match game_data.game_state {
            GameState::Idle => {
                // start betting timer, If game is idle
                self.crash_game.start_betting_timer();
            }
            _ => {
                // do nothing
            }
        };
    }
}

impl Handler<BetRequest> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: BetRequest, _: &mut Self::Context) -> Self::Result {
        if let Some(uuid) = self.peer_session_uuid_map.get(&msg.session_id) {
            let GameData { game_state, .. } = self.crash_game.get_game_data();
            if matches!(game_state, GameState::BettingInProgress) {
                info!("bets placed! {:?} {:?}", uuid, msg.bet_amount);
                // place bets and stuff
                self.bet_map.insert(uuid.clone(), msg.bet_amount);
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
        if let Some(uuid) = self.peer_session_uuid_map.get(&msg.session_id) {
            let GameData {
                game_state,
                multiplier,
                ..
            } = self.crash_game.get_game_data();
            if matches!(game_state, GameState::GameInProgress) {
                let bet_amount = self.bet_map.get(uuid).unwrap_or_else(|| &0);
                if *bet_amount > 0 {
                    let win_amount = *bet_amount * multiplier as u64;
                    info!(
                        "player crashed out! {:?}, winAmount: {:?}",
                        uuid, win_amount
                    );
                    // todo: update result in db

                    // remove from map
                    self.bet_map.remove(uuid);

                    // send result to the owner addr
                    for (client_id, client_addr) in &self.peer_addr_map {
                        if client_id == uuid {
                            client_addr.do_send(GameEvent::CrashOutResponse {
                                win_amount: win_amount,
                                multiplier: multiplier,
                            });
                        } else {
                            warn!("unable to send crash out response to peer client!");
                        }
                    }

                    // send notification to other players
                    if let Some(display_name) = self.peer_display_name_map.get(uuid) {
                        for (client_id, client_addr) in &self.peer_addr_map {
                            // skip the current player
                            if client_id != uuid {
                                client_addr.do_send(GameEvent::RemotePlayerCrashOut {
                                    display_name: display_name.clone(),
                                    win_amount: win_amount,
                                });
                            }
                        }
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

// --------------------------------
// messages from CrashGame
// --------------------------------

impl Handler<BettingTimerStarted> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: BettingTimerStarted, _: &mut Self::Context) -> Self::Result {
        for (_, client_addr) in &self.peer_addr_map {
            client_addr.do_send(GameEvent::BettingTimerStarted {
                betting_time_left_ms: msg.betting_time_left_ms,
            });
        }
    }
}

impl Handler<BettingTimerUpdate> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: BettingTimerUpdate, _: &mut Self::Context) -> Self::Result {
        info!("time left: {:?}", msg.betting_time_left_ms);
        for (_, client_addr) in &self.peer_addr_map {
            client_addr.do_send(GameEvent::BettingTimerUpdate {
                betting_time_left_ms: msg.betting_time_left_ms,
            });
        }
    }
}

impl Handler<GameRoundUpdate> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: GameRoundUpdate, _: &mut Self::Context) -> Self::Result {
        info!("multiplier: {:?}", msg.multiplier);
        for (_, client_addr) in &self.peer_addr_map {
            client_addr.do_send(GameEvent::GameRoundUpdate {
                multiplier: msg.multiplier,
            });
        }
    }
}

impl Handler<GameStarted> for GameServer {
    type Result = ();

    fn handle(&mut self, _: GameStarted, _: &mut Self::Context) -> Self::Result {
        for (_, client_addr) in &self.peer_addr_map {
            client_addr.do_send(GameEvent::GameStarted {});
        }
    }
}

impl Handler<GameFinished> for GameServer {
    type Result = ();

    fn handle(&mut self, _: GameFinished, _: &mut Self::Context) -> Self::Result {
        for (_, client_addr) in &self.peer_addr_map {
            client_addr.do_send(GameEvent::GameFinished {});
            self.bet_map.clear();

            let can_start_new_game = self.peer_addr_map.len() > 0;
            if can_start_new_game {
                self.crash_game.start_betting_timer();
            }
        }
    }
}
