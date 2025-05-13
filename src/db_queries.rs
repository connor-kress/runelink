use crate::models::{FlatMessage, Message, NewUser, User};
use crate::schema::{messages, users};
use diesel::prelude::*;
use uuid::Uuid;

pub fn insert_user(
    conn: &mut PgConnection,
    new_user: &NewUser,
) -> QueryResult<User> {
    diesel::insert_into(users::table)
        .values(new_user)
        .get_result::<User>(conn)
}

pub fn get_users(conn: &mut PgConnection) -> QueryResult<Vec<User>> {
    users::table.load::<User>(conn)
}

fn unflatten(msg: FlatMessage) -> Message {
    let sender = msg
        .sender_name
        .zip(msg.sender_domain)
        .map(|(name, domain)| User { name, domain });

    let recipient = msg
        .recipient_name
        .zip(msg.recipient_domain)
        .map(|(name, domain)| User { name, domain });

    Message {
        id: msg.id,
        sender,
        recipient,
        body: msg.body,
    }
}

pub fn get_all_messages(conn: &mut PgConnection) -> QueryResult<Vec<Message>> {
    let flat_messages = messages::table.load::<FlatMessage>(conn)?;
    Ok(flat_messages.into_iter().map(unflatten).collect())
}

pub fn get_message_by_id(
    conn: &mut PgConnection,
    msg_id: Uuid,
) -> QueryResult<Message> {
    let flat_message = messages::table
        .find(msg_id)
        .get_result::<FlatMessage>(conn)?;
    Ok(unflatten(flat_message))
}
