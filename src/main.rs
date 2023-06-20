use ethers::{
    //  contract::Abigen,
    core::types::{Address,Filter},
    prelude::*,
    providers::{Provider, Http},
};
use eyre::Result;
use std::sync::Arc;
abigen!(
    UniswapFactory,
    r#"[
    event PairCreated(address indexed token0, address indexed token1,address indexed pair, uint)
    ]"#,
    );
const HTTP_URL: &str = "http://10.234.32.252:8545";
const UNISWAP_FACTORY: &str = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f";
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(HTTP_URL)?;
    let provider = Arc::new(provider);
    let address: Address = UNISWAP_FACTORY.parse()?;
    let filter = Filter::new().address(address).event("PairCreated(address,address,address,uint256)").from_block(17519823);
    let logs = provider.get_logs(&filter).await?;
    for log in logs.iter() {
        println!("{:?}",log);
    }
    Ok(())
}
