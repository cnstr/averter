use std::collections::HashSet;

use crate::utility::{api_respond, error_respond, fetch_v2};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tide::{Request, Result};

#[derive(Serialize, Deserialize)]
struct Query {
	query: Option<String>,
	ranking: Option<String>,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
struct CanisterQuery {
	q: Option<String>,
	rank: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CanisterResponse {
	date: String,
	data: Vec<Data>,
}

pub async fn repository_search_ranking(req: Request<()>) -> Result {
	match req.query::<Query>() {
		Ok(query) => match query.query {
			Some(query) => return repository_search(query).await,
			None => match query.ranking {
				Some(ranking) => return repository_ranking(ranking).await,
				None => error_respond(400, "Missing query paramter \'query\' or \'ranking\'"),
			},
		},

		Err(_) => error_respond(422, "Malformed query parameters"),
	}
}

async fn repository_search(query: String) -> Result {
	let query = CanisterQuery {
		q: Some(query),
		rank: None,
	};

	let mut response =
		match fetch_v2::<CanisterQuery, CanisterResponse>(query, "/jailbreak/repository/search")
			.await
		{
			Ok(response) => response,
			Err(err) => return err,
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

	api_respond(
		200,
		json!({
			"data": data,
		}),
	)
}

async fn repository_ranking(ranking: String) -> Result {
	let ranks = ranking
		.split(',')
		.filter(|rank| matches!(rank, &"1" | &"2" | &"3" | &"4" | &"5"))
		.collect::<HashSet<&str>>();

	let query = CanisterQuery {
		q: None,
		rank: Some("*".to_owned()),
	};

	let mut response =
		match fetch_v2::<CanisterQuery, CanisterResponse>(query, "/jailbreak/repository/ranking")
			.await
		{
			Ok(response) => response,
			Err(err) => return err,
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

	api_respond(
		200,
		json!({
			"data": data,
		}),
	)
}
