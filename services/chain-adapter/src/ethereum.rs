use ethers::{
    prelude::*,
    providers::{Provider, Http},
    contract::abigen,
};
use std::sync::Arc;
use common::error::AppError;
use std::convert::TryFrom;

// Generate type-safe contract bindings
abigen!(
    LCRegistry,
    r#"[
        function updateLCState(string memory _lcId, string memory _state, string memory _docHash) public
        function getLCState(string memory _lcId) public view returns (string memory, string memory, address, uint256)
        event StateUpdated(string indexed lcId, string state, string documentHash, address indexed actor, uint256 timestamp)
    ]"#,
);

pub struct EthereumClient {
    contract: LCRegistry<Provider<Http>>,
}

impl EthereumClient {
    pub fn new(rpc_url: String, contract_address: String) -> Self {
        let provider = Provider::<Http>::try_from(rpc_url).expect("Invalid RPC URL");
        let client = Arc::new(provider);
        let address: Address = contract_address.parse().expect("Invalid Contract Address");
        
        let contract = LCRegistry::new(address, client);
        
        Self {
            contract,
        }
    }

    pub async fn submit_hash(&self, lc_id: String, state: String, hash: String) -> Result<String, AppError> {
        // In a real app, we need a signer (Wallet) to send transactions.
        // For this demo, we are using the provider which might fail if the node doesn't manage keys.
        // Assuming Besu dev mode with unlocked accounts or we'd need to add LocalWallet here.
        
        // Note: ethers-rs contract calls usually require a signer middleware for state-changing txs.
        // Since we only have a Provider<Http>, this call will try to use `eth_sendTransaction` 
        // which requires the node to hold the key.
        
        let call = self.contract.update_lc_state(lc_id, state, hash);
        let pending_tx = call.send().await
            .map_err(|e| AppError::Internal(format!("Contract error: {}", e)))?;
            
        let tx_hash = pending_tx.tx_hash();
        Ok(format!("{:?}", tx_hash))
    }
}
