use crate::utility::{api_notice, fetch_v2, json_respond};
use serde_json::{json, Value};
use std::collections::HashMap;
use tide::{
	prelude::Deserialize,
	Request, Result,
	StatusCode::{BadRequest, Ok as OK, UnprocessableEntity},
};

#[derive(Deserialize)]
struct Query {
	query: Option<String>,
	queries: Option<String>,
}

#[derive(Deserialize)]
struct Data {
	uri: String,
	safe: bool,
}

#[derive(Deserialize)]
struct Response {
	date: String,
	count: u32,
	data: Vec<Data>,
}

pub async fn repository_safety(req: Request<()>) -> Result {
	let (uris, is_single) = match req.query::<Query>() {
		Ok(query) => match query.queries {
			Some(queries) => (queries, false),
			None => match query.query {
				Some(query) => (query, true),
				None => {
					return Ok(json_respond(
						BadRequest,
						json!({
							"status": "400 Bad Request",
							"error": "Missing query parameters",
							"date": chrono::Utc::now().to_rfc3339(),
						}),
					));
				}
			},
		},

		Err(err) => {
			println!("Error: {err}");
			return Ok(json_respond(
				UnprocessableEntity,
				json!({
					"status": "422 Unprocessable Entity",
					"error": "Malformed query parameters",
					"date": chrono::Utc::now().to_rfc3339(),
				}),
			));
		}
	};

	let query = HashMap::from([("uris", uris.as_str())]);
	let mut response = match fetch_v2::<Response>("/jailbreak/repository/safety", query).await {
		Ok(response) => response,
		Err(err) => return Ok(err),
	};

	match is_single && response.count == 1 {
		true => Ok(json_respond(
			OK,
			json!({
				"status": "Successful",
				"date": response.date,
				"data": match response.data[0].safe {
					true => "safe",
					false => "unsafe",
				}
			}),
		)),

		false => {
			let data = response.data.iter_mut();
			let data = data
				.map(|item| {
					let safety = json!({
						"repositoryURI": item.uri,
						"status": match item.safe {
							true => "safe",
							false => "unsafe",
						}
					});

					safety
				})
				.collect::<Vec<Value>>();

			Ok(json_respond(
				OK,
				json!({
					"notice": api_notice(),
					"status": "Successful",
					"date": response.date,
					"data": data
				}),
			))
		}
	}
}
