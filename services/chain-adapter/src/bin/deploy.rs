use ethers::abi::Abi;
use ethers::prelude::*;
use std::convert::TryFrom;
use std::fs;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connect to the provider
    let provider = Provider::<Http>::try_from("http://localhost:8545")?;

    // 2. Create a wallet
    let wallet: LocalWallet = "0x8f2a55949038a9610f50fb23b5883af3b4ecb3c3bb792cbcefbd1542c692be63"
        .parse::<LocalWallet>()?
        .with_chain_id(1337u64);

    // 3. Create the client
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    // 4. Read ABI and Bytecode
    let abi_path = "../../contracts/build/LCRegistry.abi";
    let bin_path = "../../contracts/build/LCRegistry.bin";

    let abi_content = fs::read_to_string(abi_path)
        .map_err(|e| format!("Failed to read ABI from {}: {}", abi_path, e))?;
    let abi: Abi = serde_json::from_str(&abi_content)?;

    let bin_content = fs::read_to_string(bin_path)
        .map_err(|e| format!("Failed to read Bytecode from {}: {}", bin_path, e))?;
    let bytecode = hex::decode(bin_content.trim())?;
    let bytecode = Bytes::from(bytecode);

    // 5. Create Contract Factory
    let factory = ContractFactory::new(abi, bytecode, client.clone());

    // 6. Deploy
    println!("Deploying contract...");
    let contract = factory.deploy(())?.send().await?;

    println!("Contract deployed at: {:?}", contract.address());

    Ok(())
}
