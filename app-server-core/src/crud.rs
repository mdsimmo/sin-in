use std::collections::HashMap;

use app_core::api::{ListResponse, ListRequest, DeleteResponse, DeleteRequest, UpdateRequest, UpdateResponse, UpdateStatus};
use aws_sdk_dynamodb::{Client, types::{AttributeValue, ReturnValue}};
use lambda_http::{Error, aws_lambda_events::chrono};
use rand::Rng;

use crate::{serialize::ServerSerialize, RuntimeError};


pub async fn list_items<T: ServerSerialize>(_event: ListRequest, table: &str) -> Result<ListResponse<T>, Error> {
    // Get all members in dynamodb
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let table_response = client.scan()
        .table_name(table)
        .send().await;
    let table_response = table_response?;

    // Convert json into members
    let result = table_response.items()
        .map(|items| {
            let items: Vec<T> = items.into_iter().filter_map(|row| {
                match T::from_row(row) {
                    Ok(m) => Some(m),
                    Err(_err) => None,
                }
            }).collect();
            items
        });
    
    // TODO Handle the case of None
    let items = match result {
        Some(x) => x,
        None => return Err(Box::new(RuntimeError::from_str("Scan resulted in None? Why would that happen?"))),
    };
    
    Ok(ListResponse {
        items,
    })
}

pub async fn delete_items<T: ServerSerialize>(input: DeleteRequest, table: &str) -> Result<DeleteResponse<T>, Error> {

    let mut removed = vec![];

    for id in input.ids {
        let request_map = {
            let mut map = HashMap::new();
            map.insert("id".to_string(), AttributeValue::S(id.clone()));
            map
        };

        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        let table_response = client.delete_item()
            .table_name(table)
            .set_key(Some(request_map))
            .return_values(ReturnValue::AllOld)
            .send().await;
        let table_response = table_response?;

        // Read the old member (if any)
        let old_item = table_response.attributes()
            .map(|attributes| T::from_row(attributes));
        let old_item = match old_item {
            Some(Ok(m)) => Some(m),
            _ => None,
        };
        removed.push(old_item)
    }
    
    Ok(DeleteResponse {
        removed,
    })
}

pub async fn update_items<T:ServerSerialize>(input: UpdateRequest<T>, table_name: &str) -> Result<UpdateResponse<T>, Error> {
    let mut results = vec![];

    for mut item in input.values {

        // If no id assigned, assign one
        if item.id() == None {
            let id_time = chrono::Utc::now();
            let id_random = rand::thread_rng().gen::<u32>();
            let mut id_string = id_time.format("%Y-%m-%d-%H:%M:%S-").to_string();
            id_string.push_str(&id_random.to_string());
            item.set_id(id_string);
        };

        // Put the member in dynamodb
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        let table_response = client.put_item()
            .table_name(table_name)
            .set_item(Some(item.into_row()))
            .return_values(ReturnValue::AllOld)
            .send().await;
        let table_response = table_response?;

        // Read the old member (if any)
        let old_item = table_response.attributes()
            .map(|attributes| T::from_row(attributes));
        let old_item = match old_item {
            Some(Ok(m)) => Some(m),
            _ => None,
        };

        results.push(UpdateStatus {
            replaced: old_item,
            current: item,
        });
    }

    Ok(UpdateResponse {
        updates: results,
    })
}