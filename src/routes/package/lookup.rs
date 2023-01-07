use std::collections::HashMap;

use crate::utility::{api_notice, fetch_v2, json_respond};
use serde_json::json;
use tide::{
	prelude::Deserialize,
	Request, Result,
	StatusCode::{BadRequest, NotFound, Ok as OK, UnprocessableEntity},
};

#[derive(Deserialize)]
struct Query {
	id: Option<String>,
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
	#[serde(rename = "isCurrent")]
	is_current: bool,
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

pub async fn package_lookup(req: Request<()>) -> Result {
	let id = match req.query::<Query>() {
		Ok(query) => match query.id {
			Some(query) => query,
			None => {
				return Ok(json_respond(
					BadRequest,
					json!({
						"status": "400 Bad Request",
						"error": "Missing query parameter: \'id\'",
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

	let query = HashMap::from([]); // No queries necessary here
	let uri = &format!("/jailbreak/package/{}", id).to_owned();

	let mut response = match fetch_v2::<Response>(uri, query).await {
		Ok(response) => response,
		Err(err) => return Ok(err),
	};

	let mut data = response.data.iter_mut();
	let data = data.find_map(|item| match item.is_current {
		true => Some(json!({
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
		})),
		false => None,
	});

	match &data {
		Some(_) => (),
		None => {
			return Ok(json_respond(
				NotFound,
				json!({
					"status": "404 Not Found",
					"error": "Package not found",
					"date": chrono::Utc::now().to_rfc3339(),
				}),
			))
		}
	};

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
