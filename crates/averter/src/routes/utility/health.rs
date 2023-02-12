use crate::utility::{http_respond, Request, Response};
use actix_web::get;
use serde_json::json;

#[get("/healthz")]
pub async fn health(_req: Request) -> Response {
	http_respond(
		200,
		json!({
			"healthy": true
		}),
	)
}
