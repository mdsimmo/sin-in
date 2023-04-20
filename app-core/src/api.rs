use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeleteRequest {
    pub ids: Vec<String>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeleteResponse<T> {
    pub removed: Vec<Option<T>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdateRequest<T> {
    pub values: Vec<T>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdateStatus<T> {
    pub replaced: Option<T>,
    pub current: T,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdateResponse<T> {
    pub updates: Vec<UpdateStatus<T>>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ListRequest {

}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
}