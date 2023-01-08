use serde_json::json;
use tide::{Request, Result, StatusCode::Ok as OK};

use crate::utility::{api_notice, json_respond};

pub async fn health(_req: Request<()>) -> Result {
	Ok(json_respond(
		OK,
		json!({
			"notice": api_notice(),
			"status": "OK"
		}),
	))
}
