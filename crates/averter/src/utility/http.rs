use std::fmt::{Display, Formatter};

use super::{handle_error, merge_json};
use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder, ResponseError};
use anyhow::Error;
use chrono::Utc;
use http::StatusCode;
use serde_json::{json, to_string_pretty, Value};

pub type Request = HttpRequest;
pub type Response = Result<HttpResponse, InternalServerError>;

#[derive(Debug)]
pub struct InternalServerError {}

impl Display for InternalServerError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "Internal Server Error")
	}
}

impl ResponseError for InternalServerError {
	fn status_code(&self) -> StatusCode {
		StatusCode::INTERNAL_SERVER_ERROR
	}

	fn error_response(&self) -> HttpResponse {
		HttpResponseBuilder::new(self.status_code()).json(json!({
			"message": self.status_code().canonical_reason().unwrap_or("Unknown"),
			"date": Utc::now().to_rfc3339()
		}))
	}
}

/// Returns a response with the given status code and body
fn respond(status_code: u16, mut body: Value, should_merge: bool, is_cached: bool) -> Response {
	let status = match StatusCode::from_u16(status_code) {
		Ok(status) => status,
		Err(err) => {
			let anyhow: Error = err.into();
			handle_error(&anyhow);
			return Err(InternalServerError {});
		}
	};

	if should_merge {
		body = merge_json(
			json!({
				"notice": {
					"api": env!("CANISTER_NOTICE_API"),
					"data": env!("CANISTER_NOTICE_DATA"),
					"migration": env!("CANISTER_NOTICE_MIGRATION").replace("{{docs}}", env!("CANISTER_DOCS_ENDPOINT")),
				},
				"message": format!("{status_code} {}", status.canonical_reason().unwrap_or("Unknown")),
				"date": Utc::now().to_rfc3339()
			}),
			body,
		);
	}

	let body = match to_string_pretty(&body) {
		Ok(body) => body,
		Err(err) => {
			let anyhow: Error = err.into();
			handle_error(&anyhow);
			return Err(InternalServerError {});
		}
	};

	Ok(HttpResponseBuilder::new(status)
		.content_type("application/json")
		.append_header((
			"Cache-Control",
			if is_cached {
				"public, max-age=3600"
			} else {
				"no-cache"
			},
		))
		.body(body))
}

/// Returns a response with the given status code and body
pub fn http_respond(status_code: u16, body: Value) -> Response {
	respond(status_code, body, false, false)
}

/// Returns a response with the given status code and body
/// The body is merged with a date and status message
pub fn api_respond(status_code: u16, is_cached: bool, body: Value) -> Response {
	respond(status_code, body, true, is_cached)
}

/// Returns a response with the given status code and error message
pub fn error_respond(status_code: u16, message: &str) -> Response {
	api_respond(status_code, false, json!({ "error": message }))
}
