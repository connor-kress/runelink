use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub synced_at: Option<OffsetDateTime>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.domain)
    }
}

impl User {
    pub fn verbose(&self) -> String {
        format!("{} ({})", self, self.id)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewUser {
    pub name: String,
    pub domain: String,
    // pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct LocalAccount {
    pub user_id: Uuid,
    pub password_hash: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author: Option<User>,
    pub body: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewMessage {
    pub author_id: Uuid,
    pub body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewChannel {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Server {
    pub id: Uuid,
    pub domain: String,
    pub title: String,
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewServer {
    pub title: String,
    pub description: Option<String>,
    pub user_id: Uuid,
    pub user_domain: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerWithChannels {
    pub server: Server,
    pub channels: Vec<Channel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Host {
    pub domain: String,
    pub user_count: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "server_role", rename_all = "lowercase")
)]
pub enum ServerRole {
    Member,
    Admin,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerMembership {
    pub server: Server,
    pub user_id: Uuid,
    pub role: ServerRole,
    #[serde(with = "time::serde::rfc3339")]
    pub joined_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub synced_at: Option<OffsetDateTime>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerMember {
    pub user: User,
    pub role: ServerRole,
    #[serde(with = "time::serde::rfc3339")]
    pub joined_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewServerMember {
    pub user_id: Uuid,
    pub user_domain: String,
    pub role: ServerRole,
}

impl NewServerMember {
    pub fn member(user_id: Uuid, user_domain: String) -> Self {
        NewServerMember {
            user_id,
            user_domain,
            role: ServerRole::Member,
        }
    }

    pub fn admin(user_id: Uuid, user_domain: String) -> Self {
        NewServerMember {
            user_id,
            user_domain,
            role: ServerRole::Admin,
        }
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(desc) = &self.description {
            write!(f, "{} - {}", self.title, desc)
        } else {
            write!(f, "{}", self.title)
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(desc) = &self.description {
            write!(f, "#{} - {}", self.title, desc)
        } else {
            write!(f, "#{}", self.title)
        }
    }
}
