use std::collections::HashMap;

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
}

#[derive(Deserialize)]
struct Repository {
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
struct Data {
	package: String,
	architecture: String,
	price: String,
	version: String,
	name: Option<String>,
	description: Option<String>,
	author: Option<String>,
	maintainer: Option<String>,
	depiction: Option<String>,
	#[serde(rename = "sileoDepiction")]
	sileo_depiction: Option<String>,
	header: Option<String>,
	#[serde(rename = "tintColor")]
	tint_color: Option<String>,
	icon: Option<String>,
	section: Option<String>,
	repository: Repository,
}

#[derive(Deserialize)]
struct Response {
	date: String,
	data: Vec<Data>,
}

pub async fn package_search(req: Request<()>) -> Result {
	let query = match req.query::<Query>() {
		Ok(query) => match query.query {
			Some(query) => query,
			None => {
				return Ok(json_respond(
					BadRequest,
					json!({
						"status": "400 Bad Request",
						"error": "Missing query parameter: \'query\'",
						"date": chrono::Utc::now().to_rfc3339(),
					}),
				))
			}
		},

		Err(err) => {
			println!("Error: {}", err);
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

	let query = HashMap::from([("q", query.as_str())]);
	let mut response = match fetch_v2::<Response>("/jailbreak/package/search", query).await {
		Ok(response) => response,
		Err(err) => return Ok(err),
	};

	let data = response.data.iter_mut();
	let data = data
		.map(|item| {
			let package = json!({
				"identifier": item.package,
				"architecture": item.architecture,
				"price": item.price,
				"latestVersion": item.version,
				"name": item.name,
				"description": item.description,
				"author": item.author,
				"maintainer": item.maintainer,
				"depiction": item.depiction,
				"nativeDepiction": item.sileo_depiction,
				"header": item.header,
				"tintColor": item.tint_color,
				"packageIcon": item.icon,
				"section": item.section,
				"repository": {
					"slug": item.repository.slug,
					"aliases": item.repository.aliases,
					"uri": item.repository.uri,
					"version": item.repository.version,
					"suite": item.repository.suite,
					"component": item.repository.component,
					"ranking": item.repository.tier,
					"name": item.repository.name,
				}
			});

			package
		})
		.collect::<Vec<Value>>();

	Ok(json_respond(
		OK,
		json!({
			"notice": api_notice(),
			"message": "Successful",
			"date": response.date,
			"data": data,
		}),
	))
}
