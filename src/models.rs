use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub name: String,
    pub domain: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub synced_at: Option<OffsetDateTime>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewUser {
    pub name: String,
    pub domain: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author: Option<Json<User>>,
    pub body: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewMessage {
    pub channel_id: Uuid,
    pub author_name: String,
    pub author_domain: String,
    pub body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Channel {
    pub id: Uuid,
    pub server_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Server {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerWithChannels {
    pub server: Server,
    pub channels: Vec<Channel>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Host {
    pub domain: String,
    pub user_count: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
