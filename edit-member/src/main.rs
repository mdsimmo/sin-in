use app_core::Member;
use http::Response;
use lambda_http::{run, http::StatusCode, service_fn, Error, IntoResponse, Request, RequestExt};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    run(service_fn(function_handler)).await
}

pub async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    let body = event.payload::<Member>()?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json!({
            "message": "Edit Member",
            "payload": body, 
          }).to_string())
        .map_err(Box::new)?;

    Ok(response)
}