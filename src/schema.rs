// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Int4,
        #[max_length = 255]
        author -> Varchar,
        title -> Text,
        text -> Text,
        created_at -> Text,
        edited -> Bool,
        edited_at -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        created_at -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
