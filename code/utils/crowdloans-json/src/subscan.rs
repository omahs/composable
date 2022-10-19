use std::{error::Error, fmt::Display, fs, num::ParseIntError, str::FromStr};

use reqwest::header::{HeaderName, HeaderValue};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_with::{serde_as, DeserializeFromStr, SerializeDisplay};

#[derive(Clone)]
pub(crate) struct Client {
	pub(crate) inner: reqwest::Client,
	api_key: String,
}

type BoxDynError = Box<dyn Error>;

impl Client {
	pub(crate) fn new(api_key: String) -> Result<Self, BoxDynError> {
		dbg!(&api_key);
		Ok(Self {
			inner: reqwest::Client::builder()
				.default_headers(
					[(HeaderName::from_str("X-API-KEY")?, HeaderValue::from_str(&api_key)?)]
						.into_iter()
						.collect(),
				)
				.build()?,
			api_key,
		})
	}

	pub(crate) async fn post<P: Serialize, R: for<'a> Deserialize<'a>>(
		self,
		endpoint: &str,
		payload: P,
	) -> Result<R, BoxDynError> {
		let value: Value = self.inner.post(endpoint).json(&payload).send().await?.json().await?;

		// println!("{}", value);

		serde_json::from_value(value).map_err(Into::into)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Response {
	pub(crate) data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "data")]
pub(crate) struct Data {
	#[serde(rename = "transfers")]
	pub(crate) transfers: Option<Vec<TransferData>>,
}

// #[serde_as]
#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct TransferData {
	#[serde(with = "rust_decimal::serde::str")]
	pub(crate) amount: Decimal,
	pub(crate) block_num: u64,
	pub(crate) block_timestamp: u64,
	pub(crate) extrinsic_index: ExtrinsicIndex,
	pub(crate) from: String,
	pub(crate) hash: String,
	pub(crate) success: bool,
	pub(crate) to: String,
}

#[derive(Debug, SerializeDisplay, DeserializeFromStr, Default)]
pub(crate) struct ExtrinsicIndex(pub(crate) [u64; 2]);

impl FromStr for ExtrinsicIndex {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.split('-')
			.map(str::parse)
			.collect::<Result<Vec<_>, ParseIntError>>()?
			.try_into()
			.map(Self)
			.map_err(|v| format!("too many elements: {v:#?}").into())
	}
}

impl Display for ExtrinsicIndex {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}-{}", self.0[0], self.0[1])
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) struct TransferPayload {
	#[serde(rename = "row")]
	pub(crate) row: u64,
	#[serde(rename = "page")]
	pub(crate) page: u64,
	#[serde(rename = "address", skip_serializing_if = "Option::is_none")]
	pub(crate) address: Option<String>,
	#[serde(rename = "extrinsic_index", skip_serializing_if = "Option::is_none")]
	pub(crate) extrinsic_index: Option<String>,
	#[serde(rename = "from_block", skip_serializing_if = "Option::is_none")]
	pub(crate) from_block: Option<u64>,
	#[serde(rename = "to_block", skip_serializing_if = "Option::is_none")]
	pub(crate) to_block: Option<u64>,
	// #[serde(rename = "direction")]
	// pub(crate) direction: TransferDirection,
	#[serde(rename = "include_total", skip_serializing_if = "Option::is_none")]
	pub(crate) include_total: Option<bool>,
	#[serde(rename = "asset_symbol", skip_serializing_if = "Option::is_none")]
	pub(crate) asset_symbol: Option<String>,
	// pub(crate) after_id: [u64; 2],
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TransferDirection {
	All,
	Sent,
	Recieved,
}

#[test]
fn test_serde() {
	let obj = Response {
		data: Data {
			transfers: [TransferData {
				amount: Default::default(),
				block_num: Default::default(),
				block_timestamp: Default::default(),
				extrinsic_index: ExtrinsicIndex(Default::default()),
				from: Default::default(),
				hash: Default::default(),
				success: Default::default(),
				to: Default::default(),
			}]
			.into(),
		},
	};

	println!("{}", serde_json::to_string_pretty(&obj).unwrap());
	// 	row: 1,
	// 	page: 1,
	// 	address: Some("F3opxRbN5ZZRfqouuW8F31XiS8JqQnwKmiZ3ZekRfoohppp".to_string()),
	// 	extrinsic_index: None,
	// 	include_total: Some(true),
	// 	asset_symbol: Some("KSM".to_string()),
	// 	// after_id: [1, 1]
	// }));
}

#[test]
fn biguint() {
	let arr = [
		"0.1",
		"0.1",
		"0.1",
		"0.1",
		"0.1",
		"0.1",
		"0.1",
		"0.1",
		"0.1",
		"0.1",
		"0.1",
		"2.5",
		"0.5",
		"0.1",
		"0.1",
		"0.1",
		"0.1",
		"1.12",
		"0.21",
		"3",
		"9.99484709",
		"1.02",
		"14.244713667166",
		"5",
		"1",
		"1",
		"1",
		"0.44",
		"2.377979414066",
		"0.324",
		"1",
		"2",
		"10",
		"1",
		"1",
		"0.1",
		"6.71250169",
		"1.01",
		"1",
		"1",
		"0.5",
		"0.1",
		"1.01",
		"0.113592010362",
		"1.01",
		"1.01",
		"0.289824001089",
		"1.01",
		"2.099979634199",
		"1.01",
		"1.01",
		"0.35",
		"1.01",
		"1",
		"1",
		"2.679",
		"0.1",
		"1.01295",
		"0.3",
		"10.1",
		"0.1",
		"1",
		"3",
		"0.45",
		"1",
		"1.51",
		"1",
		"1",
		"1",
		"1",
		"0.1",
		"1",
		"1",
		"1",
		"1",
		"1",
		"0.3",
		"0.4",
		"1",
		"1",
		"4.03695",
		"1",
		"7",
		"1",
		"1",
		"1",
		"1",
		"1",
		"1",
		"1",
		"1",
		"0.1",
		"0.51",
		"3",
		"0.319",
		"1",
		"0.10485217",
		"0.5",
		"1",
		"0.5",
	]
	.map(|x| {
		(
			x,
			json! ({
				"amount": x.to_string(),
				"block_num": 0,
				"block_timestamp": 0,
				"extrinsic_index": ExtrinsicIndex::default().to_string(),
				"from": String::default(),
				"hash": String::default(),
				"success": bool::default(),
				"to": String::default(),
			}),
		)
	});

	for value in serde_json::from_str::<Vec<Value>>(
		&fs::read_to_string("/home/benluelo/work/composable/code/out.txt").unwrap(),
	)
	.unwrap()
	{
		serde_json::from_value::<TransferData>(value.clone())
			.expect(&format!("{}", serde_json::to_string_pretty(&value).unwrap()));
	}
}
