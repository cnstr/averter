use manifest::{load_manifest, Conditional};
use reqwest::ClientBuilder;
use serde::Deserialize;
use serde_json::from_str as from_json;
use tokio::main;
use vergen::{vergen, Config, ShaKind};

/// Kubernetes HTTP Response
#[derive(Deserialize)]
struct K8sResponse {
	#[serde(rename = "gitVersion")]
	git_version: String,
	platform: String,
}

fn main() {
	register_vergen_envs();
	let manifest = load_manifest("../../manifest.yaml");

	set_env("CANISTER_PRODUCTION_NAME", &manifest.meta.production_name);
	set_env("CANISTER_PRIVACY_ENDPOINT", &manifest.endpoints.privacy);
	set_env("CANISTER_NOTICE_MIGRATION", &manifest.notice.migration);
	set_env("CANISTER_CONTACT_EMAIL", &manifest.meta.contact_email);
	set_env("CANISTER_COPYRIGHT", &manifest.meta.copyright_string);
	set_env("CANISTER_DOCS_ENDPOINT", &manifest.endpoints.docs);
	set_env("CANISTER_SITE_ENDPOINT", &manifest.endpoints.site);
	set_env("CANISTER_API_ENDPOINT", &manifest.endpoints.api);
	set_env("CANISTER_CODE_NAME", &manifest.meta.code_name);
	set_env("CANISTER_NOTICE_DATA", &manifest.notice.data);
	set_env("CANISTER_NOTICE_API", &manifest.notice.api);

	load_sentry_dsn(manifest.build.sentry_dsn);
	load_k8s_info(manifest.build.k8s_control_plane);
}

/// Registers environment variables from the 'vergen' crate
/// While unnecessary, the defaults are tuned to only what we want
fn register_vergen_envs() {
	let mut config = Config::default();

	// Disable default features we don't need
	*config.rustc_mut().sha_mut() = false;
	*config.git_mut().semver_mut() = false;
	*config.cargo_mut().enabled_mut() = false;
	*config.rustc_mut().channel_mut() = false;
	*config.sysinfo_mut().enabled_mut() = false;
	*config.rustc_mut().commit_date_mut() = false;
	*config.git_mut().commit_timestamp_mut() = false;

	// Reconfigure the Git SHA to be shortened output
	*config.git_mut().sha_kind_mut() = ShaKind::Short;

	match vergen(config) {
		Ok(_) => (),
		Err(e) => panic!("Failed to register 'vergen' configuration ({e})"),
	}
}

/// Set a cargo environment variable
/// Used for build-time variables
fn set_env(key: &str, value: &str) {
	println!("Registering environment variable: {key}={value}");
	println!("cargo:rustc-env={key}={value}");
}

/// Loads the Sentry DSN from the manifest
fn load_sentry_dsn(dsn: Conditional) {
	let sentry_dsn = match cfg!(debug_assertions) {
		true => &dsn.debug,
		false => &dsn.release,
	};

	set_env("CANISTER_SENTRY_DSN", sentry_dsn);
}

/// Fetches the Kubernetes version from the control plane
/// Sets the CANISTER_K8S_VERSION environment variable
#[main]
async fn load_k8s_info(k8s_host: String) {
	let client = match ClientBuilder::new()
		.danger_accept_invalid_certs(true)
		.build()
	{
		Ok(client) => client,
		Err(e) => panic!("Failed to build insecure HTTP client ({e})"),
	};

	let url = format!("https://{k8s_host}/version");
	let json = match client.get(url).send().await {
		Ok(response) => match response.text().await {
			Ok(value) => {
				let value: K8sResponse = match from_json(&value) {
					Ok(value) => value,
					Err(e) => panic!("Failed to deserialize Kubernetes HTTP response ({e})"),
				};

				value
			}
			Err(e) => panic!("Failed to parse Kubernetes HTTP response ({e})"),
		},
		Err(e) => panic!("Failed to fetch Kubernetes version ({e})"),
	};

	set_env(
		"CANISTER_K8S_VERSION",
		format!("k8s_{}-{}", &json.git_version, &json.platform).as_str(),
	);
}
