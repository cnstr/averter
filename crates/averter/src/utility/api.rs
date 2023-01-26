use super::{canister, error_respond, handle_error};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{to_value, Value};
use surf::StatusCode;
use tide::Result as TideResult;

#[derive(Serialize, Deserialize)]
struct HTTPError {
	message: String,
	date: String,
	error: String,
}

/// Merges two JSON objects together in the order of left, right
/// If the object is a strictly-typed struct, it is serialized into a Value
pub fn merge_json<L: Serialize, R: Serialize>(left: L, right: R) -> Value {
	let mut left = match to_value(left) {
		Ok(value) => value,
		Err(err) => {
			handle_error(&err.into());
			return Value::Null;
		}
	};

	let right = match to_value(right) {
		Ok(value) => value,
		Err(err) => {
			handle_error(&err.into());
			return Value::Null;
		}
	};

	merge_json_value(&mut left, right);
	left
}

// Modified to be a bit more readable and concise
// Original: https://stackoverflow.com/questions/47070876/how-can-i-merge-two-json-objects-with-rust
fn merge_json_value(left: &mut Value, right: Value) {
	match (left, right) {
		(&mut Value::Object(ref mut left), Value::Object(right)) => {
			for (key, value) in right {
				merge_json_value(left.entry(key).or_insert(Value::Null), value);
			}
		}

		(left, right) => *left = right,
	}
}

/// Fetches data from the Canister v2 API
/// This function serializes the responses into strict types
pub async fn fetch_v2<Q: Serialize, R: DeserializeOwned>(
	query: Q,
	url: &str,
) -> Result<R, TideResult> {
	let request = match canister().get(url).query(&query) {
		Ok(request) => request,
		Err(err) => {
			handle_error(&err.into_inner());
			return Err(error_respond(500, "Failed to create Canister query"));
		}
	};

	let mut response = match canister().send(request).await {
		Ok(response) => response,
		Err(err) => {
			handle_error(&err.into_inner());
			return Err(error_respond(500, "Failed to execute Canister query"));
		}
	};

	if cfg!(debug_assertions) {
		println!("v2 -> {} {}", response.status(), url);
	}

	match response.status() {
		StatusCode::Ok => {
			let response: R = match response.body_json().await {
				Ok(response) => response,
				Err(err) => {
					handle_error(&err.into_inner());
					return Err(error_respond(500, "Failed to parse Canister response"));
				}
			};

			Ok(response)
		}

		StatusCode::BadRequest | StatusCode::NotFound => {
			let response: HTTPError = match response.body_json().await {
				Ok(response) => response,
				Err(err) => {
					handle_error(&err.into_inner());
					return Err(error_respond(500, "Failed to parse Canister response"));
				}
			};

			Err(error_respond(400, &response.error))
		}

		_ => Err(error_respond(500, "Failed to fetch from Canister")),
	}
}
