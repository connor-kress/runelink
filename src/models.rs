use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub name: String,
    pub domain: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
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
    pub sender_created_at: Option<OffsetDateTime>,
    pub recipient_name: Option<String>,
    pub recipient_domain: Option<String>,
    pub recipient_created_at: Option<OffsetDateTime>,
    pub body: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub sender: Option<User>,
    pub recipient: Option<User>,
    pub body: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<FlatMessage> for Message {
    fn from(value: FlatMessage) -> Self {
        let sender = value
            .sender_name
            .zip(value.sender_domain)
            .zip(value.sender_created_at)
            .map(|((name, domain), created_at)| User {
                name,
                domain,
                created_at,
            });
        let recipient = value
            .recipient_name
            .zip(value.recipient_domain)
            .zip(value.recipient_created_at)
            .map(|((name, domain), created_at)| User {
                name,
                domain,
                created_at,
            });
        Message {
            id: value.id,
            sender,
            recipient,
            body: value.body,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
