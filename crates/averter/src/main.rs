use actix_cors::Cors;
use actix_web::{dev::Service, web, App, HttpServer};
use futures_util::future::FutureExt;
use http::header::HeaderName;
use sentry::{init, ClientOptions};
use sentry_actix::Sentry;
use std::{env::set_var, io::Result, str::FromStr, time::Instant};
use utility::create_canister_client;

mod routes;
mod utility;

#[warn(clippy::all)]
#[warn(clippy::correctness)]
#[warn(clippy::suspicious)]
#[warn(clippy::pedantic)]
#[warn(clippy::style)]
#[warn(clippy::complexity)]
#[warn(clippy::perf)]

/// The main function of the Averter service
#[actix_web::main]
async fn main() -> Result<()> {
	let _guard = init((
		env!("CANISTER_SENTRY_DSN"),
		ClientOptions {
			release: Some(env!("VERGEN_BUILD_SEMVER").into()),
			traces_sample_rate: 0.5,
			..Default::default()
		},
	));

	// Enable backtraces for Sentry
	set_var("RUST_BACKTRACE", "1");
	create_canister_client();

	HttpServer::new(|| {
		App::new()
			.wrap_fn(|req, next| {
				let start = Instant::now();
				next.call(req).map(move |res| {
					let elapsed = start.elapsed().as_millis();
					res.map(|mut res| {
						res.headers_mut().insert(
							HeaderName::from_str("X-Response-Time").unwrap(),
							format!("{}", elapsed).parse().unwrap(),
						);
						res
					})
				})
			})
			.wrap(Sentry::new())
			.wrap(Cors::default().allow_any_origin().send_wildcard())
			.default_service(web::to(routes::utility::not_found))
			.service(routes::utility::index)
			.service(routes::utility::health)
			.service(routes::package::lookup)
			.service(routes::package::multi_lookup)
			.service(routes::package::search)
			.service(routes::repository::safety)
			.service(routes::repository::search_ranking)
	})
	.bind(("0.0.0.0", 3000))?
	.run()
	.await
}
