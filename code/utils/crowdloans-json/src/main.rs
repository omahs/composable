use aws_sdk_dynamodb::{
	model::{AttributeValue, Select},
	Client, Region,
};
use clap::Parser;
use futures::stream::TryStreamExt;
use serde_json::{json, Value};
use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};

use crate::subscan::{
	Data, ExtrinsicIndex, Response, TransferData, TransferDirection, TransferPayload,
};

mod subscan;

#[derive(Parser)]
struct Args {
	auction_11_ksm_price: f32,
	auction_12_ksm_price: f32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	do_main().await
}

async fn do_main() -> Result<(), Box<dyn Error>> {
	dotenvy::from_path("/home/benluelo/work/composable/code/utils/.crowdloans-json.env").unwrap();

	// parse_subscan_csv()?;

	// dyanmo_db_fetch_all().await?;

	let subscan_client = subscan::Client::new(
		dotenvy::var("SUBSCAN_API_KEY").expect("SUBSCAN_API_KEY environment variable not present"),
	)?;

	dbg!();

	let mut all_results = vec![];
	let mut last_ext_idx = [9790726, 1];
	// let mut last_ext_idx = [8980174, 3];

	loop {
		let result: Response = subscan_client
			.clone()
			.post(
				"https://kusama.api.subscan.io/api/v2/scan/transfers",
				json!({
					"row": 100,
					// "page": 10000,
					"address": "F3opxRbN5ZZRfqouuW8F31XiS8JqQnwKmiZ3ZekRfoohppp",
					// "direction": "sent",
					"after_id": last_ext_idx
					// "after_id": [9789012, 2]
				}),
			)
			.await?;

		let transfers = match result.data.transfers {
			Some(transfers) => transfers,
			None => break,
		};

		last_ext_idx = match transfers.last() {
			Some(td) => td.extrinsic_index.0,
			None => break,
		};

		println!("fetched {} transfers, {}", transfers.len(), ExtrinsicIndex(last_ext_idx));

		all_results.extend(transfers);
	}

	// println!("{}", serde_json::to_string_pretty(&result).unwrap());

	dbg!(all_results.len());

	Ok(())

	// get all auction 11 stakes
	// get all auction 12 stakes
	// get all stakes in both
	// - any of them that staked the same or more in auction 12 as 11 get 5% bonus

	// referrals
	//
}

fn parse_subscan_csv() -> Result<(), Box<dyn Error>> {
	let mut rdr = csv::Reader::from_path(
		"/home/benluelo/Downloads/F3opxRbN5ZZRfqouuW8F31XiS8JqQnwKmiZ3ZekRfoohppp.csv",
	)?;

	Ok(for result in rdr.deserialize() {
		// Notice that we need to provide a type hint for automatic
		// deserialization.
		let record: SubscanExport = result?;
		// println!("{:?}", record);
	})
}

async fn dyanmo_db_fetch_all() -> Result<(), Box<dyn Error>> {
	let shared_config = aws_config::load_from_env().await;

	let client = aws_sdk_dynamodb::Client::new(&shared_config);

	let items = client
		.scan()
		.table_name("composable-production-api-referral")
		.select(Select::AllAttributes)
		.into_paginator()
		.items()
		.send()
		.try_collect::<Vec<HashMap<String, AttributeValue>>>()
		.await?;
	dbg!(&items[0]);
	Ok(())
}

time::serde::format_description!(
	subscan_time_format,
	PrimitiveDateTime,
	"[year]-[month]-[day] [hour]:[minute]:[second]"
);

#[derive(Debug, Serialize, Deserialize)]
struct SubscanExport {
	#[serde(rename = "Extrinsic ID")]
	extrinsic_id: String,
	#[serde(rename = "Date", with = "subscan_time_format")]
	date: time::PrimitiveDateTime,
	#[serde(rename = "Block")]
	block: u64,
	#[serde(rename = "Hash")]
	hash: String,
	#[serde(rename = "Symbol")]
	symbol: String,
	#[serde(rename = "From")]
	from: String,
	#[serde(rename = "To")]
	to: String,
	#[serde(rename = "Value")]
	value: String,
	#[serde(rename = "Result")]
	result: bool,
}

enum Referrals {}

// struct EthReferrals {

//             "referralEnteredAt": {
//                 "N": "1635589846270"
//             },
//             "referralCode": {
//                 "S": "i9Nfo1"
//             },
//             "enteredReferralCodeOf": {
//                 "S": "0x9237F3FAF9b03e5bd54EccDc24EeD2b277807F24"
//             },
//             "walletAddress": {
//                 "S": "0x9AF6B6F2a118ff0A07690e0Ed2C1631608390CC6"
//             }
// }
