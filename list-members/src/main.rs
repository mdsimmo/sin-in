use app_core::{Member, add_cors};
use http::Response;
use lambda_http::{run, http::StatusCode, service_fn, Error, Request};
use serde_json::json;
use aws_sdk_dynamodb::{Client};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    run(service_fn(function_error_wrap)).await
}

pub async fn function_error_wrap(event: Request) -> Result<app_core::StringResponse, Error> { 
    if event.method() == http::Method::OPTIONS {
        return Ok(Response::builder()
            .status(200)
            .body("".to_string())
            .unwrap())
    }
    let result = function_handler(event).await;
    let result = match result {
        Ok(r) => Ok(r),
        Err(e) => {
            let new_response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(json!({
                "error": e.to_string(),
                "source": match e.source() {
                    Some(cause) => cause.to_string(),
                    None => "none".to_string(),
                }
              }).to_string())
            .map_err(Box::new)?;
            Ok(new_response)
        }
    };
    return add_cors(result);
}


pub async fn function_handler(_event: Request) -> Result<app_core::StringResponse, Error> {
    // Get all members in dynamodb
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let table_response = client.scan()
        .table_name("sinln-members")
        .send().await;
    log::info!("Table data: {:?}", table_response);
    let table_response = table_response?;

    // Convert json into members
    let result = table_response.items()
        .map(|items| {
            let members: Vec<Member> = items.into_iter().filter_map(|row| {
                match Member::from_row(row) {
                    Ok(m) => Some(m),
                    Err(_err) => None,
                }
            }).collect();
            members
        });
    
    // TODO Handle the case of None
    let members = match result {
        Some(x) => x,
        None => return Err(Box::new(app_core::RuntimeError::from_str("Scan resulted in None? Why would that happen?"))),
    };
    
    // Send response
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json!({
            "members": members, 
          }).to_string())
        .map_err(Box::new)?;

    Ok(response)
}