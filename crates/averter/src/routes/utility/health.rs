use crate::utility::http_respond;
use serde_json::json;
use tide::{Request, Result};

pub async fn health(_req: Request<()>) -> Result {
	http_respond(
		200,
		json!({
			"healthy": true
		}),
	)
}
