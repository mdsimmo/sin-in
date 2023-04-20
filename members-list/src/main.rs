use app_server_core::{Member, ServerSerialize, MembersListRequest, MembersListResponse, StringResponse, run_handler};
use lambda_http::{run, service_fn, Error, Request};
use aws_sdk_dynamodb::{Client};

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


pub async fn function_handler(_event: MembersListRequest) -> Result<MembersListResponse, Error> {
    // Get all members in dynamodb
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
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
        None => return Err(Box::new(app_server_core::RuntimeError::from_str("Scan resulted in None? Why would that happen?"))),
    };
    
    Ok(MembersListResponse {
        members
    })
}