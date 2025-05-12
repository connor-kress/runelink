// @generated automatically by Diesel CLI.

diesel::table! {
    users (name, domain) {
        name -> Text,
        domain -> Text,
    }
}
