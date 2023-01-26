use crate::utility::{api_respond, error_respond, fetch_v2};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tide::{Request, Result};

#[derive(Serialize, Deserialize)]
struct Query {
	id: Option<String>,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
struct CanisterQuery {}

#[derive(Serialize, Deserialize)]
struct CanisterResponse {
	date: String,
	data: Vec<Data>,
}

pub async fn package_lookup(req: Request<()>) -> Result {
	let id = match req.query::<Query>() {
		Ok(query) => match query.id {
			Some(query) => query,
			None => return error_respond(400, "Missing query parameter: \'id\'"),
		},

		Err(_) => return error_respond(422, "422 Unprocessable Entity"),
	};

	let query = CanisterQuery {}; // No query parameters
	let uri = &format!("/jailbreak/package/{id}").to_owned();

	let mut response = match fetch_v2::<CanisterQuery, CanisterResponse>(query, uri).await {
		Ok(response) => response,
		Err(err) => return err,
	};

	let mut data = response.data.iter_mut();
	let data: Option<Value> = data.find_map(|item| match item.is_current {
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
		None => return error_respond(404, "Package not found"),
	};

	api_respond(
		200,
		json!({
			"data": data,
		}),
	)
}
