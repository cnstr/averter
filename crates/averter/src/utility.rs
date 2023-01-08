use lazy_static::lazy_static;
use reqwest::StatusCode as RStatusCode;
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
) -> Result<R, Response> {
	let mut url = Url::parse(&format!("{}{path}", env!("CANISTER_API_ENDPOINT"))).unwrap();

	for (key, value) in query {
		url.query_pairs_mut().append_pair(key, value);
	}

	tokio_run(async {
		let res = match reqwest::get(url).await {
			Ok(res) => res,
			Err(err) => {
				println!("Error: {err}");

				return Err(json_respond(
					StatusCode::InternalServerError,
					json!({
						"notice": api_notice(),
						"status": "500 Internal Server Error",
						"error": "Failed to fetch data from Canister 2",
						"date": chrono::Utc::now().to_rfc3339(),
					}),
				));
			}
		};

		if cfg!(debug_assertions) {
			println!("v2 -> {} {}", res.status(), res.url());
		}

		match res.status() {
			RStatusCode::OK => {
				let text = res.text().await.unwrap();
				let value: R = from_str(&text).unwrap();
				Ok(value)
			}

			RStatusCode::BAD_REQUEST => {
				let text = res.text().await.unwrap();
				let value: Value = from_str(&text).unwrap();
				Err(json_respond(StatusCode::BadRequest, value))
			}

			RStatusCode::NOT_FOUND => {
				let text = res.text().await.unwrap();
				let value: Value = from_str(&text).unwrap();
				Err(json_respond(StatusCode::NotFound, value))
			}

			_ => Err(json_respond(
				StatusCode::InternalServerError,
				json!({
					"notice": api_notice(),
					"status": "500 Internal Server Error",
					"error": "Failed to fetch data from Canister 2",
					"date": chrono::Utc::now().to_rfc3339(),
				}),
			)),
		}
	})
}

#[must_use]
pub fn api_notice() -> Value {
	json!({
		"api": env!("CANISTER_NOTICE_API"),
		"data": env!("CANISTER_NOTICE_DATA"),
		"migration": env!("CANISTER_NOTICE_MIGRATION").replace("{{docs}}", env!("CANISTER_DOCS_ENDPOINT")),
	})
}

#[must_use]
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
	RUNTIME.block_on(future)
}
