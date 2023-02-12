use std::collections::HashMap;

use app_core::Member;
use aws_sdk_dynamodb::{Client, model::{AttributeValue, ReturnValue}};
use http::Response;
use lambda_http::{run, http::StatusCode, service_fn, Error, Request, RequestExt};
use serde::{Deserialize, Serialize};
use serde_json::json;

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
    let body = event.payload::<Data>()?;
    log::info!("Request: {:?}", body);
    let data = match body {
        Some(x) => x,
        None => return Err(Box::new(app_core::RuntimeError::from_str("No data given"))),
    };
    
    let id = data.id;
    let request_map = {
        let mut map = HashMap::new();
        map.insert("id".to_string(), AttributeValue::S("asd".to_string()));
        map
    };

    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let table_response = client.delete_item()
        .table_name("sinln-members")
        .set_key(Some(request_map))
        .return_values(ReturnValue::AllOld)
        .send().await?;
    log::info!("Table update: {:?}", table_response);

    // Read the old member (if any)
    let old_member = table_response.attributes()
        .map(|attributes| Member::from_row(attributes));
    let old_member = match old_member {
        Some(Ok(m)) => Some(m),
        _ => None,
    };

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json!({
            "id": id,
            "old-member": old_member, 
          }).to_string())
        .map_err(Box::new)?;

    Ok(response)
}


#[derive(Deserialize, Serialize, Debug, Clone)]
struct Data {
    id: String,
}