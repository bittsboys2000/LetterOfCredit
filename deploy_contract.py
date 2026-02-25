import requests
import json
import os

RPC_URL = "http://localhost:8545"
CONTRACT_PATH = "contracts/build/LCRegistry.bin"
ABI_PATH = "contracts/build/LCRegistry.abi"

def get_bytecode():
    with open(CONTRACT_PATH, 'r') as f:
        return f.read().strip()

def get_abi():
    with open(ABI_PATH, 'r') as f:
        return json.load(f)

def get_accounts():
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_accounts",
        "params": [],
        "id": 1
    }
    response = requests.post(RPC_URL, json=payload)
    return response.json()['result']

def deploy():
    private_key = "0x8f2a55949038a9610f50fb23b5883af3b4ecb3c3bb792cbcefbd1542c692be63"
    sender = "0xfe3b557e8fb62b89f4916b721be55ceb828dbd73"
    print(f"Deploying from account: {sender}")

    bytecode = get_bytecode()
    if not bytecode.startswith("0x"):
        bytecode = "0x" + bytecode

    # Get nonce
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_getTransactionCount",
        "params": [sender, "latest"],
        "id": 1
    }
    response = requests.post(RPC_URL, json=payload)
    nonce = int(response.json()['result'], 16)

    # Build transaction
    tx = {
        "nonce": nonce,
        "gasPrice": 0, # Free gas in dev mode usually, or set a value
        "gas": 4700000,
        "data": bytecode,
        "chainId": 1337 # Besu dev chain ID is usually 1337 or 2018. Let's try 1337.
    }
    
    # We need to sign the transaction. 
    # Since we don't have web3 installed, we can't easily sign in Python without a library.
    # But wait, I can use `ethers-rs` to deploy since I have the key now!
    # Creating a Rust script is safer than trying to implement RLP encoding and signing in Python from scratch.
    print("Switching to Rust deployment...")

if __name__ == "__main__":
    deploy()
