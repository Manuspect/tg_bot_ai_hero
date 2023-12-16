-- Your SQL goes here

-- Create table.
CREATE TABLE channel_messages(
    message_id SERIAL PRIMARY KEY,
    tg_copy_channel_message_id INT,
    vk_copy_channel_message_id INT,
    inst_copy_channel_message_id INT,
    paste_channel_message_id INT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
); -- end create table channel_messages