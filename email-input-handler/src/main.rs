use app_core::{Member, Topic};
use aws_lambda_events::ses::SimpleEmailEvent;
use aws_sdk_dynamodb::primitives::Blob;
use aws_sdk_sesv2::types::{Destination, EmailContent, RawMessage};
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::Value;
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn handler(event: LambdaEvent<Value>) -> Result<(), Error> {
    log::info!("Loading config...");

    let (event_value, _context) = event.into_parts();   
    log::info!("event: {:?}", event_value);
    let email_event: SimpleEmailEvent = serde_json::from_value(event_value)?;

    log::info!("Connecting clients...");
    let config = aws_config::load_from_env().await;
    let ses_client = aws_sdk_sesv2::Client::new(&config);
    let dyno_client = aws_sdk_dynamodb::Client::new(&config);
    let s3_client = aws_sdk_s3::Client::new(&config);

    log::info!("Fetching endpoint & members...");
    let (routes, members) = try_join!(get_routes(&dyno_client), get_members(&dyno_client))?;

    for record in &email_event.records {
        let a = &record.ses.mail.common_headers.from;
        let message_id = record.ses.mail.message_id.as_ref().unwrap();
        let email = get_email(&message_id, &s3_client).await?; // TODO parralise awaits
        for target in &record.ses.mail.destination { 
            if let Some(topic) = routes.iter().find(|topic| &topic.endpoint == target) {
                queue_emails(&topic, &members, &email, &ses_client).await?; // TODO parralise awaits
            } else {
                let topic = Topic {
                    id: Some("hello".to_string()),
                    name: "hello".to_string(),
                    endpoint: "hello@mdsimmo.com".to_string(),
                    default: true,
                };
                queue_emails(&topic, &members, &email, &ses_client).await?; // TODO parralise awaits
                //todo!("What to do if there is no route???");
            }
        }
    }

    Ok(())
}

async fn queue_emails(topic: &Topic, members: &Vec<Member>, email: &str, client: &aws_sdk_sesv2::Client) -> Result<(), Error> {
    for member in members {
        //if member.subscriptions.iter().any(|sub| route.endpoint.contains(sub)) {
            queue_email(topic, member, client, email).await?;
        //}
    }
    Ok(())
}

async fn queue_email(topic: &Topic, member: &Member, client: &aws_sdk_sesv2::Client, email: &str) -> Result<(), Error> {
    log::info!("Build destination");
    let destintation = Destination::builder()
        .to_addresses(&member.email)
        .build();

    log::info!("Build content");
    let raw_message = RawMessage::builder()
        .data(Blob::new(email))
        .build();

    let content = EmailContent::builder()
        .raw(raw_message)
        .build();

    log::info!("Build reponse");
    let response = client.send_email()
        .content(content)
        .from_email_address(&topic.endpoint)
        .destination(destintation)
        .send()
        .await?;

    log::info!("Response: {:?}", response);
    
    Ok(())
}

async fn get_email(message_id: &str, client: &aws_sdk_s3::Client) -> Result<String, Error> {
    /*let copy_result = client.copy_object()
        .bucket("sinln-input-emails")
        .copy_source("sinln-input-emails/.".to_string() + message_id)
        .key(message_id)
        .acl(aws_sdk_s3::types::ObjectCannedAcl::Private)
        .content_type("text/plain")
        .storage_class(StorageClass::Standard)
        .send().await?;*/
    
    let get_result = client.get_object()
        .bucket("sinln-input-emails")
        .key(message_id)
        .send().await?;

    let bytes = get_result.body.collect().await?.into_bytes();
    let email_data = std::str::from_utf8(&bytes)?;

    Ok(email_data.to_string())
}

async fn get_routes(client: &aws_sdk_dynamodb::Client) -> Result<Vec<Topic>, Error> {
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
        None => return Err(Box::new(app_core::RuntimeError::from_str("Scan resulted in None? Why would that happen?"))),
    };

    Ok(topics)
}

async fn get_members(client: &aws_sdk_dynamodb::Client) -> Result<Vec<Member>, Error> {
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

    Ok(members)
}