use app_server_core::{Member, Topic, serialize::ServerSerialize, EmailRequest};
use aws_lambda_events::{sns::SnsMessage, sqs::SqsEvent,ses::SimpleEmailService};
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    lambda_runtime::run(service_fn(handler)).await
}

async fn handler(event: LambdaEvent<Value>) -> Result<(), Error> {
    log::info!("Loading config...");

    let (event_value, _context) = event.into_parts();   
    log::info!("event: {:?}", event_value);
    let sqs_event: SqsEvent = serde_json::from_value(event_value)?;
    
    log::info!("Connecting clients...");
    let config = aws_config::load_from_env().await;
    let sqs_client = aws_sdk_sqs::Client::new(&config);
    let dyno_client = aws_sdk_dynamodb::Client::new(&config);

    log::info!("Fetching endpoint & members...");
    let topics = get_topics(&dyno_client).await?;

    for sqs_record in &sqs_event.records {
        log::info!("Decoding SNS Record");
        let sns_message: SnsMessage = serde_json::from_str(&sqs_record.body.as_ref().unwrap())?;
        log::info!("Decoding SES Record");
        let ses_service: SimpleEmailService = serde_json::from_str(&sns_message.message)?;
        log::info!("Get message id");
        let message_id = ses_service.mail.message_id.unwrap();

        for target in &ses_service.mail.destination { 
            if let Some(topic) = topics.iter().find(|topic| &topic.endpoint == target) {
                let member = Member {
                    id: Some("---".to_owned()),
                    name: "Sender".to_owned(),
                    email: ses_service.mail.source.as_ref().unwrap().to_owned(),
                    address: None,
                    mobile: None,
                    subscriptions: vec![],
                };
                queue_email(&topic, &member, &message_id, &sqs_client).await?;
            } else {
                todo!("Send bad endpoint email back");
            }
        }
    }

    Ok(())
}

async fn queue_email(topic: &Topic, member: &Member, email_id: &str, client: &aws_sdk_sqs::Client) -> Result<(), Error> {
    let event = EmailRequest {
        topic: topic.clone(),
        member: member.clone(),
        email_id: email_id.to_owned(),
        confirm_link: true,
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
