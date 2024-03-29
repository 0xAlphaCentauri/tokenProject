use ethers::{
    core::types::Filter,
    prelude::Provider,
    providers::Ws,
    types::{Address, U256, U64},
    utils::format_units,
};
#[path = "addwebhook.rs"]
mod addwebhook;
use addwebhook::send_webhook;
use ethers_contract::{abigen, providers::Middleware};
use std::sync::Arc;
use chrono::{Local};

abigen!(
    UniswapFactory,
    r#"[
    event PairCreated(address indexed token0, address indexed token1,address indexed pair, uint)
    ]"#,
);

abigen!(
    UniswapPair,
    r#"[
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
    ]"#,
);

abigen!(
    ERC20,
    r#"[
    function name() public view virtual returns (string)
    function symbol() public view virtual returns (string)
    ]"#,
);
const UNISWAP_FACTORY: &str = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f";
const ONE_ETHER: u128 = 1000000000000000000;
const WETH_ADDRESS: &str = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
pub async fn pair_create_monitor(
    provider: Arc<Provider<Ws>>,
    block_number: U64,
) -> Result<(), Box<dyn std::error::Error>> {
    let address: Address = UNISWAP_FACTORY.parse().unwrap();
    let filter = Filter::new()
        .address(address)
        .event("PairCreated(address,address,address,uint256)")
        .from_block(block_number);
    let logs = provider.get_logs(&filter).await.unwrap();
    for log in logs.iter() {
        let token0 = Address::from(log.topics[1]);
        let token1 = Address::from(log.topics[2]);
        let weth_address: Address = WETH_ADDRESS.parse().unwrap();
        let pairadd = Address::from(&log.data[12..32].try_into()?);
        let pair_contract = UniswapPair::new(pairadd, provider.clone());
        if token0 == weth_address {
            let liq = pair_contract.get_reserves().await.unwrap();
            if liq.0 < ONE_ETHER {
                continue;
            }
            let liq_0 = format_units(U256::from(liq.0), "ether").unwrap();
            let token1_contract = ERC20::new(token1, provider.clone());
            let token1_symbol: String = token1_contract.symbol().await.unwrap();
            println!(
                "New Pair Detected {:?}/WETH at {:?} with {:?}ETH pooled ETH at {:?}",
                &token1_symbol, pairadd, liq_0, Local::now()
            );
            if send_webhook(token1_symbol.clone(), pairadd, liq_0.clone())
                .await
                .is_ok()
            {
                println!("Webhook sent");
            }
        } else {
            let liq = pair_contract.get_reserves().await.unwrap();
            let liq_1 = format_units(U256::from(liq.1), "ether").unwrap();
            if liq.1 < ONE_ETHER {
                continue;
            }
            let token0_contract = ERC20::new(token0, provider.clone());
            let token0_symbol: String = token0_contract.symbol().await.unwrap();
            println!(
                "New Pair Detected {:?}/WETH at {:?} with {:?}ETH pooled ETH @ {}",
                &token0_symbol, pairadd, liq_1, Local::now()
            );
            if send_webhook(token0_symbol.clone(), pairadd, liq_1.clone())
                .await
                .is_ok()
            {
                println!("Webhook sent");
            }
        }
    }
    Ok(())
}

// Need to trigger a contract event, send it to test net. Time from contract begin to execution of this how long it takes
// likely need to know pending blocks, see when liquidity is being added and use same gas to get on the same block
