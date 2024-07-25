// @generated automatically by Diesel CLI.

diesel::table! {
    channel_messages (message_id) {
        message_id -> Integer,
        tg_copy_channel_message_id -> Nullable<Integer>,
        vk_copy_channel_message_id -> Nullable<Integer>,
        inst_copy_channel_message_id -> Nullable<Integer>,
        paste_channel_message_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    members (username) {
        username -> Text,
        disabled -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    preferences (pref_key) {
        pref_key -> Text,
        value -> Nullable<Text>,
    }
}

diesel::table! {
    token_usage (user_id, time) {
        user_id -> Text,
        time -> Timestamp,
        tokens -> BigInt,
    }
}

diesel::table! {
    user_profile (user_id) {
        user_id -> Nullable<Text>,
        subject -> Text,
        name -> Nullable<Text>,
        given_name -> Nullable<Text>,
        family_name -> Nullable<Text>,
        email -> Nullable<Text>,
        picture -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    channel_messages,
    members,
    preferences,
    token_usage,
    user_profile,
);
