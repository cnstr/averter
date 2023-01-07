mod routes;
pub mod utility;

use serde_json::json;
use std::{future::Future, pin::Pin};
use tokio::io::Error;

use tide::{
	security::{CorsMiddleware, Origin},
	utils::After,
	Next, Request, Response, Result,
	StatusCode::InternalServerError,
};

use crate::utility::json_respond;

#[tokio::main]
async fn main() -> Result<()> {
	let mut app = tide::new();
	let cors = CorsMiddleware::new().allow_origin(Origin::from("*"));

	app.with(cors);
	app.with(response_time);
	app.with(After(|mut res: Response| async {
		if let Some(err) = res.downcast_error::<Error>() {
			println!("Error: {}", err);
			res = json_respond(
				InternalServerError,
				json!({
					"message": "500 Internal Server Error",
					"date": chrono::Utc::now().to_rfc3339(),
				}),
			);
		}

		Ok(res)
	}));

	app.at("/").get(routes::index);
	app.at("/healthz").get(routes::health);

	app.at("/community/repositories").nest({
		let mut app = tide::new();
		app.at("/safety").get(routes::repository_safety);
		app.at("/search").get(routes::repository_search_ranking);

		app
	});

	app.at("/community/packages").nest({
		let mut app = tide::new();
		app.at("/").get(routes::package_lookup);
		app.at("/lookup").get(routes::package_multi_lookup);
		app.at("/search").get(routes::package_search);

		app
	});

	app.listen("0.0.0.0:3000").await?;
	return Ok(());
}

fn response_time<'a>(
	req: Request<()>,
	next: Next<'a, ()>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
	let start = std::time::Instant::now();
	Box::pin(async move {
		let mut res = next.run(req).await;
		let elapsed = start.elapsed().as_millis();

		res.insert_header("X-Response-Time", elapsed.to_string());
		return Ok(res);
	})
}
