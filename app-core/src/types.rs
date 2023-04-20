extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Member {
    pub id: Option<String>,
    pub name: String,
    pub email: String,
    pub address: Option<String>,
    pub mobile: Option<u64>,
    pub subscriptions: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Topic {
    pub id: Option<String>,
    pub name: String,
    pub endpoint: String,
    pub default: bool,
}