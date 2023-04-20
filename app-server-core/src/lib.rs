pub use app_core::*;
pub use app_core::api::*;
use serde::{Deserialize, Serialize};

extern crate serde;
extern crate model;
extern crate aws_sdk_dynamodb;
use std::{collections::HashMap, future::Future};
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_http::{http::StatusCode, Error, Response, Request, RequestExt};
use serde_json::json;

fn read_string<'a>(data: &'a HashMap<String, AttributeValue>, key: &str) -> Result<&'a str, RuntimeError> {
    match data.get(key) {
        Some(attribute) => match attribute.as_s() {
            Ok(string) => Ok(&string[..]),
            Err(_) => {
                let msg = "Key requires a string: ".to_string() + key;
                return Err(RuntimeError::from_string(msg))
            },
        },
        None => {
            let msg = "No value for: ".to_string() + key;
            return Err(RuntimeError::from_string(msg))
        },
    }
}

fn read_string_optional<'a>(data: &'a HashMap<String, AttributeValue>, key: &str) -> Option<&'a str> {
    match data.get(key) {
        Some(attribute) => match attribute.as_s() {
            Ok(string) => Some(&string[..]),
            Err(_) => None,
        },
        None => None,
    }
}

fn read_bool(data: &HashMap<String, AttributeValue>, key: &str) -> Result<bool, RuntimeError> {
    match data.get(key) {
        Some(attribute) => match attribute.as_bool() {
            Ok(&bool) => Ok(bool),
            Err(_) => {
                let msg = "Key requires a boolean: ".to_string() + key;
                return Err(RuntimeError::from_string(msg))
            },
        },
        None => {
            let msg = "No value for: ".to_string() + key;
            return Err(RuntimeError::from_string(msg))
        },
    }
}

fn read_integer_optional(data: &HashMap<String, AttributeValue>, key: &str) -> Option<u64> {
    match data.get(key) {
        Some(attribute) => match attribute.as_n() {
            Ok(string) => match u64::from_str_radix(&string[..], 10) {
                Ok(n) => Some(n),
                Err(_) => None,
            },
            Err(_) => None,
        },
        None => None,
    }
}

fn read_string_list<'a>(data: &'a HashMap<String, AttributeValue>, key: &str) -> Option<&'a Vec<String>> {
    match data.get(key) {
        Some(attribute) => match attribute.as_ss() {
            Ok(vec) => Some(vec),
            Err(_) => None,
        },
        None => None,
    }
}

pub trait ServerSerialize:Sized  {
    fn from_row(data: &HashMap<String, AttributeValue>) -> Result<Self, RuntimeError>;
    fn into_row(&self) -> HashMap<String, AttributeValue>;
}

impl ServerSerialize for Member {

    fn from_row(data: &HashMap<String, AttributeValue>) -> Result<Self, RuntimeError> {
        let id = read_string(data, "id")?.to_string();
        let name = read_string(data, "name")?.to_string();
        let email = read_string(data, "email")?.to_string();
        let address = read_string_optional(data, "address").map(|x| x.to_string());
        let mobile = read_integer_optional(data, "mobile");
        let subscriptions = read_string_list(data, "subscriptions").map_or_else(
            || vec![],
            |vec| vec.to_owned()
        );
        Ok(Member {
            id: Some(id),
            name,
            email,
            address,
            mobile,
            subscriptions,
        })
    }

    fn into_row(&self) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        if let Some(id) = &self.id {
            map.insert("id".to_string(), AttributeValue::S(id.clone()));
        }
        map.insert("name".to_string(), AttributeValue::S(self.name.clone()));
        map.insert("email".to_string(), AttributeValue::S(self.email.clone()));
        if let Some(address) = &self.address {
            map.insert("address".to_string(), AttributeValue::S(address.clone()));
        }
        if let Some(mobile) = &self.mobile {
            map.insert("mobile".to_string(), AttributeValue::N(mobile.to_string()));
        }
        map.insert("subscriptions".to_string(), AttributeValue::Ss(self.subscriptions.clone()));
        return map;
    }
}

impl ServerSerialize for Topic {

    fn from_row(data: &HashMap<String, AttributeValue>) -> Result<Self, RuntimeError> {
        let id = read_string(data, "id")?.to_string();
        let name = read_string(data, "name")?.to_string();
        let endpoint = read_string(data, "endpoint")?.to_string();
        let default = read_bool(data, "address")?;
        Ok(Topic {
            id: Some(id),
            name,
            endpoint,
            default,
        })
    }
    
    fn into_row(&self) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        if let Some(id) = &self.id {
            map.insert("id".to_string(), AttributeValue::S(id.clone()));
        }
        map.insert("name".to_string(), AttributeValue::S(self.name.clone()));
        map.insert("endpoint".to_string(), AttributeValue::S(self.endpoint.clone()));
        map.insert("default".to_string(), AttributeValue::Bool(self.default));
        return map;
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    details: String
}

impl RuntimeError {
    pub fn from_str(msg: &str) -> Self {
        RuntimeError{details: msg.to_string()}
    }

    pub fn from_string(msg: String) -> Self {
        RuntimeError { details: msg }
    }

    pub fn details(self) -> String {
        self.details
    } 
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl std::error::Error for RuntimeError {
    fn description(&self) -> &str {
        &self.details
    }
}

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
