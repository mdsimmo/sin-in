use std::collections::HashMap;

use app_server_core::{Member, ServerSerialize, MembersDeleteResponse, StringResponse, run_handler};
use app_server_core::api::MembersDeleteRequest;
use aws_sdk_dynamodb::{Client, types::{AttributeValue, ReturnValue}};
use lambda_http::{run, service_fn, Error, Request};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    run(service_fn(function_handler_wrap)).await
}

async fn function_handler_wrap(event: Request) -> Result<StringResponse, Error> {
    run_handler(&function_handler, event).await
}

pub async fn function_handler(input: MembersDeleteRequest) -> Result<MembersDeleteResponse, Error> {

    let id = input.id;
    let request_map = {
        let mut map = HashMap::new();
        map.insert("id".to_string(), AttributeValue::S(id.clone()));
        map
    };

    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let table_response = client.delete_item()
        .table_name("sinln-members")
        .set_key(Some(request_map))
        .return_values(ReturnValue::AllOld)
        .send().await;
    log::info!("Table update: {:?}", table_response);
    let table_response = table_response?;

    // Read the old member (if any)
    let old_member = table_response.attributes()
        .map(|attributes| Member::from_row(attributes));
    let old_member = match old_member {
        Some(Ok(m)) => Some(m),
        _ => None,
    };

    Ok(MembersDeleteResponse {
        id,
        old_member,
    })
}

