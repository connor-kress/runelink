use crate::channel::Channel;
use crate::user::User;

use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;
use uuid::Uuid;

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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerWithChannels {
    pub server: Server,
    pub channels: Vec<Channel>,
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

impl Server {
    pub fn verbose(&self) -> String {
        format!("{} ({})", self.title, self.id)
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
