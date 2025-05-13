use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub name: String,
    pub domain: String,
}

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub name: String,
    pub domain: String,
}

#[derive(Debug, FromRow)]
pub struct FlatMessage {
    pub id: Uuid,
    pub sender_name: Option<String>,
    pub sender_domain: Option<String>,
    pub recipient_name: Option<String>,
    pub recipient_domain: Option<String>,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub sender: Option<User>,
    pub recipient: Option<User>,
    pub body: String,
}

impl From<FlatMessage> for Message {
    fn from(value: FlatMessage) -> Self {
        let sender = value
            .sender_name
            .zip(value.sender_domain)
            .map(|(name, domain)| User { name, domain });
        let recipient = value
            .recipient_name
            .zip(value.recipient_domain)
            .map(|(name, domain)| User { name, domain });
        Message {
            id: value.id,
            sender,
            recipient,
            body: value.body,
        }
    }
}
