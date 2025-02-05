use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::{
    instruction::Instruction,
    pubkey::Pubkey,
    system_instruction::AccountMeta,
};
use solana_sdk::{
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

pub struct ProofState {
    pub challenge: [u8; 32],
    pub last_reset_at: i64,
    pub min_difficulty: u32,
    pub base_reward_rate: u64,
    pub top_balance: u64,
}

pub async fn get_config(client: &RpcClient) -> Result<ProofState, Box<dyn std::error::Error>> {
    Ok(ProofState {
        challenge: [0; 32],
        last_reset_at: 0,
        min_difficulty: 16,
        base_reward_rate: 1000,
        top_balance: 0,
    })
}

pub async fn get_updated_proof_with_authority(
    client: &RpcClient,
    authority: Pubkey,
    index: u64,
) -> Result<ProofState, Box<dyn std::error::Error>> {
    Ok(ProofState {
        challenge: [0; 32],
        last_reset_at: 0,
        min_difficulty: 16,
        base_reward_rate: 1000,
        top_balance: 0,
    })
}

pub fn create_mine_ix(
    miner: Pubkey,
    validator: Pubkey,
    bus: Pubkey,
    hash: Vec<u8>,
    nonce: Vec<u8>,
) -> Instruction {
    Instruction {
        program_id: Pubkey::from_str("ore11111111111111111111111111111111111111111").unwrap(),
        accounts: vec![
            AccountMeta::new(miner, false),
            AccountMeta::new(validator, true),
            AccountMeta::new(bus, false),
        ],
        data: [hash, nonce].concat(),
    }
}

pub async fn find_available_bus(client: &RpcClient) -> Result<Pubkey, Box<dyn std::error::Error>> {
    let bus_addresses = [
        "BUS1111111111111111111111111111111111111111",
        "BUS2222222222222222222222222222222222222222",
        "BUS3333333333333333333333333333333333333333",
    ];

    for addr in bus_addresses {
        let pubkey = Pubkey::from_str(addr)?;
        if let Ok(_) = client.get_account(&pubkey).await {
            return Ok(pubkey);
        }
    }

    Err("No available bus found".into())
}

pub async fn send_and_confirm_transaction(
    client: &RpcClient,
    mut tx: Transaction,
    signers: &[&dyn Signer],
) -> Result<Signature, Box<dyn std::error::Error>> {
    let blockhash = client.get_latest_blockhash().await?;
    tx.sign(signers, blockhash);
    let signature = client.send_and_confirm_transaction(&tx).await?;
    Ok(signature)
}

pub fn amount_u64_to_string(amount: u64) -> String {
    format!("{}", amount)
}

pub fn amount_f64_to_u64(amount: f64) -> u64 {
    (amount * 1_000_000_000.0) as u64
}

pub fn amount_f64_to_u64_v1(amount: f64) -> u64 {
    (amount * 1_000_000_000.0) as u64
}

pub fn ask_confirm(prompt: &str) -> bool {
    println!("{} [y/N]", prompt);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase() == "y"
}

pub fn proof_pubkey(index: u64) -> Pubkey {
    Pubkey::from_str(&format!("proof{}", index)).unwrap()
}

pub async fn get_proof(client: &RpcClient, index: u64) -> Result<ProofState, Box<dyn std::error::Error>> {
    Ok(ProofState {
        challenge: [0; 32],
        last_reset_at: 0,
        min_difficulty: 16,
        base_reward_rate: 1000,
        top_balance: 0,
    })
}

pub async fn get_proof_with_authority(
    client: &RpcClient,
    authority: Pubkey,
    index: u64,
) -> Result<ProofState, Box<dyn std::error::Error>> {
    Ok(ProofState {
        challenge: [0; 32],
        last_reset_at: 0,
        min_difficulty: 16,
        base_reward_rate: 1000,
        top_balance: 0,
    })
}
