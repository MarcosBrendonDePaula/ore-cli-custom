use std::sync::{Arc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use solana_sdk::signer::Signer;

#[derive(Debug, Serialize)]
struct HashSubmission {
    #[serde(rename = "type")]
    msg_type: String,
    hash: String,
    difficulty: u32,
    minerAddress: String,
    nonce: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ValidationRequest {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub hashId: String,
    pub hash: String,
    pub difficulty: u32,
    pub minerAddress: String,
    pub nonce: String,
}

#[derive(Debug, Serialize)]
struct ValidationResult {
    #[serde(rename = "type")]
    msg_type: String,
    hashId: String,
    success: bool,
    signature: Option<String>,
    error: Option<String>,
}

pub struct PoolClient {
    ws_url: String,
    miner: Arc<crate::Miner>,
    validation_request: Arc<Mutex<Option<ValidationRequest>>>,
}

impl PoolClient {
    pub fn new(ws_url: String, miner: Arc<crate::Miner>) -> Self {
        Self {
            ws_url,
            miner,
            validation_request: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (ws_stream, _) = connect_async(&self.ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Register with the pool
        let register_msg = serde_json::json!({
            "type": "register",
            "address": self.miner.signer().pubkey().to_string()
        });
        write.send(Message::Text(register_msg.to_string())).await?;

        // Handle incoming messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let data: serde_json::Value = serde_json::from_str(&text)?;
                    match data["type"].as_str() {
                        Some("validate_hash") => {
                            if let Ok(req) = serde_json::from_value::<ValidationRequest>(data) {
                                *self.validation_request.lock().unwrap() = Some(req);
                            }
                        },
                        Some("hash_confirmed") => {
                            println!("Hash confirmed with signature: {}", 
                                data["signature"].as_str().unwrap_or_default());
                        },
                        Some("hash_rejected") => {
                            println!("Hash rejected: {}", 
                                data["error"].as_str().unwrap_or_default());
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub async fn submit_hash(&self, hash: String, difficulty: u32, nonce: String) -> Result<(), Box<dyn std::error::Error>> {
        let (mut ws_stream, _) = connect_async(&self.ws_url).await?;
        
        let submission = HashSubmission {
            msg_type: "submit_hash".to_string(),
            hash,
            difficulty,
            minerAddress: self.miner.signer().pubkey().to_string(),
            nonce,
        };

        ws_stream.send(Message::Text(serde_json::to_string(&submission)?)).await?;
        Ok(())
    }

    pub async fn get_validation_request(&self) -> Option<ValidationRequest> {
        self.validation_request.lock().unwrap().clone()
    }

    pub async fn submit_validation_result(
        &self,
        hash_id: String,
        success: bool,
        signature: Option<String>,
        error: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (mut ws_stream, _) = connect_async(&self.ws_url).await?;
        
        let result = ValidationResult {
            msg_type: "validation_result".to_string(),
            hashId: hash_id,
            success,
            signature,
            error,
        };

        ws_stream.send(Message::Text(serde_json::to_string(&result)?)).await?;
        Ok(())
    }
}
