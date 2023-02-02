use lambda_http::{service_fn, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // initialize dependencies once here for the lifetime of your
    // lambda task
    lambda_http::run(service_fn(|_request| async {
        Result::<&str, std::convert::Infallible>::Ok("👋 world!")
    })).await?;
    Ok(())
}