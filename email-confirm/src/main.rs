use app_server_core::{Member, Topic, serialize::ServerSerialize, EmailRequest, ConfirmEmailRequest, runtime::{StringResponse, run_handler}, ConfirmEmailResponse};
use lambda_http::{run, Request};
use lambda_runtime::{service_fn, Error};
use tokio::try_join;

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

async fn function_handler(input: ConfirmEmailRequest) -> Result<ConfirmEmailResponse, Error> {
    log::info!("Connecting clients...");
    let config = aws_config::load_from_env().await;
    let sqs_client = aws_sdk_sqs::Client::new(&config);
    let dyno_client = aws_sdk_dynamodb::Client::new(&config);

    log::info!("Fetching endpoints & members...");
    let (topics, members) = try_join!(get_topics(&dyno_client), get_members(&dyno_client))?;

    if let Some(topic) = topics.into_iter().find(|topic| topic.id.as_ref() == Some(&input.topic_id)) {
        queue_emails(&topic, &members, &input.email_id, &sqs_client).await?;
        Ok(ConfirmEmailResponse { 
            topic: Some(topic)
        })
    } else {
        Ok(ConfirmEmailResponse {
            topic: None,
        })
    }
}

async fn queue_emails(topic: &Topic, members: &Vec<Member>, email_id: &str, client: &aws_sdk_sqs::Client) -> Result<(), Error> {
    for member in members {
        if member.subscriptions.iter().any(|sub| topic.endpoint.contains(sub)) {
            queue_email(topic, member, client, email_id).await?;
        }
    }
    Ok(())
}

async fn queue_email(topic: &Topic, member: &Member, client: &aws_sdk_sqs::Client, email_id: &str) -> Result<(), Error> {
    let event = EmailRequest {
        topic: topic.clone(),
        member: member.clone(),
        email_id: email_id.to_owned(),
        confirm_link: false,
    };

    let result = client.send_message()
        .queue_url("https://sqs.us-east-1.amazonaws.com/400928329577/sinln-output-queue") // TODO don't hard code output queue URL
        .message_body(serde_json::to_string(&event)?)
        .send()
        .await?;

    log::info!("Email queue: {:?}", result);

    Ok(())
}

async fn get_topics(client: &aws_sdk_dynamodb::Client) -> Result<Vec<Topic>, Error> {
    let table_response = client.scan()
        .table_name("sinln-topics")
        .send().await;
    log::info!("Topics data: {:?}", table_response);
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

    Ok(topics)
}

async fn get_members(client: &aws_sdk_dynamodb::Client) -> Result<Vec<Member>, Error> {
    let table_response = client.scan()
        .table_name("sinln-members")
        .send().await;
    log::info!("Members data: {:?}", table_response);
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
        None => return Err(Box::new(app_server_core::RuntimeError::from_str("Scan resulted in None? Why would that happen?"))),
    };

    Ok(members)
}