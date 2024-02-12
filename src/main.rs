use ethers::{
    prelude::*,
    providers::{ Provider, Ws},
};
use eyre::Result;
use std::sync::Arc;
mod monitor_pair;
use monitor_pair::pair_create_monitor;
const WSS_URL : &str = "wss://mainnet.infura.io/ws/v3/8f9394402cb346ff99e49d7dd33dd36b";
// const WSS_URL : &str = "ws://192.168.0.45:7545";
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wss_provider = Provider::<Ws>::connect(WSS_URL)
        .await
        .expect("Need to connect to a WS");
    let rpc_url = "https://sepolia.infura.io/v3/8f9394402cb346ff99e49d7dd33dd36b";
    let provider = Arc::new(wss_provider);    
    // let http_provider = Provider::<Http>::connect(rpc_url).await?;

    let private_key = "your-private-key"; // Replace with your private key
    // let wallet: Wallet<_, _> = private_key.parse()?;
    // let sender_address: Address = wallet.address().await?;

    let contract_address: Address = "your-contract-address".parse()?; // Replace with your contract address

    
    let mut pendingStream = provider.subscribe_pending_txs().await.unwrap();
    // why completed blocks and not pending tx?
    let mut stream = provider.subscribe_blocks().await?;
    println!("connected");
    while let Some(block) = stream.next().await {
        match block.number {
            Some(blocknumber) => {
                println!("{:?}",blocknumber);
                pair_create_monitor(provider.clone(),blocknumber).await?
                
            }
            None  => {
                println!("error getting block");
            }
        }
    }
    Ok(())
}
