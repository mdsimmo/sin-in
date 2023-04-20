use app_server_core::{Topic, DeleteResponse,  DeleteRequest, runtime::StringResponse, runtime::run_handler, crud::delete_items};
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

pub async fn function_handler(input: DeleteRequest) -> Result<DeleteResponse<Topic>, Error> {
    delete_items(input, "sinln-topics").await
}

