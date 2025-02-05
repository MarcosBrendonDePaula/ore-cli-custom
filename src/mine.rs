use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}};

use colored::*;
use drillx::{
    equix::{self},
    Hash, Solution,
};
use rand::{Rng, RngCore, thread_rng};
use solana_rpc_client::spinner;
use solana_sdk::signer::Signer;

use crate::{
    args::MineArgs,
    pool::submit_hash_to_pool,
    Miner,
};

impl Miner {
    pub async fn mine(&self, args: MineArgs) {
        // Get signer
        let signer = self.signer();

        // Check num threads
        self.check_num_cores(args.cores);

        // Start mining loop
        loop {
            // Run drillx
            let (solution, best_hash, best_difficulty) =
                Self::find_hash_par(args.cores, args.min_difficulty)
                    .await;

            // Submit hash to pool
            if let Err(e) = submit_hash_to_pool(
                bs58::encode(best_hash.h).into_string(),
                best_difficulty,
                signer.pubkey(),
            ).await {
                println!("Failed to submit hash to pool: {}", e);
            } else {
                println!("Successfully submitted hash to pool");
            }
        }
    }

    async fn find_hash_par(
        cores: u64,
        min_difficulty: u32,
    ) -> (Solution, Hash, u32) {
        // Dispatch job to each thread
        let stop_flag = Arc::new(AtomicBool::new(false));
        let progress_bar = Arc::new(spinner::new_progress_bar());
        progress_bar.set_message("Mining...");
        let core_ids = core_affinity::get_core_ids().unwrap();
        let handles: Vec<_> = core_ids
            .into_iter()
            .map(|i| {
                let progress_bar = progress_bar.clone();
                let stop_flag = stop_flag.clone();
                std::thread::spawn(move || {
                    let mut nonce = u64::MAX.saturating_div(cores).saturating_mul(i.id as u64);
                    let mut best_nonce = nonce;
                    let mut best_difficulty = 0;
                    let mut best_hash = Hash::default();
                    let mut memory = equix::SolverMemory::new();
                    // Return if core should not be used
                    if (i.id as u64).ge(&cores) {
                        return (0, 0, Hash::default());
                    }

                    // Pin to core
                    let _ = core_affinity::set_for_current(i);

                    // Create random challenge
                    let mut challenge = [0u8; 32];
                    for byte in challenge.iter_mut() {
                        *byte = thread_rng().gen();
                    }

                    // Start hashing
                    loop {
                        // Check if stop flag is set
                        if stop_flag.load(Ordering::Relaxed) {
                            break;
                        }
                        // Create hash
                        if let Ok(hx) = drillx::hash_with_memory(
                            &mut memory,
                            &challenge,
                            &nonce.to_le_bytes(),
                        ) {
                            let difficulty = hx.difficulty();
                            if difficulty.gt(&best_difficulty) {
                                best_nonce = nonce;
                                best_difficulty = difficulty;
                                best_hash = hx;
                            }
                            
                            progress_bar.set_message(format!(
                                "MIN_DIFFICULTY: {} > {} Mining...",
                                min_difficulty,
                                best_difficulty
                            ));

                            // Exit loop if difficulty meets or exceeds min_difficulty
                            if best_difficulty.ge(&min_difficulty) {
                                stop_flag.store(true, Ordering::Relaxed);
                                break;
                            }
                        }
                        nonce += 1;
                    }
                    // Return the best nonce
                    (best_nonce, best_difficulty, best_hash)
                })
            })
            .collect();

        // Join handles and return best nonce
        let mut best_nonce = 0;
        let mut best_difficulty = 0;
        let mut best_hash = Hash::default();
        for h in handles {
            if let Ok((nonce, difficulty, hash)) = h.join() {
                if difficulty > best_difficulty {
                    best_difficulty = difficulty;
                    best_nonce = nonce;
                    best_hash = hash;
                }
            }
        }

        // Update log
        progress_bar.finish_with_message(format!(
            "Best hash: {} (difficulty: {})",
            bs58::encode(best_hash.h).into_string(),
            best_difficulty
        ));

        let solution = Solution::new(best_hash.d, best_nonce.to_le_bytes());
        (solution, best_hash, best_difficulty)
    }

    pub fn check_num_cores(&self, cores: u64) {
        let num_cores = num_cpus::get() as u64;
        if cores.gt(&num_cores) {
            println!(
                "{} Cannot exceeds available cores ({})",
                "WARNING".bold().yellow(),
                num_cores
            );
        }
    }
}
