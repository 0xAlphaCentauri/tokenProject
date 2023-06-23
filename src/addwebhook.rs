use ethers::types::Address;
use dotenv::dotenv;
use serde_json::json;
pub async fn send_webhook(
    token_name: String,
    pairadd: Address,
    pooled_ether: String,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let webhook = std::env::var("DISCORD_WEBHOOK").expect("We need a webhook to start");
    let client = reqwest::Client::new();
    let json = json!({
        "embeds":[{
            "title":"New Pair Deployed",
            "fields": [
                {
                    "name": "Name",
                    "value" : format!("{}/WETH)",token_name),
                },
                {
                    "name" : "Address",
                    "value" : format!("https://dexscreener.com/ethereum/{:?}",pairadd),
                },
                {
                    "name" : "Pooled Ether",
                    "value" : format!("{}ETH",pooled_ether),
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
