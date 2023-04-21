pub use app_core::*;
pub use app_core::api::*;
use serde::{Deserialize, Serialize};

pub mod serialize;
pub mod runtime;
pub mod crud;

extern crate serde;
extern crate model;
extern crate aws_sdk_dynamodb;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmailRequest {
    pub topic: Topic,
    pub member: Member,
    pub email_id: String,
    pub confirm_link: bool,
}