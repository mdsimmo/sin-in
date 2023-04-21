pub use app_core::*;
pub use app_core::api::*;

extern crate serde;
extern crate model;
extern crate aws_sdk_dynamodb;
use std::collections::HashMap;
use aws_sdk_dynamodb::types::AttributeValue;

use crate::RuntimeError;

pub trait ServerSerialize:Sized  {
    fn from_row(data: &HashMap<String, AttributeValue>) -> Result<Self, RuntimeError>;
    fn into_row(&self) -> HashMap<String, AttributeValue>;

    fn id(&self) -> Option<&str>;
    fn set_id(&mut self, id: String) -> &mut Self;
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

    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|id| &id[..])
    }

    fn set_id(&mut self, id: String) -> &mut Self {
        self.id = Some(id);
        self
    }
}

impl ServerSerialize for Topic {

    fn from_row(data: &HashMap<String, AttributeValue>) -> Result<Self, RuntimeError> {
        let id = read_string(data, "id")?.to_string();
        let name = read_string(data, "name")?.to_string();
        let endpoint = read_string(data, "endpoint")?.to_string();
        let default = read_bool(data, "default")?;
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

    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|id| &id[..])
    }

    fn set_id(&mut self, id: String) -> &mut Self {
        self.id = Some(id);
        self
    }
}

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