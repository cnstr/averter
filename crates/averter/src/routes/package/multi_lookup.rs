use crate::utility::{api_respond, error_respond, fetch_v2, Request, Response};
use actix_web::{get, web::Query};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize)]
struct Params {
	packages: String,
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
struct CanisterQuery {
	ids: String,
}

#[derive(Serialize, Deserialize)]
struct CanisterResponse {
	date: String,
	data: Vec<Data>,
}

#[get("/community/packages/lookup")]
pub async fn multi_lookup(req: Request) -> Response {
	let packages = match Query::<Params>::from_query(req.query_string()) {
		Ok(query) => query.packages.clone(),
		Err(_) => return error_respond(400, "Missing query parameter: \'packages\'"),
	};

	let query = CanisterQuery { ids: packages };
	let mut response = match fetch_v2::<CanisterQuery, CanisterResponse>(
		query,
		"/jailbreak/package/multi",
	)
	.await
	{
		Ok(response) => response,
		Err(err) => return err,
	};

	let data = response.data.iter_mut();
	let data = data
		.map(|item| {
			let package = json!({
				"package": item.package,
				"fields": vec![json!({
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
				})]
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
