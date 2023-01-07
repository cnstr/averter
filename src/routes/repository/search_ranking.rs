use std::collections::{HashMap, HashSet};

use crate::utility::{api_notice, fetch_v2, json_respond};
use serde_json::{json, Value};
use tide::{
	prelude::Deserialize,
	Request, Result,
	StatusCode::{BadRequest, Ok as OK, UnprocessableEntity},
};

#[derive(Deserialize)]
struct Query {
	query: Option<String>,
	ranking: Option<String>,
}

#[derive(Deserialize)]
struct Data {
	slug: String,
	suite: String,
	uri: String,
	tier: u8,
	aliases: Vec<String>,
	name: Option<String>,
	version: Option<String>,
	component: Option<String>,
}

#[derive(Deserialize)]
struct Response {
	date: String,
	data: Vec<Data>,
}

pub async fn repository_search_ranking(req: Request<()>) -> Result {
	match req.query::<Query>() {
		Ok(query) => match query.query {
			Some(query) => return repository_search(query).await,
			None => match query.ranking {
				Some(ranking) => return repository_ranking(ranking).await,
				None => Ok(json_respond(
					BadRequest,
					json!({
						"status": "400 Bad Request",
						"error": "Missing query parameter: \'query\' or \'ranking\'",
						"date": chrono::Utc::now().to_rfc3339(),
					}),
				)),
			},
		},

		Err(err) => {
			println!("Error: {err}");
			Ok(json_respond(
				UnprocessableEntity,
				json!({
					"status": "422 Unprocessable Entity",
					"error": "Malformed query parameters",
					"date": chrono::Utc::now().to_rfc3339(),
				}),
			))
		}
	}
}

async fn repository_search(query: String) -> Result {
	let query = HashMap::from([("q", query.as_str())]);
	let mut response = match fetch_v2::<Response>("/jailbreak/repository/search", query).await {
		Ok(response) => response,
		Err(err) => return Ok(err),
	};

	let data = response.data.iter_mut();
	let data = data
		.map(|item| {
			let package = json!({
				"slug": item.slug,
				"aliases": item.aliases,
				"uri": item.uri,
				"version": item.version,
				"suite": item.suite,
				"component": item.component,
				"ranking": item.tier,
				"name": item.name,
			});

			package
		})
		.collect::<Vec<Value>>();

	Ok(json_respond(
		OK,
		json!({
			"notice": api_notice(),
			"status": "Successful",
			"date": response.date,
			"data": data,
		}),
	))
}

async fn repository_ranking(ranking: String) -> Result {
	let ranks = ranking
		.split(',')
		.filter(|rank| matches!(rank, &"1" | &"2" | &"3" | &"4" | &"5"))
		.collect::<HashSet<&str>>();

	let query = HashMap::from([("rank", "*")]);
	let mut response = match fetch_v2::<Response>("/jailbreak/repository/ranking", query).await {
		Ok(response) => response,
		Err(err) => return Ok(err),
	};

	let data = response.data.iter_mut();
	let data = data
		.filter_map(|item| {
			if ranks.contains(item.tier.to_string().as_str()) {
				let package = json!({
					"slug": item.slug,
					"aliases": item.aliases,
					"uri": item.uri,
					"version": item.version,
					"suite": item.suite,
					"component": item.component,
					"ranking": item.tier,
					"name": item.name,
				});

				Some(package)
			} else {
				None
			}
		})
		.collect::<Vec<Value>>();

	Ok(json_respond(
		OK,
		json!({
			"notice": api_notice(),
			"status": "Successful",
			"date": chrono::Utc::now().to_rfc3339(),
			"data": data,
		}),
	))
}
