use app_core::{Member, Topic};
use aws_sdk_sesv2::types::{Destination, Body, Content, Message, EmailContent};
use lambda_http::{Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // extract members from message
    // Read message template from S3
    // Send individual emails in SES

    Ok(())
}

async fn _send_emails(route: &Topic, members: &Vec<Member>, _email: &str, client: &aws_sdk_sesv2::Client) -> Result<(), Error> {

    for member in members {
        if member.subscriptions.iter().any(|sub| &route.endpoint == sub) {
            send_email(member, client).await?;
        }
    }

    Ok(())
}

async fn send_email(member: &Member, client: &aws_sdk_sesv2::Client) -> Result<(), Error> {
    log::info!("Build destination");
    let destintation = Destination::builder()
        .to_addresses(&member.email)
        .build();

    log::info!("Build content");
    let body = Body::builder()
        .text(Content::builder().data("Hello World".to_string()).build()
        ).build(); 
    let message = Message::builder()
        .body(body)
        .subject(Content::builder().data("Test Email!!".to_string()).build())
        .build();
    let content = EmailContent::builder()
        .simple(message)
        .build();

    log::info!("Build reponse");
    let response = client.send_email()
        .destination(destintation)
        .from_email_address("todo@mdsimmo.com")
        .content(content)
        .send()
        .await?;

    log::info!("Response: {:?}", response);
    
    Ok(())
}