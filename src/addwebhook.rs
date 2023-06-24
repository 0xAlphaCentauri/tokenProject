use std::str::FromStr;

use ethers::types::{Address, U256};
use dotenv::dotenv;
use serde_json::json;
use serde::Deserialize;


#[derive(Default, Debug, Clone, PartialEq,Deserialize)]
pub struct Root {
    pub status: String,
    pub message: String,
    pub result: ResultEtherscan,
}

#[derive(Default, Debug, Clone, PartialEq,Deserialize)]
pub struct ResultEtherscan {
    pub ethbtc: String,
    pub ethbtc_timestamp: String,
    pub ethusd: String,
    pub ethusd_timestamp: String,
}

pub async fn send_webhook(
    token_name: String,
    pairadd: Address,
    pooled_ether: String,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let webhook = std::env::var("DISCORD_WEBHOOK").expect("We need a webhook to send the details");
    let etherscan_api = std::env::var("ETHERSCAN_API").expect("We need an Etherscan API to convert the price");
    let etherscan_url = format!("https://api.etherscan.io/api?module=stats&action=ethprice&apikey={etherscan_api}");
    let client = reqwest::Client::new();
    let etherscan_api_call = reqwest::get(&etherscan_url).await?;
    let etherscan_response : Root = etherscan_api_call.json().await?;
    let pooled_in_usd = (etherscan_response.result.ethusd.parse::<f64>().unwrap() * pooled_ether.parse::<f64>().unwrap()).round() ; 
        let json = json!({
        "embeds":[{
            "title":"New Pair Deployed",
            "fields": [
                {
                    "name": "Name",
                    "value" : format!("{}/WETH",token_name),
                },
                {
                    "name" : "Address",
                    "value" : format!("https://dexscreener.com/ethereum/{:?}",pairadd),
                },
                {
                    "name" : "Pooled Ether",
                    "value" : format!("{}ETH",pooled_ether),
                },
                {
                    "name" : "Eth Pooled in USD Value",
                    "value" : format!("${}",pooled_in_usd.to_string()), 

                },
            ]


        }]
    })
    .to_string();
    let _response = client
        .post(&webhook)
        .header("Content-type", "application/json")
        .body(json.to_owned())
        .send()
        .await;
    Ok(())
}


