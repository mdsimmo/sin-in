extern crate serde;
extern crate model;
extern crate aws_sdk_dynamodb;
use std::collections::HashMap;
use aws_sdk_dynamodb::model::AttributeValue;
use serde::{Deserialize, Serialize};
use lambda_http::http;
use serde_json::json;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Member {
    pub id: Option<String>,
    pub name: String,
    pub email: String,
    pub address: Option<String>,
    pub mobile: Option<u64>,
}

impl Member {

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

    fn read_integer_optional<'a>(data: &'a HashMap<String, AttributeValue>, key: &str) -> Option<u64> {
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

    pub fn from_row(data: &HashMap<String, AttributeValue>) -> Result<Self, RuntimeError> {
        let id = Member::read_string(data, "id")?.to_string();
        let name = Member::read_string(data, "name")?.to_string();
        let email = Member::read_string(data, "email")?.to_string();
        let address = Member::read_string_optional(data, "name").map(|x| x.to_string());
        let mobile = Member::read_integer_optional(data, "mobile");
        Ok(Member {
            id: Some(id),
            name,
            email,
            address,
            mobile,
        })
    }
}

impl From<&Member> for HashMap<String, AttributeValue> {
    // TODO generate this using a macro
    fn from(member: &Member) -> Self {
        let mut map = HashMap::new();
        map.insert("name".to_string(), AttributeValue::S(member.name.clone()));
        map.insert("email".to_string(), AttributeValue::S(member.email.clone()));
        if let Some(address) = &member.address {
            map.insert("address".to_string(), AttributeValue::S(address.clone()));
        }
        if let Some(mobile) = &member.mobile {
            map.insert("mobile".to_string(), AttributeValue::N(mobile.to_string()));
        }
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


// This function doesn't work properly yet...
pub fn wrap_error<F>(handler: F) -> impl Fn(lambda_http::Request)->http::Result<StringResponse>
    where 
        F: Fn(lambda_http::Request)->Result<StringResponse, lambda_http::Error>,
{
    move |event: lambda_http::Request| {
        let result = handler(event);
        let reponse = match result {
            Ok(r) => Ok(r),
            Err(e) => {
                http::Response::builder()
                    .status(http::StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(json!({
                        "error": e.to_string(), 
                    }).to_string())
            }
        };
        reponse
    }
}