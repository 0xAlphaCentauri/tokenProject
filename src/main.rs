use ethers::{
    prelude::*,
    providers::{ Provider, Ws},
};
use eyre::Result;
use std::sync::Arc;
mod monitor_pair;
use monitor_pair::pair_create_monitor;
const WSS_URL : &str = "ws://10.234.32.252:8546";
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wss_provider = Provider::<Ws>::connect(WSS_URL)
        .await
        .expect("Need to connect to a WS");
    let provider = Arc::new(wss_provider);
    let mut stream = provider.subscribe_blocks().await?;
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
