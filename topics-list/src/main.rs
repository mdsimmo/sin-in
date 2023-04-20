use app_server_core::{Topic, ServerSerialize, run_handler, StringResponse, TopicsListRequest, TopicsListResponse};
use lambda_http::{run, service_fn, Error, Request};
use aws_sdk_dynamodb::Client;

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

pub async fn function_handler(_: TopicsListRequest) -> Result<TopicsListResponse, Error> {
    // Get all topics in dynamodb
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let table_response = client.scan()
        .table_name("sinln-topics")
        .send().await;
    log::info!("Table data: {:?}", table_response);
    let table_response = table_response?;

    // Convert json into topics
    let result = table_response.items()
        .map(|items| {
            let topics: Vec<Topic> = items.into_iter().filter_map(|row| {
                match Topic::from_row(row) {
                    Ok(m) => Some(m),
                    Err(_err) => None,
                }
            }).collect();
            topics
        });
    
    // TODO Handle the case of None
    let topics = match result {
        Some(x) => x,
        None => return Err(Box::new(app_server_core::RuntimeError::from_str("Scan resulted in None? Why would that happen?"))),
    };
    
    Ok(TopicsListResponse {
        topics
    })
}