use hex;
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

// Function to generate SHA256 hash
pub fn sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub struct CrashGameMath {}

impl CrashGameMath {
    pub fn generate_crash_point(
        server_seed: &str,
        client_seed: &str,
        house_edge_pct: &f32,
        round_id: &u32,
    ) -> Option<f64> {
        let hex_hash = CrashGameMath::generate_round_hex_hash(server_seed, client_seed, round_id);

        let hs: u32 = 100 / (house_edge_pct * 100.0) as u32;

        if CrashGameMath::divisible(&hex_hash, hs) {
            return Some(1.0);
        }

        // 64 bit Double-precision floating-point format -> 12 = (Sign bit: 1 bit, Exponent: 11 bits)
        let precision: usize = 64 - 12;

        // 4 = Since each hex character represents 4 bits
        let h = u64::from_str_radix(&hex_hash[..(precision / 4)], 16).unwrap();
        let e = 2u64.pow(precision as u32);

        let result = ((100 * e - h) / (e - h)) as f64 / 100.0; // Round to 2 decimal places

        Some(result)
    }

    pub fn generate_seed() -> String {
        // Generate a random seed
        let mut rng = rand::thread_rng();
        let seed: String = (0..64)
            .map(|_| rng.gen_range(33..127) as u8 as char)
            .collect();

        seed
    }

    fn generate_round_hex_hash(server_seed: &str, client_seed: &str, round_id: &u32) -> String {
        let mut mac = HmacSha256::new_from_slice(server_seed.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(format!("{}{}", client_seed, round_id).as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    fn divisible(hash: &str, mod_val: u32) -> bool {
        // We will read in 4 hex at a time, but the first chunk might be a bit smaller
        // So ABCDEFGHIJ should be chunked like  AB CDEF GHIJ
        let mut val = 0;
        let o = hash.len() % 4;
        let start_index = if o > 0 { o - 4 } else { 0 };
        for n in (start_index..hash.len()).step_by(4) {
            let h = u64::from_str_radix(&hash[n..n + 4], 16).unwrap() % mod_val as u64;
            let b = val << 16; // same as val * Math.pow(2, 16)
            val = b + h;
        }
        val == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_crash_point_standard() {
        let mut total_payout: f64 = 0.0;
        let mut total_wager: f64 = 0.0;

        let rounds = 100_000;
        let bet_amount = 1.0;

        let mut max_val = 0.0;

        for _ in 0..rounds {
            let server_seed = &CrashGameMath::generate_seed();
            let client_seed = &format!(
                "{}{}{}",
                &CrashGameMath::generate_seed(),
                &CrashGameMath::generate_seed(),
                &CrashGameMath::generate_seed()
            );

            let house_edge_pct = 0.07; // value between 0 to 1
            let round_id = 1;

            let crash_point_multiplier = CrashGameMath::generate_crash_point(
                server_seed,
                client_seed,
                &house_edge_pct,
                &round_id,
            )
            .unwrap();

            // let multiplier = crash_point as f32 / 100.0;
            if crash_point_multiplier >= max_val {
                max_val = crash_point_multiplier;
            }
            if crash_point_multiplier > 100.00 {
                println!("{}", crash_point_multiplier);
            }
            total_wager += bet_amount;
            total_payout += bet_amount * crash_point_multiplier;
            // assert_eq!(crash_point, Some(0)); // Adjust this based on expected outcome
        }

        let rtp = total_payout / total_wager;

        println!("RTP: {:?} - {:.2}%", rtp, rtp * 100.00);
        println!("max_val: {:?}", max_val);
        // println!("Observed house edge: {:.2}%", (1.0 - 1.0 / average_result) * 100.0);
    }
}
