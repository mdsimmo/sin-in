use serde::{Deserialize, Serialize};

use crate::Member;
use crate::Topic;


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MembersDeleteRequest {
    pub id: String,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MembersDeleteResponse {
    pub id: String,
    pub old_member: Option<Member>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MembersUpdateRequest {
    pub member: Member,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MembersUpdateResponse {
    pub member: Member,
    pub old_member: Option<Member>
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MembersListRequest {

}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MembersListResponse {
    pub members: Vec<Member>,
}



#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TopicsDeleteRequest {
    pub id: String,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TopicsDeleteResponse {
    pub id: String,
    pub old_topic: Option<Topic>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TopicsUpdateRequest {
    pub topic: Topic,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TopicsUpdateResponse {
    pub topic: Topic,
    pub old_topic: Option<Topic>
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TopicsListRequest {

}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TopicsListResponse {
    pub topics: Vec<Topic>,
}