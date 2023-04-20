use app_server_core::{Topic, ServerSerialize, StringResponse, run_handler, TopicsUpdateRequest, TopicsUpdateResponse};
use lambda_http::{run, service_fn, Error, Request};
use rand::Rng;
use serde::{Deserialize, Serialize};
use aws_sdk_dynamodb::{Client, types::{ReturnValue}};

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

pub async fn function_handler(input: TopicsUpdateRequest) -> Result<TopicsUpdateResponse, Error> {
    let mut topic = input.topic;

    // If no id assigned, assign one
    if topic.id == None {
        let id_time = chrono::Utc::now();
        let id_random = rand::thread_rng().gen::<u32>();
        let mut id_string = id_time.format("%Y-%m-%d-%H:%M:%S-").to_string();
        id_string.push_str(&id_random.to_string());
        topic.id = Some(id_string)
    };
    log::info!("New topic: {:?}", topic.id);

    // Put the topic in dynamodb
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let table_response = client.put_item()
        .table_name("sinln-topics")
        .set_item(Some(topic.into_row()))
        .return_values(ReturnValue::AllOld)
        .send().await;
    log::info!("Table update: {:?}", table_response);
    let table_response = table_response?;

    // Read the old topic (if any)
    let old_topic = table_response.attributes()
        .map(|attributes| Topic::from_row(attributes));
    let old_topic = match old_topic {
        Some(Ok(m)) => Some(m),
        _ => None,
    };

    Ok(TopicsUpdateResponse { 
        topic, 
        old_topic,
    })
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Data {
    topic: Topic,
}