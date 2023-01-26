use crate::utility::{api_respond, error_respond, fetch_v2};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tide::{Request, Result};

#[derive(Serialize, Deserialize)]
struct Query {
	query: Option<String>,
	queries: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct Data {
	uri: String,
	safe: bool,
}

#[derive(Serialize, Deserialize)]
struct CanisterQuery {
	uris: String,
}

#[derive(Serialize, Deserialize)]
struct CanisterResponse {
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
					return error_respond(400, "Missing query paramter \'query\' or \'queries\'")
				}
			},
		},

		Err(_) => return error_respond(422, "Malformed query parameters"),
	};

	let query = CanisterQuery { uris };
	let mut response =
		match fetch_v2::<CanisterQuery, CanisterResponse>(query, "/jailbreak/repository/safety")
			.await
		{
			Ok(response) => response,
			Err(err) => return err,
		};

	match is_single && response.count == 1 {
		true => api_respond(
			200,
			json!({
				"data": match response.data[0].safe {
					true => "safe",
					false => "unsafe",
				}
			}),
		),

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

			api_respond(200, json!({ "data": data }))
		}
	}
}
