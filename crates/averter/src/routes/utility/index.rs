use crate::utility::{http_respond, Request, Response};
use actix_web::get;
use chrono::{Datelike, Utc};
use serde_json::json;

#[get("/")]
pub async fn index(req: Request) -> Response {
	let name = format!(
		"{} ({})",
		env!("CANISTER_PRODUCTION_NAME"),
		env!("CANISTER_CODE_NAME")
	);

	let build = format!(
		"{}+git-{}-tree/{}",
		env!("VERGEN_BUILD_TIMESTAMP"),
		env!("VERGEN_GIT_SHA_SHORT"),
		env!("VERGEN_GIT_BRANCH")
	);

	let platform = format!(
		"rust-{}+{}_llvm{}",
		env!("VERGEN_RUSTC_SEMVER"),
		env!("VERGEN_RUSTC_HOST_TRIPLE"),
		env!("VERGEN_RUSTC_LLVM_VERSION")
	);

	let copyright = env!("CANISTER_COPYRIGHT").replace("{{year}}", &Utc::now().year().to_string());
	let current_date = Utc::now().date_naive().to_string();
	let current_epoch = Utc::now().timestamp();

	let connection_info = req.connection_info();
	let remote_address = connection_info.realip_remote_addr().unwrap_or("Unknown");

	let user_agent = match req.headers().get("User-Agent") {
		Some(user_agent) => user_agent.to_str().unwrap_or("Unknown"),
		None => "Unknown",
	};

	http_respond(
		200,
		json!({
			"message": name,
			"version": env!("VERGEN_BUILD_SEMVER"),
			"build": build,
			"platform": platform,
			"runtime": env!("CANISTER_K8S_VERSION"),

			"reference": {
				"docs": env!("CANISTER_DOCS_ENDPOINT"),
				"privacy_policy": env!("CANISTER_PRIVACY_ENDPOINT"),
				"contact_email": env!("CANISTER_CONTACT_EMAIL"),
				"copyright": copyright,
			},

			"connection": {
				"current_date": current_date,
				"current_epoch": current_epoch,
				"remote_address": remote_address,
				"user_agent": user_agent,
			}
		}),
	)
}
