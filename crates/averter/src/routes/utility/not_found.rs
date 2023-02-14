use crate::utility::{api_respond, Request, Response};
use serde_json::json;

pub async fn not_found(_req: Request) -> Response {
	api_respond(404, false, json!({}))
}
