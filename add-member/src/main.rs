use std::collections::HashMap;
use app_core::Member;
use http::Response;
use lambda_http::{run, http::StatusCode, service_fn, Error, Request, RequestExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use aws_sdk_dynamodb::{Client, model::AttributeValue};

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
    let result = function_handler(event).await;
    return match result {
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
    }
}


pub async fn function_handler(event: Request) -> Result<app_core::StringResponse, Error> {
    log::info!("Event: {:?}", event);
    
    // decrypt the request
    let raw_data = event.payload::<Data>()?;
    log::info!("Data: {:?}", raw_data);
    let data = match raw_data {
        Some(x) => x,
        None => return Err(Box::new(app_core::RuntimeError::from_str("No data given")))
    };

    // Assign a new id
    let id = {
        let id_time = chrono::Utc::now();
        let id_random = rand::thread_rng().gen::<u32>();
        let mut id_string = id_time.format("%Y-%m-%d-%H:%M:%S-").to_string();
        id_string.push_str(&id_random.to_string());
        id_string
    };
    log::info!("Id: {:?}", id);

    // Put the member in dynamodb
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let table_response = client.put_item()
        .table_name("sinln-members")
        .set_item(Some(HashMap::from(&data.member)))
        .item("id",AttributeValue::S(id.clone()))
        .send().await?;
   log::info!("Table update: {:?}", table_response);
    
    // Send response
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json!({
            "id": id,
            "member": data.member, 
          }).to_string())
        .map_err(Box::new)?;

    Ok(response)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Data {
    member: Member,
}