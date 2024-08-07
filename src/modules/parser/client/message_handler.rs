// This file contains the main loop for incoming
// messages tracking.

use std::sync::Arc;

use grammers_client::{
    types::{Chat, Media, Message},
    Client, Update,
};
use std::path::Path;
use teloxide::{
    requests::{Request, Requester},
    Bot,
};
use tokio::sync::Mutex;

use crate::{
    env_config,
    modules::{
        config::{calculate_time, get_current_time},
        parser::{parser_mgr::ParserManager, store::ChannelMessagesBuilder},
    },
    utils::{
        app_error::AppError,
        types_and_constants::{MAX_TG_CAPTIONS_LENGTH, MAX_TG_MESSAGE_LENGTH, MEDIA_PATH},
    },
};

use serde_json::{self, Number};
use serde_json::{Map, Value};
use std::fs;

/// This struct contains all the functionality to
/// track and to handle incoming messages for a Telegram client.
pub struct ClientService;

/// Implement all the functionality to track and to handle
/// incoming messages for a Telegram client.
impl ClientService {
    pub(crate) async fn init_copy_to_db_last_messages(
        tg_client: &Client,
        copy_chat: &Chat,
        num_last_messages: Option<usize>,
        parser_mgr: ParserManager,
    ) {
        let mut messages = if let Some(n) = num_last_messages {
            tg_client.iter_messages(copy_chat).limit(n)
        } else {
            tg_client.iter_messages(copy_chat)
        };

        log::info!(
            "Chat {} has {} total messages.",
            copy_chat.name(),
            messages.total().await.unwrap()
        );

        let mut counter = 0;
        let not_found_counter = Arc::new(Mutex::new(0));

        while let Some(msg) = messages.next().await.unwrap() {
            counter = counter + 1;
            Self::save_message(
                tg_client,
                &msg,
                not_found_counter.clone(),
                parser_mgr.clone(),
                false,
            )
            .await
            .unwrap();
        }

        println!("Downloaded {} messages", counter);
    }

    /// This function save a messages to CSV file and specific path.
    /// It works even if the channel is private and the redirection is prohibited.
    /// If the "is_editing_existing_message" is set to true, then the existing
    /// message will be edited to an updated version.
    pub(crate) async fn save_message(
        tg_client: &Client,
        message: &Message,
        mut not_found_counter: Arc<Mutex<i32>>,
        parser_mgr: ParserManager,
        is_editing_existing_message: bool,
    ) -> Result<(), AppError> {
        // If message is a service message.
        if message.action().is_some() {
            // Safely unwrap the action.
            let _action = message.action().unwrap();

            // End message redirection.
            return Ok(());
        } // end if

        // Initialize the message to send to another chat.
        // let mut input_message = InputMessage::text(message.text());
        let mut input_message = Map::new();
        input_message.insert(
            "text".to_string(),
            Value::String(message.text().to_string()),
        );

        // Check if the message is longer than the normal message limit.
        if message.text().chars().count() > MAX_TG_MESSAGE_LENGTH {
            // Avoid sending this message.
            return Ok(());
        } // end if

        // Check if the message contains both media and text and the captions exceed
        // the maximum limit, then remember the text and send it as a separate message.
        if message.text().chars().count() > MAX_TG_CAPTIONS_LENGTH && message.media().is_some() {
            input_message.insert("text".to_string(), Value::String("".to_string()));
            input_message.insert(
                "leftover_text".to_string(),
                Value::String(message.text().to_string()),
            );
        } // end if

        // Check if the message contains media.
        if message.media().is_some() {
            // Safely unwrap the media.
            let message_media = message.media().unwrap();

            // Determine the type of the media before downloading it.
            // Get the path where the media will be stored.
            let media_path = match message_media {
                // Check if the media is a document.
                Media::Document(doc) => {
                    // Save the media as a document with an appropriate extension.

                    // Get the file extension.
                    let file_extension = match doc.mime_type() {
                        Some(file_type) => {
                            // Split the file type into type and extension.
                            let (_, extension) = file_type.split_once("/").unwrap();

                            extension
                        }
                        None => "",
                    }; // end match

                    format!("{}_{}.{}", MEDIA_PATH, message.id(), file_extension)
                } // end Media::Document
                // Check if the media is a sticker.
                Media::Sticker(sticker) => {
                    // If a sticker is animated, use one format.
                    if sticker.is_animated() {
                        format!("{}_{}.{}", MEDIA_PATH, message.id(), "webm")
                    } else {
                        format!("{}_{}.{}", MEDIA_PATH, message.id(), "webp")
                    } // end if
                } // end Media::Sticker
                // Check if the media is a photo.
                Media::Photo(_photo) => {
                    format!("{}_{}.{}", MEDIA_PATH, message.id(), "jpg")
                } // end Media::Photo
                _ => {
                    // Any other type of media should be ignored.
                    format!("None")
                } // end default arm
            }; // end match

            // Check if sending of the message may lead to failure,
            // because there is nothing to send.
            if media_path == "None" && message.text() == "" {
                // Ignore the message.
                return Ok(());
            } // end if

            input_message.insert(
                "media_path".to_string(),
                Value::String(media_path.to_string()),
            );
        } // end if

        // Assign some format entities to the message
        // if there are some in the message, that should be redirected.
        if message.fmt_entities().is_some() {
            // Get the format entities.
            let fmt_entities = message.fmt_entities().unwrap().to_owned();

            // Assign the format entities to the message sent.
            input_message.insert(
                "fmt_entities".to_string(),
                Value::String(format!("{:#?}", fmt_entities)),
            );
        } // end if

        // Check if the message is a reply to another message.
        if message.reply_to_message_id().is_some() {
            // Get the reply to message id from the copy chat.
            let copy_chat_reply_to_message_id = message.reply_to_message_id().unwrap();

            let paste_chat_reply_to_message_id = parser_mgr
                .get_paste_channel_message_id(copy_chat_reply_to_message_id)
                .await
                .map_err(|error| AppError::new(error.to_string()))?;

            // Assign a reply message id to the message in the "paste to" chat.
            input_message.insert(
                "reply_to".to_string(),
                Value::Number(Number::from(
                    paste_chat_reply_to_message_id
                        .unwrap_or(Some(-1))
                        .unwrap_or(-1),
                )),
            );
        } // end if

        // Check if it the editing of an old message.
        if is_editing_existing_message {
            // If it is a message editing, then send edit the existing message.

            // Try to get a corresponding message in the "paste to" chat.
            //
            // NOTE: If the corresponding message is not found, then the function
            // ends its execution.
            let edit_message_id = match parser_mgr.get_paste_channel_message_id(message.id()).await
            {
                Ok(paste_channel_message_id) => match paste_channel_message_id {
                    Some(paste_channel_message_id) => paste_channel_message_id,
                    None => {
                        // The message was not found.
                        // Ignore the message.
                        return Ok(());
                    } // end None
                }, // end Ok
                Err(error) => return Err(AppError::new(error.to_string())),
            }; // end match
            log::info!("edit message_id: {}", message.id().to_string(),);

            // TODO: edit saved message
            ClientService::update_chat_storage(message, &input_message).await?;

        // // Try to edit the message.
        // //
        // // NOTE: It may occur so, that the message was already modified and
        // //       if the content does not change, then there will be an error thrown.
        // let res = tg_client
        //     .edit_message(paste_chat, edit_message_id, input_message)
        //     .await;
        // // If there was an error, then log it.
        // if let Err(error) = res {
        //     error!("{}", error);
        // } // end if
        } else {
            // If it is a new message, then send it.

            // Check if the message was already sent, then do not send it
            // for the second time.
            let edit_message_id = match parser_mgr.get_paste_channel_message_id(message.id()).await
            {
                Ok(paste_channel_message_id) => match paste_channel_message_id {
                    Some(_) => {
                        log::info!("message_id: {} already exists", message.id().to_string(),);
                        return Ok(());
                    }
                    None => {
                        // Then it is fine.
                    } // end None
                }, // end Ok
                // If any other error occurs, then this is an issue.
                Err(error) => return Err(AppError::new(error.to_string())),
            }; // end match
            let mut not_found_counter = not_found_counter.lock().await;
            *not_found_counter += 1;

            log::info!(
                "message_id: {},  not_found_counter: {}",
                message.id().to_string(),
                not_found_counter,
            );

            // // Send the message to another chat.
            // let sent_message = tg_client
            //     .send_message(paste_chat, input_message)
            //     .await
            //     .map_err(|error| AppError::new(error.to_string()))?;
            ClientService::update_chat_storage(message, &input_message).await?;
            let time = get_current_time();
            let channel_messages = ChannelMessagesBuilder::new()
                .with_tg_copy_channel_message_id(message.id())
                .with_updated_at(time)
                .with_created_at(time)
                .build()
                .unwrap();

            parser_mgr
                .add_channel_message(channel_messages)
                .await
                .map_err(|error| AppError::new(error.to_string()))?;
        } // end if

        Ok(())
    } // end fn redirect_message

    pub async fn update_chat_storage(
        message: &Message,
        input_message: &Map<String, Value>,
    ) -> Result<(), AppError> {
        log::info!("update_chat_storage{:?}", input_message);
        let media_path = input_message.get("media_path");
        // Check if the media path is not None and the media should be
        // included into the redirected message.
        if let Some(media_path) = media_path.clone() {
            match media_path {
                Value::String(string_media_path) => {
                    if &*string_media_path == "None" {
                        return Ok(());
                    }
                    // Download the media.
                    message
                        .download_media(&string_media_path)
                        .await
                        .map_err(|error| AppError::new(error.to_string()))?;
                }
                _ => {}
            }

            // // Upload the media.
            // let uploaded_document = tg_client
            //     .upload_file(&media_path)
            //     .await
            //     .map_err(|error| AppError::new(error.to_string()))?;

            // // Get the media extension.
            // let (_, media_extension) = media_path.rsplit_once(".").unwrap_or(("", ""));

            // // Remove the file from the media folder.
            // std::fs::remove_file(media_path)
            //     .map_err(|error| AppError::new(error.to_string()))?;

            // input_message = input_message.copy_media(&message.media().unwrap());
        } // end if

        let file_path = "./data/output2.json";
        if !Path::new(file_path).exists() {
            fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .append(true)
                .open(file_path)
                .unwrap();
        }
        let history_message_json = fs::read_to_string(file_path).unwrap();

        let mut history_message = if history_message_json.len() == 0 {
            Map::new()
        } else {
            serde_json::from_str(&history_message_json).expect("string should be proper JSON")
        };

        history_message.insert(
            message.id().to_string(),
            Value::Object(input_message.clone()),
        );

        fs::write(file_path, serde_json::to_string(&history_message).unwrap()) // save result
            .expect("Can't write to file");
        Ok({})
    }

    /// This function gets both "copy from" and "paste to" chats.
    /// The function returns None, if the chat was not found.
    /// Otherwise - the chat itself.
    pub async fn get_chats_by_name(
        tg_client: &Client,
        config: env_config::SharedConfig,
    ) -> Result<(Option<Chat>, Option<Chat>), AppError> {
        // Get all chats.
        let mut all_chats = tg_client.iter_dialogs();

        // Find the necessary chats.
        let mut copy_chat: Option<Chat> = None;
        let mut paste_chat: Option<Chat> = None;

        // Traverse all available chats and find the necessary one.
        while let Some(chat) = all_chats
            .next()
            .await
            .map_err(|error| AppError::new(error.to_string()))?
        {
            // Check if the current chat is the chat that it is necessary to copy
            // all messages from.
            if chat.chat().name() == config.tg_copy_channel_name {
                // It is the chat it is necessary to copy all messages from.
                copy_chat = Some(chat.chat().clone());
            }

            // Check if the current chat is the chat that it is necessary to paste
            // all messages to.
            if chat.chat().name() == config.tg_paste_channel_name {
                // It is the chat it is necessary to paste all messages to.
                paste_chat = Some(chat.chat().clone());
            }
        } // end while

        Ok((copy_chat, paste_chat))
    } // end fn get_chat_by_id

    /// This function runs an endless loop to listen for
    /// incoming messages.
    pub(crate) async fn listen_for_incoming_messages(
        tg_client: Client,
        copy_channel_id: i64,
        parser_mgr: ParserManager,
    ) -> Result<(), AppError> {
        let not_found_counter = Arc::new(Mutex::new(0));

        // Monitor all incoming messages and select only those
        // that come from a trading channel.
        while let Some(update) = tg_client
            .next_update()
            .await
            .map_err(|error| AppError::new(error.to_string()))?
        {
            // Check for incoming messages only.
            match update {
                // A new message arrived.
                Update::NewMessage(message) if !message.outgoing() => {
                    // Make sure this is a message from the trusted channel.
                    if message.chat().id() != copy_channel_id {
                        // It is not a trusted user.
                        // Reject the request.

                        continue;
                    } // end if
                      // Otherwise, handle a message.
                    ClientService::save_message(
                        &tg_client,
                        &message,
                        not_found_counter.clone(),
                        parser_mgr.clone(),
                        false,
                    )
                    .await?;
                } // end Update::NewMessage
                Update::MessageDeleted(_message_deleted) => {} // end Update::MessageDeleted
                Update::MessageEdited(message) if !message.outgoing() => {
                    // Make sure this is a message from the trusted channel.
                    if message.chat().id() != copy_channel_id {
                        // It is not a trusted user.
                        // Reject the request.

                        continue;
                    } // end if

                    ClientService::save_message(
                        &tg_client,
                        &message,
                        not_found_counter.clone(),
                        parser_mgr.clone(),
                        true,
                    )
                    .await?;
                } // end Update::MessageEdited
                _ => {}
            } // end match
        } // end while

        Ok(())
    } // end fn listen_for_incoming_messages

    /// This function gets read of old messages from the database.
    /// This function is considered to be run as a separate asynchronous task.
    pub(crate) async fn remove_old_messages(
        age: std::time::Duration,
        wait_time_secs: u64,
        bot: Bot,
        trusted_user_id: i64,
        parser_mgr: ParserManager,
    ) {
        // This variable indicates whether or not the operation was successful.
        let mut is_success = false;

        // Endless loop that consistently removes legacy messages.
        loop {
            // Check whether or not the operation was successful.
            if is_success {
                // Reset the flag to default value.
                is_success = false;

                // Wait till the next day to perform the task.
                tokio::time::sleep(tokio::time::Duration::from_secs(wait_time_secs)).await;
            } else {
                // Wait for some time to repeat the task.
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            } // end if

            // Calculate the timestamp of an "old" post.
            let old_timestamp = calculate_time(age);

            // Try to remove the "old" messages from the database.
            match parser_mgr.remove_old_messages(old_timestamp).await {
                Ok(_) => (),
                Err(error) => {
                    error!("{}", error);

                    match bot
                        .send_message(
                            teloxide::types::ChatId(trusted_user_id),
                            "Failed to remove old posts",
                        )
                        .send()
                        .await
                    {
                        Ok(_) => (),
                        Err(error) => {
                            error!("{}", error);
                        } // end Err
                    }; // end match
                }
            } // end match

            // Indicate that the operation was completed successfully.
            is_success = true;

            // Send the message to the log channel that the operation was successful.
            // Inform the users in the log channel about the error.
            match bot
                .send_message(
                    teloxide::types::ChatId(trusted_user_id),
                    "Old posts were removed from the database",
                )
                .send()
                .await
            {
                Ok(_) => (),
                Err(error) => {
                    error!("{}", error);
                } // end Err
            }; // end match
        } // end loop
    } // end fn remove_old_messages
} // end impl ClientHandler
