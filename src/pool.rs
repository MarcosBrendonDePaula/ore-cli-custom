use reqwest::Client;
use serde::Serialize;
use solana_sdk::pubkey::Pubkey;

const POOL_API_URL: &str = "http://localhost:3000/api/hashes";

#[derive(Serialize)]
struct HashSubmission {
    hash: String,
    difficulty: u32,
    miner_address: String,
}

pub async fn submit_hash_to_pool(hash: String, difficulty: u32, miner_address: Pubkey) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let submission = serde_json::json!({
        "hash": hash,
        "difficulty": difficulty,
        "minerAddress": miner_address.to_string(),
    });

    let response = client
        .post(POOL_API_URL)
        .header("Content-Type", "application/json")
        .json(&submission)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Failed to submit hash to pool: {}", response.status()).into());
    }

    Ok(())
}
