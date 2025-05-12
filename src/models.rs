use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub domain: String,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub domain: &'a str,
}

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = crate::schema::messages)]
pub struct FlatMessage {
    pub id: Uuid,
    pub sender_name: Option<String>,
    pub sender_domain: Option<String>,
    pub recipient_name: Option<String>,
    pub recipient_domain: Option<String>,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub id: Uuid,
    pub sender: Option<User>,
    pub recipient: Option<User>,
    pub body: String,
}
