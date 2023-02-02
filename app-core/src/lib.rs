extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Member {
    name: String,
    last_name: String,
    address: Option<String>,
    mobile: Option<String>,
}
