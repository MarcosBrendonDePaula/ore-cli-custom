use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    time::{Duration, Instant},
};

use colored::*;
use drillx::{
    equix::{self},
    Hash, Solution,
};
use solana_rpc_client::spinner;
use solana_sdk::{
    signer::Signer,
    transaction::Transaction,
    pubkey::Pubkey,
    signature::Keypair,
};

use crate::{
    args::MineArgs,
    mining_history::{MiningHistory, MiningPattern},
    pool_client::PoolClient,
    utils,
    Miner,
};

impl Miner {
    pub async fn mine(&self, args: MineArgs) -> Result<(), Box<dyn std::error::Error>> {
        // Get signer
        let signer = self.signer();

        // Check num threads
        self.check_num_cores(args.cores);

        // Load mining history
        let mut history = MiningHistory::new();

        // Create and connect pool client
        let miner_arc = Arc::new(self.clone());
        let pool_client = Arc::new(PoolClient::new(
            args.pool_url.clone(),
            miner_arc,
        ));
        let pool_clone = pool_client.clone();
        tokio::spawn(async move {
            if let Err(e) = pool_clone.connect().await {
                println!("Failed to connect to pool: {}", e);
            }
        });

        // Start mining loop
        loop {
            // Fetch proof for challenge
            let proof = utils::get_updated_proof_with_authority(&self.rpc_client, signer.pubkey(), 0).await.unwrap();
            
            // Run drillx with time limit
            let (solution, hash, difficulty, nonce_range) = Self::find_hash_par(
                args.cores,
                args.min_difficulty,
                proof.challenge,
                args.time_limit,
            ).await;

            // Add successful pattern to history if difficulty meets target
            if difficulty >= args.min_difficulty {
                history.add_pattern(MiningPattern {
                    challenge: hash.h.to_vec(),
                    nonce_range,
                    difficulty,
                });

                // Submit hash to pool
                if let Err(e) = pool_client.submit_hash(
                    bs58::encode(hash.h).into_string(),
                    difficulty,
                    hex::encode(solution.n),
                ).await {
                    println!("Failed to submit hash to pool: {}", e);
                } else {
                    println!("Successfully submitted hash to pool");
                }
            }

            // Handle validation requests if we're the validator
            if let Some(validation_request) = pool_client.get_validation_request().await {
                println!("Validating hash: {}", validation_request.hash);
                
                // Create transaction
                let result = async {
                    let miner_pubkey = Pubkey::try_from(validation_request.minerAddress.as_str())?;
                    let bus = utils::find_available_bus(&self.rpc_client).await?;
                    let hash_data = bs58::decode(&validation_request.hash).into_vec()?;
                    let nonce_data = hex::decode(&validation_request.nonce)?;
                    
                    let tx = Transaction::new_with_payer(
                        &[utils::create_mine_ix(
                            miner_pubkey,
                            self.signer().pubkey(),
                            bus,
                            hash_data,
                            nonce_data,
                        )],
                        Some(&self.signer().pubkey()),
                    );

                    Ok::<Transaction, Box<dyn std::error::Error>>(tx)
                }.await;

                match result {
                    Ok(tx) => {
                        let signer = self.signer() as &dyn Signer;
                        match utils::send_and_confirm_transaction(&self.rpc_client, tx, &[signer]).await {
                            Ok(signature) => {
                                if let Err(e) = pool_client.submit_validation_result(
                                    validation_request.hashId,
                                    true,
                                    Some(signature.to_string()),
                                    None,
                                ).await {
                                    println!("Failed to submit validation result: {}", e);
                                }
                            }
                            Err(e) => {
                                if let Err(e) = pool_client.submit_validation_result(
                                    validation_request.hashId,
                                    false,
                                    None,
                                    Some(e.to_string()),
                                ).await {
                                    println!("Failed to submit validation result: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if let Err(e) = pool_client.submit_validation_result(
                            validation_request.hashId,
                            false,
                            None,
                            Some(e.to_string()),
                        ).await {
                            println!("Failed to submit validation result: {}", e);
                        }
                    }
                }
            }
        }
    }

    async fn find_hash_par(
        cores: u64,
        min_difficulty: u32,
        challenge: [u8; 32],
        time_limit: u64,
    ) -> (Solution, Hash, u32, (u64, u64)) {
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
                let challenge = challenge;
                let time_limit = time_limit;
                std::thread::spawn(move || {
                    let mut memory = equix::SolverMemory::new();
                    // Return if core should not be used
                    if (i.id as u64).ge(&cores) {
                        return (0u64, 0u32, Hash::default(), (0u64, 0u64));
                    }

                    // Pin to core
                    let _ = core_affinity::set_for_current(i);

                    // Initialize mining variables
                    let mut nonce = u64::MAX.saturating_div(cores).saturating_mul(i.id as u64);
                    let start_nonce = nonce;
                    let mut best_nonce = nonce;
                    let mut best_difficulty = 0;
                    let mut best_hash = Hash::default();

                    let start_time = Instant::now();

                    // Start hashing
                    loop {
                        // Check if stop flag is set or time limit reached
                        if stop_flag.load(Ordering::Relaxed) || start_time.elapsed() >= Duration::from_secs(time_limit) {
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
                            
                            // Update progress every 1000 hashes
                            if nonce % 1000 == 0 {
                                progress_bar.set_message(format!(
                                    "Current: {} | Best: {} | Target: {} | Time: {:?} | Mining...",
                                    difficulty,
                                    best_difficulty,
                                    min_difficulty,
                                    start_time.elapsed()
                                ));
                            }

                            // Exit loop if difficulty meets or exceeds min_difficulty
                            if best_difficulty.ge(&min_difficulty) {
                                stop_flag.store(true, Ordering::Relaxed);
                                break;
                            }
                        }
                        nonce += 1;
                    }
                    // Return the best result
                    (best_nonce, best_difficulty, best_hash, (start_nonce, best_nonce))
                })
            })
            .collect();

        // Join handles and return best nonce
        let mut best_result = (0u64, 0u32, Hash::default(), (0u64, 0u64));
        for h in handles {
            if let Ok(result) = h.join() {
                if result.1 > best_result.1 {
                    best_result = result;
                }
            }
        }

        // Update log
        progress_bar.finish_with_message(format!(
            "Best hash: {} (difficulty: {})",
            bs58::encode(best_result.2.h).into_string(),
            best_result.1
        ));

        let solution = Solution::new(best_result.2.d, best_result.0.to_le_bytes());
        (solution, best_result.2, best_result.1, best_result.3)
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
