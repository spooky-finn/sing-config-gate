diesel::table! {
    user (id) {
        id -> BigInt,
        username -> Text,
        status -> Integer,
        auth_key -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    vpn_uuid (id) {
        id -> BigInt,
        uuid -> Text,
        user_id -> BigInt,
        created_at -> Text,
    }
}

diesel::joinable!(vpn_uuid -> user (user_id));
diesel::allow_tables_to_appear_in_same_query!(user, vpn_uuid);
