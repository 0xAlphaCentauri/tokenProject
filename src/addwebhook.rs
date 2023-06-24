use dotenv::dotenv;
use ethers::types::Address;
use serde::Deserialize;
use serde_json::json;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct EtherscanResponse {
    pub result: ResultEtherscan,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct ResultEtherscan {
    pub ethusd: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoneypotResponse {
    pub simulation_success: bool,
    pub honeypot_result: HoneypotResult,
    pub simulation_result: SimulationResult,
    pub holder_analysis: HolderAnalysis,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoneypotResult {
    pub is_honeypot: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationResult {
    pub buy_tax: f64,
    pub sell_tax: f64,
    pub transfer_tax: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct HolderAnalysis {
    pub holders: String,
}

pub async fn send_webhook(
    token_name: String,
    pairadd: Address,
    pooled_ether: String,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let webhook = std::env::var("DISCORD_WEBHOOK").expect("We need a webhook to send the details");
    let etherscan_api =
        std::env::var("ETHERSCAN_API").expect("We need an Etherscan API to convert the price");
    let etherscan_url =
        format!("https://api.etherscan.io/api?module=stats&action=ethprice&apikey={etherscan_api}");
    let honeypot_url = format!(
        "https://api.honeypot.is/v2/IsHoneypot?address={:?}",
        pairadd
    );
    let client = reqwest::Client::new();
    let etherscan_api_call = reqwest::get(&etherscan_url).await?;
    let etherscan_response: EtherscanResponse = etherscan_api_call.json().await?;
    let honeypot_api_call = reqwest::get(honeypot_url).await?;
    let honeypot_response: HoneypotResponse = honeypot_api_call.json().await?;
    let pooled_in_usd = (etherscan_response.result.ethusd.parse::<f64>().unwrap()
        * pooled_ether.parse::<f64>().unwrap())
    .round();
    /* TO DO - add a filter if the honeypot api simulation is false also fix around the JSON to the
     * webhook to add more sites like etherscan dextools ETC*/
    let json = json!({
          "embeds":[{
              "color": 0x0099ff,
              "title": "New Pair Deployed",
              "fields": [
              {
                  "name": "Name",
                  "value" : format!("{}/WETH",token_name)
              },
              {
                  "name" : "DexScreener",
                  "value" : format!("[here](https://dexscreener.com/ethereum/{:?})",pairadd)
              },
              {
                  "name" : "Pooled Ether",
                  "value" : format!("{}ETH",pooled_ether)
              },
              {
                  "name" : "Eth Pooled in USD Value",
                  "value" : format!("${}",pooled_in_usd.to_string())

               },
              {
                  "name" : "Is a Honeypot?",
                  "value" : honeypot_response.honeypot_result.is_honeypot.to_string()

               },
              {
                  "name" : "BuyTax",
                  "value" : format!("{}%",honeypot_response.simulation_result.buy_tax.to_string()),
                  "inline" : true

               },
              {
                  "name" : "SellTax",
                  "value" : format!("{}%",honeypot_response.simulation_result.sell_tax.to_string()),
                  "inline" : true

               }
                  ]


              }]
      })
    .to_string();
    let response = client
        .post(&webhook)
        .header("Content-type", "application/json")
        .body(json.to_owned())
        .send()
        .await?;
    println!("{:?}",response);
    Ok(())
}
