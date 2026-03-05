// @generated automatically by Diesel CLI.

diesel::table! {
    user (id) {
        id -> BigInt,
        username -> Text,
        status -> Integer,
        created_at -> Text,
    }
}

diesel::table! {
    vless_identity (uuid) {
        uuid -> Text,
        user_id -> Nullable<BigInt>,
    }
}

diesel::joinable!(vless_identity -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(user, vless_identity,);
