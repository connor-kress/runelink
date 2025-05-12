// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Uuid,
        sender_name -> Nullable<Text>,
        sender_domain -> Nullable<Text>,
        recipient_name -> Nullable<Text>,
        recipient_domain -> Nullable<Text>,
        body -> Text,
    }
}

diesel::table! {
    users (name, domain) {
        name -> Text,
        domain -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    users,
);
