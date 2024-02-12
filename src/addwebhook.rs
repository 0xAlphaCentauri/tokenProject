use dotenv::dotenv;
use ethers::types::Address;
use serde::Deserialize;
use serde_json::json;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtherscanResponse {
    pub result: ResultEtherscan,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultEtherscan {
    pub ethusd: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoneypotResponse {
    pub simulation_success: bool,
    pub honeypot_result: Option<HoneypotResult>,
    pub simulation_result: Option<SimulationResult>,
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

pub fn prettify_decimal(str: String) -> String {
    if str == "0" || str.len() == 1 {
        return str;
    } else {
        let idx = str.chars().position(|c| c == '.').unwrap_or(0);
        return str.chars().take(idx + 3).collect::<String>();
    }
}
pub fn prettify_dollars(str: String) -> String {
    let mut s = String::new();
    if str.len() == 3 {
        return str;
    } else {
        let a = str.chars().rev().enumerate();
        for (idx, val) in a {
            if idx != 0 && idx % 3 == 0 {
                s.insert(0, ' ');
            }
            s.insert(0, val);
        }
        s
    }
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
    let honeypot_api_call: HoneypotResponse = reqwest::get(honeypot_url).await?.json().await?;
    if !honeypot_api_call.simulation_success
        || honeypot_api_call
            .honeypot_result
            .as_ref()
            .unwrap()
            .is_honeypot
    {
        Err(
            "Simulation Failed don't even bother sending the hook because it doesn't make sense"
                .try_into()?,
        )
    } else {
        let etherscan_api_call = reqwest::get(&etherscan_url).await?;
        let etherscan_response: EtherscanResponse = etherscan_api_call.json().await?;
        let pooled_in_usd = (etherscan_response.result.ethusd.parse::<f64>().unwrap()
            * pooled_ether.parse::<f64>().unwrap())
        .round();
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
                    "name" : "Etherscan",
                    "value" : format!("[here](https://etherscan.com/address/{:?})",pairadd),
                    "inline" : true
                },
                {
                    "name" : "DexScreener",
                    "value" : format!("[here](https://dexscreener.com/ethereum/{:?})",pairadd),
                    "inline" : true
                },
                {
                    "name" : "Pooled Ether",
                    "value" : format!("{}ETH",prettify_decimal(pooled_ether))
                },
                {
                    "name" : "Eth Pooled in USD Value",
                    "value" : format!("${}",prettify_dollars(pooled_in_usd.to_string()))

                 },
                {
                    "name" : "Is a Honeypot?",
                    "value" : honeypot_api_call.honeypot_result.unwrap().is_honeypot.to_string()

                 },
                {
                    "name" : "BuyTax",
                    "value" : format!("{}%",prettify_decimal(honeypot_api_call.simulation_result.as_ref().unwrap().buy_tax.to_string())),
                    "inline" : true

                 },
                {
                    "name" : "SellTax",
                    "value" : format!("{}%",prettify_decimal(honeypot_api_call.simulation_result.as_ref().unwrap().sell_tax.to_string())),
                    "inline" : true

                 }
                    ]


          }]
        })
        .to_string();
        println!("Sending {:?}", json);
        let response = reqwest::Client::new()
            .post(&webhook)
            .header("Content-type", "application/json")
            .body(json.to_owned())
            .send()
            .await?;
        println!("{:?}", response.status());
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_prettify() {
    assert_eq!(
        prettify_decimal(0.9999999999999832.to_string()),
        "0.99".to_string()
    );
    assert_eq!(
        prettify_decimal(7.461748091112812.to_string()),
        "7.46".to_string()
    );
    assert_eq!(prettify_decimal(0.to_string()), "0".to_string());
    assert_eq!(
        prettify_decimal(2.001240998420113656.to_string()),
        "2.00".to_string()
    );
}

#[test]
fn test_prettify_dollars() {
    assert_eq!(prettify_dollars("19123".to_string()), "19 123".to_string());
    assert_eq!(prettify_dollars(696.to_string()), 696.to_string());
    assert_eq!(prettify_dollars(1234.to_string()), "1 234".to_string());
}
