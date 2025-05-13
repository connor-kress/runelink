use crate::error::ApiError;
use crate::models::{FlatMessage, Message, NewUser, User};
use crate::schema::{messages, users};
use diesel::prelude::*;
use uuid::Uuid;

pub fn insert_user(
    conn: &mut PgConnection,
    new_user: &NewUser,
) -> Result<User, ApiError> {
    diesel::insert_into(users::table)
        .values(new_user)
        .get_result(conn)
        .map_err(ApiError::from)
}

pub fn get_users(conn: &mut PgConnection) -> Result<Vec<User>, ApiError> {
    users::table.load::<User>(conn).map_err(ApiError::from)
}

pub fn get_all_messages(
    conn: &mut PgConnection,
) -> Result<Vec<Message>, ApiError> {
    let messages = messages::table.load::<FlatMessage>(conn)?;
    Ok(messages.into_iter().map(Message::from).collect())
}

pub fn get_message_by_id(
    conn: &mut PgConnection,
    msg_id: Uuid,
) -> Result<Message, ApiError> {
    messages::table
        .find(msg_id)
        .get_result::<FlatMessage>(conn)
        .map(Message::from)
        .map_err(ApiError::from)
}
