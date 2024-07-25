-- Your SQL goes here

-- Create table.
CREATE TABLE IF NOT EXISTS channel_messages(
    message_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    tg_copy_channel_message_id INT,
    vk_copy_channel_message_id INT,
    inst_copy_channel_message_id INT,
    paste_channel_message_id INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
); -- end create table channel_messages