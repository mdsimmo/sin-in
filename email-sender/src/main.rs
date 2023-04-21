use app_server_core::{EmailRequest};
use aws_sdk_dynamodb::primitives::Blob;
use aws_sdk_sesv2::types::{Destination, EmailContent, RawMessage};
use email_format::{Email, rfc5322::Parsable};
use lambda_http::{service_fn};
use lambda_runtime::{LambdaEvent, Error};
use serde_json::Value;
use aws_lambda_events::sqs::SqsEvent;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    lambda_runtime::run(service_fn(event_handler)).await
}

async fn event_handler(event: LambdaEvent<Value>) -> Result<(), Error> {
    log::info!("Loading SQS Event");
    let (event_value, _context) = event.into_parts();
    let sqs_event: SqsEvent = serde_json::from_value(event_value)?;
    
    log::info!("Loading Config/clients");
    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);
    let ses_client = aws_sdk_sesv2::Client::new(&config);
    
    for sqs_record in &sqs_event.records {
        log::info!("Decoding SQS Record");
        log::info!("SQS Body: {}", &sqs_record.body.as_ref().unwrap());
        let request: EmailRequest = serde_json::from_str(&sqs_record.body.as_ref().unwrap())?;
        log::info!("Getting email content");
        let email_content = get_email(&request.email_id[..], &s3_client).await?;

        send_email(&request, &ses_client, &email_content).await?;
    }
    
    Ok(()) 
}

async fn send_email(request: &EmailRequest, client: &aws_sdk_sesv2::Client, email_template: &str) -> Result<(), Error> {
    log::info!("Interperetting Email....");
    
    let (mut email_obj, _remainder) = Email::parse(email_template.as_bytes())?;

    if let Some(body) = email_obj.get_body() {
        let mut message = body.to_string();
        
        if request.confirm_link {
            message += &format!("\r\n\r\nConfirm email: https://sinln.mdsimmo.com/email-confirm?topic={}&email={}", request.topic.id.as_ref().unwrap(), &request.email_id);
        } else {
            message += &format!("\r\n\r\nUnsubscibe: https://sinln.mdsimmo.com/unsubscribe?member={}&topic={}", request.topic.id.as_ref().unwrap(), request.member.id.as_ref().unwrap()) 
        }
        email_obj.set_body(message.as_str()).unwrap();
    }
    let email = email_obj.to_string();

    log::info!("Build destination");
    let destintation = Destination::builder()
        .to_addresses(&request.member.email)
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
        .from_email_address(&request.topic.endpoint)
        .destination(destintation)
        .send()
        .await?;

    log::info!("Response: {:?}", response);
    
    Ok(())
}

async fn get_email(message_id: &str, client: &aws_sdk_s3::Client) -> Result<String, Error> {
    let get_result = client.get_object()
        .bucket("sinln-input-emails")
        .key(message_id)
        .send().await?;

    let bytes = get_result.body.collect().await?.into_bytes();
    let email_data = std::str::from_utf8(&bytes)?;

    Ok(email_data.to_string())
}

#[cfg(test)]
mod tests {
    use email_format::{Email, rfc5322::Parsable};

    #[test]
    fn test_sender() {
        let input = "".as_bytes();//include_bytes!("p3dl6t0bta05uvn533d6ev3dj9ts1r4g6c399ag1.txt");
        //println!("Input: {}", input);
        
        let (mut email_obj, _remainder) = Email::parse(input).unwrap();
        if let Some(body) = email_obj.get_body() {
            let mut message = body.to_string();
            message += "\r\n\r\nHello World";
            println!("Message: {}", message);
            email_obj.set_body(message.as_str()).unwrap();
        }
        let email = email_obj;
        println!("Output: {}", email);
        println!("Output: {:?}", String::from_utf8(_remainder.into_iter().map(|&a|a).collect()).unwrap());
    }
}