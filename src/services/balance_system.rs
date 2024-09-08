#[derive(Debug, Clone)]
pub struct BalanceSystem {}

impl BalanceSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_guest_balance(&self, uuid: String) {
        //
    }

    pub fn load(&self, uuid: String) -> u64 {
        0
    }

    pub fn fetch_add(&self, uuid: String, amount_to_add: u64) -> u64 {
        0
    }

    pub fn fetch_sub(&self, uuid: String, amount_to_sub: u64) -> u64 {
        0
    }
}
