use anyhow::Error;
use once_cell::sync::OnceCell;
use sentry::integrations::anyhow::capture_anyhow;
use surf::{Client, Config, Url};

static CANISTER: OnceCell<Client> = OnceCell::new();

/// Connects to the API v2 client and globalizes it
pub fn create_canister_client() {
	let base_url = match Url::parse(env!("CANISTER_API_ENDPOINT")) {
		Ok(url) => url,
		Err(err) => {
			let anyhow: Error = err.into();
			handle_error(&anyhow);
			panic!("Failed to parse Canister Host: {}", anyhow)
		}
	};

	let client = match Config::new()
		.set_base_url(base_url)
		.add_header("Accept", "application/json")
	{
		Ok(client) => {
			let client: Client = match client.try_into() {
				Ok(client) => client,
				Err(err) => {
					let anyhow: Error = err.into();
					handle_error(&anyhow);
					panic!("Failed to create Canister Client: {}", anyhow)
				}
			};

			client
		}
		Err(err) => {
			let anyhow: Error = err.into_inner();
			handle_error(&anyhow);
			panic!("Failed to create Canister Client: {}", anyhow)
		}
	};

	match CANISTER.set(client) {
		Ok(_) => (),
		Err(_) => panic!("Failed to globalize Canister Client"),
	}
}

/// Returns the globalized API v2 Client
pub fn canister() -> &'static Client {
	match CANISTER.get() {
		Some(client) => client,
		None => panic!("Canister Client not initialized"),
	}
}

/// Takes an error and reports it to Sentry
pub fn handle_error(err: &Error) {
	let uuid = capture_anyhow(err);
	println!("--------------------------");
	println!("Reporting an error (Sentry UUID: {})", uuid);
	println!("Error: {}", err);
	println!("--------------------------");
}
