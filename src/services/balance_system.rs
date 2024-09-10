use std::{
    collections::HashMap,
    sync::{atomic::{AtomicU64, Ordering}, Arc, RwLock},
};

use log::info;

const DEFAULT_GUEST_BALANCE: u64 = 999_900;

#[derive(Debug, Clone)]
pub struct BalanceSystem {
    balance_map: Arc<RwLock<HashMap<String, AtomicU64>>>,
}

impl BalanceSystem {
    pub fn new() -> Self {
        Self {
            balance_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Ensures that a user with the given UUID has an entry in the balance map.
    /// If the user does not exist, their balance is initialized.
    pub fn ensure_balance(&self, uuid: String) {
        let mut map = self.balance_map.write().unwrap();
        map.entry(uuid).or_insert_with(|| AtomicU64::new(DEFAULT_GUEST_BALANCE));
    }

    /// Fetches the balance for a given user UUID. Returns 0 if the user does not exist.
    pub fn fetch_balance(&self, uuid: &str) -> u64 {
        let map = self.balance_map.read().unwrap();
        if let Some(balance) = map.get(uuid) {
            balance.load(Ordering::Relaxed)
        } else {
            0
        }
    }

    /// Adds the given amount to the balance of the user with the provided UUID.
    pub fn add(&self, uuid: String, amount_to_add: u64) -> Result<u64, ()> {
        let map = self.balance_map.read().unwrap();
        if let Some(balance) = map.get(&uuid) {
            let new_balance = balance.fetch_add(amount_to_add, Ordering::SeqCst) + amount_to_add;
            info!("Added {} to balance of {}. New balance: {}", amount_to_add, uuid, new_balance);
            Ok(new_balance)
        } else {
            Err(())
        }
    }

    /// Subtracts the given amount from the balance of the user with the provided UUID.
    /// Returns an error if the user does not exist or if the subtraction would cause an underflow.
    pub fn sub(&self, uuid: String, amount_to_sub: u64) -> Result<u64, ()> {
        let map = self.balance_map.read().unwrap();
        if let Some(balance) = map.get(&uuid) {
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
    pub fn can_sub(&self, uuid: &str, amount_to_sub: u64) -> bool {
        let map = self.balance_map.read().unwrap();
        if let Some(balance) = map.get(uuid) {
            let current_balance = balance.load(Ordering::SeqCst);
            current_balance >= amount_to_sub
        } else {
            false
        }
    }
}
