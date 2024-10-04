use std::{
    collections::HashMap,
    sync::{atomic::{AtomicU64, Ordering}, Arc, RwLock},
};

use log::info;

const DEFAULT_GUEST_BALANCE: u64 = 999_900;

#[derive(Debug, Clone)]
pub struct BalanceSystem {
    balance_map: Arc<RwLock<HashMap<String, AtomicU64>>>,
    reserved_money_map: Arc<RwLock<HashMap<String, AtomicU64>>>,
}

impl BalanceSystem {
    pub fn new() -> Self {
        Self {
            balance_map: Arc::new(RwLock::new(HashMap::new())),
            reserved_money_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Ensures that a user with the given UUID has an entry in the balance map.
    /// If the user does not exist, their balance is initialized.
    pub fn ensure_balance(&self, uuid: String) {
        let mut map = self.balance_map.write().unwrap();
        map.entry(uuid.clone()).or_insert_with(|| AtomicU64::new(DEFAULT_GUEST_BALANCE));

        let mut map = self.reserved_money_map.write().unwrap();
        map.entry(uuid.clone()).or_insert_with(|| AtomicU64::new(0));
    }

    /// Fetches the balance for a given user UUID. Returns 0 if the user does not exist.
    pub fn fetch_balance(&self, uuid: &str) -> u64 {
        let balance_map = match self.balance_map.read() {
            Ok(m) => m,
            Err(_) => return 0,
        };

        let reserved_map = match self.reserved_money_map.read() {
            Ok(m) => m,
            Err(_) => return 0,
        };

        let balance = balance_map.get(uuid).map_or(0, |b| b.load(Ordering::Relaxed));
        let reserved = reserved_map.get(uuid).map_or(0, |r| r.load(Ordering::Relaxed));

        balance.saturating_sub(reserved)
    }

    /// Adds the given amount to the balance of the user with the provided UUID.
    pub fn add(&self, uuid: &str, amount_to_add: u64) -> Result<u64, ()> {
        let map = self.balance_map.read().unwrap();
        if let Some(balance) = map.get(uuid) {
            let new_balance = balance.fetch_add(amount_to_add, Ordering::SeqCst) + amount_to_add;
            info!("Added {} to balance of {}. New balance: {}", amount_to_add, uuid, new_balance);
            Ok(new_balance)
        } else {
            Err(())
        }
    }

    /// Subtracts the given amount from the balance of the user with the provided UUID.
    /// Returns an error if the user does not exist or if the subtraction would cause an underflow.
    fn sub(&self, uuid: &str, amount_to_sub: u64) -> Result<u64, ()> {
        let map = self.balance_map.read().unwrap();
        if let Some(balance) = map.get(uuid) {
            let current_balance = balance.load(Ordering::SeqCst);
            if current_balance >= amount_to_sub {
                let new_balance = balance.fetch_sub(amount_to_sub, Ordering::SeqCst) - amount_to_sub;
                info!("Subtracted {} from balance of {}. New balance: {}", amount_to_sub, uuid, new_balance);
                Ok(new_balance)
            } else {
                info!("Failed to subtract {} from balance of {}. Current balance: {}", amount_to_sub, uuid, current_balance);
                Err(())
            }
        } else {
            Err(())
        }
    }

    /// Checks if the user has enough balance to subtract the given amount.
    /// Returns true if the subtraction is possible, otherwise false.
    fn can_sub(&self, uuid: &str, amount_to_sub: u64) -> bool {
        let map = self.balance_map.read().unwrap();
        if let Some(balance) = map.get(uuid) {
            let current_balance = balance.load(Ordering::SeqCst);
            current_balance >= amount_to_sub
        } else {
            false
        }
    }

    pub fn reserve_bet_amount(&self, uuid: &str, amount_to_reserve: u64) -> bool {
        if self.can_sub(uuid, amount_to_reserve) {
            if let Ok(map) = self.reserved_money_map.read() {
                if let Some(amount) = map.get(uuid) {
                    amount.store(amount_to_reserve, Ordering::SeqCst);
                    return true;
                }
            }
        }
        false
    }

    pub fn commit_reserved_bet_amount(&self, uuid: &str) {
        if let Ok(map) = self.reserved_money_map.read() {
            if let Some(reserved_amount_atomic) = map.get(uuid) {
                let reserved_amount = reserved_amount_atomic.load(Ordering::SeqCst);
                let _ = self.sub(uuid, reserved_amount);
                reserved_amount_atomic.store(0, Ordering::SeqCst);
            }
        }
    }
}
