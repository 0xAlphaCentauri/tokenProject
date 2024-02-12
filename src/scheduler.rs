use web3::types::Address;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::Path;
use std::collections::HashMap;
use web3::transports::Http;
use web3::types::{Address, BlockId, Filter, Log, Transaction};
use web3::Web3;
use std::convert::TryInto;
use std::error::Error;

///////////////////// EXPECTATIONS //////////////////////////
fn main() {
    let accounts_list = Arc::new(Mutex::new(Vec::<Account>::new()));
    let account_index = Arc::new(Mutex::new(0));
    
    bnb_price();
    expectations(MY_BUY, EXTERNAL_BUY, RESERVE_IN, RESERVE_OUT);
    create_bee_book(&accounts_list, &account_index);
    let user_input = input("Press 'y' to check for seller book. Any other key to skip");
    if user_input == "y" {
        create_sellers_book(&accounts_list);
    }
    send_global_to_dark_forester();
    configure_trigger();
}

fn bnb_price() {    
    // Create a transport using the BSC node URL
    let http = Http::new("https://bsc-dataseed.binance.org/")?;
    let web3 = Web3::new(http);

    // Get the latest block number
    let block_number = web3.eth().block_number().await?;

    // Create a filter to get logs for the BNB/BUSD pair contract
    let filter = Filter {
        from_block: BlockId::Number(web3::types::BlockNumber::Number(
            block_number.try_into().unwrap(),
        )),
        to_block: BlockId::Latest,
        address: vec![Address::from_str(PANCAKESWAP_V2_BNB_BUSD_PAIR_ADDRESS)?],
        ..Default::default()
    };

    // Fetch logs that match the filter (this includes swap events)
    let logs = web3.eth().logs(filter).await?;

    // Extract the BNB price from the first log entry (assuming there's at least one swap)
    if let Some(log) = logs.first() {
        let bnb_reserve: f64 = f64::from(u128::from_be_bytes(log.topics[1].as_bytes()));
        let busd_reserve: f64 = f64::from(u128::from_be_bytes(log.topics[2].as_bytes()));
        let bnb_price = busd_reserve / bnb_reserve;
        Ok(bnb_price)
    } else {
        Err("No swap logs found for the BNB/BUSD pair")?
    }
}

fn quote(amin: f64, reserve_in: f64, reserve_out: f64) -> f64 {
    if reserve_in == 0.0 && reserve_out == 0.0 {
        return 0.0;
    }
    let amount_in_with_fee = amin * 998.0;
    let num = amount_in_with_fee * reserve_out;
    let den = reserve_in * 1000.0 + amount_in_with_fee;
    (num / den).round()
}

fn expectations(my_buy: f64, external_buy: f64, reserve_in: f64, reserve_out: f64) {
    let bnb_price = 0.0; // Placeholder for the BNB price

    println!(
        "--> If the liq added is {} BNB / {} tokens and I want to buy with {} BNB:",
        reserve_in, reserve_out, my_buy
    );
    for i in 1..=30 {
        let (bought_tokens, price_per_token, add_in) = expectations_helper(
            my_buy,
            external_buy,
            reserve_in,
            reserve_out,
            i,
        );

        if BASE_ASSET == "BNB" {
            println!(
                "amount bought: {} | {:.5} BNB/tkn | {:.7} $/tkn | , capital entered before me: {} BNB",
                bought_tokens,
                price_per_token,
                price_per_token * bnb_price,
                add_in
            );
        } else {
            println!(
                "amount bought: {} | {:.5} BNB/tkn| , capital entered before me: {} BNB",
                bought_tokens, price_per_token, add_in
            );
        }
    }
    println!("--> BNB price: {} $", bnb_price);
    println!("WARNING: Exit and restart brownie to be sure variable corrections are taken into account!\n");

    input("Press any key to continue, or ctrl+c to stop and try other expectation parameters");
}

fn expectations_helper(
    my_buy: f64,
    external_buy: f64,
    reserve_in: f64,
    reserve_out: f64,
    queue_number: i32,
) -> (f64, f64, f64) {
    let mut i = 1;
    let mut add_in = 0.0;
    let mut sub_out = 0.0;

    while i < queue_number {
        let amount = quote(external_buy, reserve_in + add_in, reserve_out - sub_out);
        add_in += external_buy;
        sub_out += amount;
        i += 1;
    }
    let bought_tokens = quote(my_buy, reserve_in + add_in, reserve_out - sub_out);
    let price_per_token = my_buy / bought_tokens;
    (bought_tokens, price_per_token, add_in)
}

//////////////////////// SWARMER //////////////////////////////
struct Account {
    idx: i32,
    address: Address,
    pk: String,
}

fn create_temp_address_book(temp_path: &str) {
    // TODO: Create the temporary CSV file that stores addresses
}

fn save_address_book(temp_path: &str, path: &str) {
    println!("---> Saving address book...");
    // TODO: Implement the saving of the address book data to a file
    println!("Done!");
}

fn create_account(accounts_list: &Arc<Mutex<Vec<Account>>>, account_index: &Arc<Mutex<i32>>) -> Account {
    let mut index = account_index.lock().unwrap();
    let new_account = web3::types::Address::random();
    let account = Account {
        idx:
