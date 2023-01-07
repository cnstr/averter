use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, ser::PrettyFormatter, Serializer, Value};
use std::{collections::HashMap, future::Future};
use tide::{Response, StatusCode};
use tokio::runtime::{Builder, Runtime};
use url::Url;

pub fn json_stringify(value: Value) -> String {
	let buffer = Vec::new();
	let formatter = PrettyFormatter::with_indent(b"    ");
	let mut serialized = Serializer::with_formatter(buffer, formatter);

	value.serialize(&mut serialized).unwrap();
	String::from_utf8(serialized.into_inner()).unwrap()
}

pub async fn fetch_v2<R: for<'a> Deserialize<'a>>(
	path: &str,
	query: HashMap<&str, &str>,
) -> Result<R, String> {
	let mut url = Url::parse(&format!("{}{}", env!("CANISTER_API_ENDPOINT"), path)).unwrap();

	for (key, value) in query {
		url.query_pairs_mut().append_pair(key, value);
	}

	return tokio_run(async {
		let res = match reqwest::get(url).await {
			Ok(res) => res,
			Err(err) => return Err(err.to_string()),
		};

		match res.status() {
			reqwest::StatusCode::OK => {
				let text = res.text().await.unwrap();
				let value: R = from_str(&text).unwrap();
				return Ok(value);
			}
			_ => {
				return Err("Failed to deserialize response".to_owned());
			}
		};
	});
}

pub fn api_notice() -> Value {
	return json!({
		"api": env!("CANISTER_NOTICE_API"),
		"data": env!("CANISTER_NOTICE_DATA"),
		"migration": env!("CANISTER_NOTICE_MIGRATION").replace("{{docs}}", env!("CANISTER_DOCS_ENDPOINT")),
	});
}

pub fn json_respond(status: StatusCode, value: Value) -> Response {
	Response::builder(status)
		.header("Content-Type", "application/json")
		.body(json_stringify(value))
		.build()
}

lazy_static! {
	static ref RUNTIME: Runtime = Builder::new_multi_thread().enable_all().build().unwrap();
}

pub fn tokio_run<F: Future>(future: F) -> <F as Future>::Output {
	return RUNTIME.block_on(future);
}
