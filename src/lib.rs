use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;

pub struct Miner {
    pub rpc_client: RpcClient,
    keypair: Keypair,
}

impl Miner {
    pub fn new(rpc_client: RpcClient, keypair: Keypair) -> Self {
        Self {
            rpc_client,
            keypair,
        }
    }

    pub fn signer(&self) -> &Keypair {
        &self.keypair
    }
}
