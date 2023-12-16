// @generated automatically by Diesel CLI.

diesel::table! {
    channel_messages (message_id) {
        message_id -> Int4,
        tg_copy_channel_message_id -> Nullable<Int4>,
        vk_copy_channel_message_id -> Nullable<Int4>,
        inst_copy_channel_message_id -> Nullable<Int4>,
        paste_channel_message_id -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
