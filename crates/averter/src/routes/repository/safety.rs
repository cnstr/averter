use crate::utility::{api_respond, error_respond, fetch_v2, Request, Response};
use actix_web::{get, web::Query};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize)]
struct Params {
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

#[get("/community/repositories/safety")]
pub async fn safety(req: Request) -> Response {
	let (uris, is_single) = match Query::<Params>::from_query(req.query_string()) {
		Ok(query) => match query.queries.clone() {
			Some(queries) => (queries, false),
			None => match query.query.clone() {
				Some(query) => (query, true),
				None => {
					return error_respond(400, "Missing query parameter: \'query\' or \'queries\'");
				}
			},
		},

		Err(_) => return error_respond(400, "Missing query paramter \'query\' or \'queries\'"),
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
