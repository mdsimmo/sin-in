pub use app_core::*;
pub use app_core::api::*;
use serde::{Deserialize, Serialize};

extern crate serde;
extern crate model;
extern crate aws_sdk_dynamodb;
use std::future::Future;
use lambda_http::{http::StatusCode, Error, Response, Request, RequestExt};
use serde_json::json;

use crate::RuntimeError;

pub type StringResponse = lambda_http::Response<std::string::String>;

pub async fn run_handler<T, R, Fut>(f: &impl Fn(T) -> Fut, event: Request)
    -> Result<StringResponse, Error>
    where Fut: Future<Output = Result<R, Error>> + Send,
    T: for<'de> Deserialize<'de>,
    R: Serialize
{
    add_cors(wrap_errors(run_handler_event(f, event).await))
}

/**
 * Decypts the input event into the requested type, runs the passed function, and encrypts the response. 
 */
async fn run_handler_event<T, R, Fut>(f: &impl Fn(T) -> Fut, event: Request)
    -> Result<StringResponse, Error>
    where Fut: Future<Output = Result<R, Error>> + Send,
    T: for<'de> Deserialize<'de>,
    R: Serialize
{
    // Decrypt the data
    let payload = event.payload::<T>()?;
    let data = match payload {
        Some(x) => x,
        None => return Err(Box::new(RuntimeError::from_str("No data given"))),
    };

    // Run the function
    let result = f(data).await?;

    // Construct response
    let response_body = serde_json::to_string(&result)?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(response_body)
        .map_err(Box::new)?;
    
    Ok(response)
} 

/**
 * Wraps all errors as a 400 response.
 * If they don't get wrapped, AWS Gateway will send a 500 response and not display the error message
 */
fn wrap_errors(result: Result<StringResponse, Error>) -> Result<StringResponse, Error> {
    match result {
        Ok(r) => Ok(r),
        Err(e) => {
            let new_response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(json!({
                "error": e.to_string(),
                "source": match e.source() {
                    Some(cause) => cause.to_string(),
                    None => "none".to_string(),
                }
              }).to_string())
            .map_err(Box::new)?;
            Ok(new_response)
        }
    }
}

fn add_cors(response: Result<StringResponse, Error>) -> Result<StringResponse, Error> {
    match response {
        Ok(msg) => {
            let mut builder = Response::builder();
            {
                let headers = builder.headers_mut().unwrap();
                headers.clone_from(msg.headers());
                //headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("http://localhost:3000"));
            }
            let r = builder
                .status(msg.status())
                .version(msg.version())
                .header("access-control-allow-origin", "http://localhost:3000")
                .body(msg.into_body()).map_err(Box::new)?;
            Ok(r)
        }
        Err(e) => Err(e)
    }
}
