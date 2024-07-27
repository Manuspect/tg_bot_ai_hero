#![allow(clippy::too_many_arguments)]

mod braille;
mod markdown;
mod session;
mod session_mgr;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Error;
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
    ChatCompletionRequestMessageContentPart, ChatCompletionRequestMessageContentPartImage,
    ChatCompletionRequestMessageContentPartText, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, ImageDetail, ImageUrl, Role,
};
use base64::Engine;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::di::DependencySupplier;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{File as TgFile, InputMedia};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Me};
use tokio::fs::File;

use crate::{
    dispatcher::noop_handler,
    env_config::SharedConfig,
    module_mgr::{Command, Module},
    modules::openai::{ChatModelResult, OpenAIClient},
    modules::{admin::MemberManager, stats::StatsManager},
    types::HandlerResult,
};
use braille::BrailleProgress;
pub(crate) use session::Session;
pub(crate) use session_mgr::SessionManager;

use super::openai::get_msg_content;

#[derive(Debug, Clone, PartialEq, Eq)]
struct MessageText(String);

async fn handle_chat_message(
    bot: Bot,
    me: Me,
    msg: Message,
    chat_id: ChatId,
    session_mgr: SessionManager,
    stats_mgr: StatsManager,
    member_mgr: MemberManager,
    openai_client: OpenAIClient,
    config: SharedConfig,
) -> bool {
    let mut text = msg.text().map_or(Default::default(), |t| t.to_owned());
    let chat_id = chat_id.to_string();

    if text.starts_with('/') {
        // Let other modules to process the command.
        return false;
    }

    // Image handling
    let img = msg.photo().map_or(Default::default(), |t| t.to_owned());
    let mut url_image = None;
    if let Some(last_photo) = img.last() {
        match bot.get_file(last_photo.file.id.clone()).send().await {
            Ok(TgFile { meta, path }) => {
                let mut buffer: Vec<u8> = Vec::new();
                match bot.download_file(&path, &mut buffer).await {
                    Ok(_) => {
                        log::info!("Downloading File: {} | Size: {} ...", path, meta.size);
                        // Determine the MIME type based on the extension
                        let mime_type =
                            match Path::new(&path).extension().and_then(|ext| ext.to_str()) {
                                Some("jpg") | Some("jpeg") => "image/jpeg",
                                Some("png") => "image/png",
                                Some("gif") => "image/gif",
                                Some("bmp") => "image/bmp",
                                _ => "application/octet-stream",
                            };
                        let encoded_image =
                            base64::engine::general_purpose::STANDARD.encode(&buffer);
                        url_image = Some(format!("data:{};base64,{}", mime_type, encoded_image));
                    }
                    Err(err) => log::warn!("{}", err),
                }
            }
            Err(err) => log::warn!("{}", err),
        }
    }

    let sender_username = msg
        .from()
        .and_then(|u| u.username.clone())
        .unwrap_or_default();
    if !member_mgr
        .is_member_allowed(sender_username.clone())
        .await
        .unwrap_or(false)
    {
        log::info!("{} allowed", sender_username);
        let _ = bot
            .send_message(msg.chat.id, &config.i18n.not_allowed_prompt)
            .reply_to_message_id(msg.id)
            .await;
        return true;
    } else {
        log::info!("{} not allowed", sender_username);
    }

    let trimmed_text = text.trim_start();
    if let Some(text_without_at) = trimmed_text.strip_prefix('@') {
        // Remove the leading mention to prevent the model from
        // being affected by it.
        let username = me.username();
        if let Some(text_without_mention) = text_without_at.strip_prefix(username) {
            text = text_without_mention.to_owned();
        }
    }
    text = text.trim().to_owned();

    let mut req_content = vec![];
    req_content.push(ChatCompletionRequestMessageContentPart::Text(
        ChatCompletionRequestMessageContentPartText { text: text },
    ));
    if let Some(url_image) = url_image {
        req_content.push(ChatCompletionRequestMessageContentPart::ImageUrl(
            ChatCompletionRequestMessageContentPartImage {
                image_url: ImageUrl {
                    url: url_image,
                    detail: Some(ImageDetail::Low),
                },
            },
        ));
    }
    let user_msg = ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Array(req_content.into()),
        name: None,
    });

    if let Err(err) = actually_handle_chat_message(
        bot,
        Some(msg),
        user_msg,
        chat_id,
        session_mgr,
        stats_mgr,
        openai_client,
        config,
    )
    .await
    {
        error!("Failed to handle chat message: {}", err);
    }

    true
}

async fn handle_retry_action(
    bot: Bot,
    query: CallbackQuery,
    session_mgr: SessionManager,
    stats_mgr: StatsManager,
    openai_client: OpenAIClient,
    config: SharedConfig,
) -> bool {
    log::info!("DA5 {:?}", query.data);
    if !query.data.map(|data| data == "/retry").unwrap_or(false) {
        return false;
    }

    let message = query.message;
    if message.is_none() {
        return false;
    }
    let message = message.unwrap();

    if let Err(err) = bot.delete_message(message.chat.id, message.id).await {
        error!("Failed to revoke the retry message: {}", err);
        return false;
    }

    let chat_id = message.chat.id.to_string();
    // let last_message = session_mgr.swap_session_pending_message(chat_id.clone(), None);
    let last_message =
        session_mgr.with_mut_session(chat_id.to_string(), |session| session.get_last_message());
    if last_message.is_none() {
        error!("Last message not found");
        return true;
    }
    let last_message = last_message.unwrap();
    if let Err(err) = actually_handle_chat_message(
        bot,
        None,
        last_message,
        chat_id,
        session_mgr,
        stats_mgr,
        openai_client,
        config,
    )
    .await
    {
        error!("Failed to retry handling chat message: {}", err);
    }

    true
}

async fn edit_message(
    user_message_content_part: ChatCompletionRequestMessageContentPart,
    bot: Bot,
    message: teloxide::types::Message,
    chat_id: teloxide::types::ChatId,
) {
    match user_message_content_part {
        async_openai::types::ChatCompletionRequestMessageContentPart::Text(content) => {
            let _ = bot
                .edit_message_text(chat_id, message.id, content.text)
                .await;
        }
        // TODO: implement image part
        async_openai::types::ChatCompletionRequestMessageContentPart::ImageUrl(image_url) => {
            let t_image = teloxide::types::InputMediaPhoto::new(
                teloxide::types::InputFile::memory(image_url.image_url.url),
            );
            let _ = bot
                .edit_message_media(chat_id, message.id, InputMedia::Photo(t_image))
                .await;
        }
    }
}

async fn handle_show_raw_action(
    bot: Bot,
    query: CallbackQuery,
    session_mgr: SessionManager,
) -> bool {
    log::info!("DA6 {:?}", query.data);
    let history_msg_id: Option<i64> = query
        .data
        .as_ref()
        .and_then(|data| data.strip_prefix("/show_raw:"))
        .and_then(|id_str| id_str.parse().ok());
    if history_msg_id.is_none() {
        return false;
    }
    let history_msg_id = history_msg_id.unwrap();

    let message = query.message;
    if message.is_none() {
        return false;
    }
    let message = message.unwrap();
    let chat_id = message.chat.id;
    let history_message = session_mgr.with_mut_session(chat_id.to_string(), |session| {
        session.get_history_message(history_msg_id)
    });
    match history_message {
        Some(history_message) => {
            let msg_content = get_msg_content(&history_message);
            match msg_content {
                super::openai::MsgContent::Text(text) => {
                    let _ = bot.edit_message_text(chat_id, message.id, text).await;
                }
                super::openai::MsgContent::ContentPart(content) => {
                    let future_res = content.into_iter().map(|content| {
                        edit_message(content, bot.clone(), message.clone(), chat_id.clone())
                    });
                    futures::future::join_all(future_res).await;
                }
            }
        }
        None => {
            let _ = bot.send_message(chat_id, "The message is stale.").await;
        }
    }

    true
}

async fn actually_handle_chat_message(
    bot: Bot,
    reply_to_msg: Option<Message>,
    content_msg: ChatCompletionRequestMessage,
    chat_id: String,
    session_mgr: SessionManager,
    stats_mgr: StatsManager,
    openai_client: OpenAIClient,
    config: SharedConfig,
) -> HandlerResult {
    // Send a progress indicator message first.
    let progress_bar = BrailleProgress::new(1, 1, 3, Some("Thinking... 🤔".to_owned()));
    let mut send_progress_msg = bot.send_message(chat_id.clone(), progress_bar.current_string());
    send_progress_msg.reply_to_message_id = reply_to_msg.as_ref().map(|m| m.id);
    let sent_progress_msg = send_progress_msg.await?;

    // Construct the request messages.
    let mut msgs = session_mgr.get_history_messages(&chat_id);
    for x in &msgs {
        log::info!("{}", get_msg_content(x).to_string());
    }
    msgs.push(content_msg.clone());

    let result = stream_model_result(
        &bot,
        &chat_id,
        &sent_progress_msg,
        progress_bar,
        msgs,
        openai_client,
        &config,
    )
    .await;

    // Record stats and add the reply to history.
    let reply_result = match result {
        Ok(res) => {
            let reply_history_message = session_mgr.with_mut_session(chat_id.clone(), |session| {
                let msg = ChatCompletionRequestAssistantMessage {
                    content: Some(res.content.clone()),
                    name: None,
                    tool_calls: None,
                    function_call: None,
                };
                session.prepare_history_message(ChatCompletionRequestMessage::Assistant(msg))
            });

            let need_fallback = if config.renders_markdown {
                let parsed_content = markdown::parse(&res.content);
                #[cfg(debug_assertions)]
                {
                    debug!(
                        "rendered Markdown contents: {}\ninto: {:#?}",
                        res.content, parsed_content
                    );
                }
                let mut edit_message_text = bot.edit_message_text(
                    chat_id.to_owned(),
                    sent_progress_msg.id,
                    parsed_content.content,
                );
                if !parsed_content.entities.is_empty() {
                    let show_raw_button = InlineKeyboardButton::callback(
                        "Show Raw Contents",
                        format!("/show_raw:{}", reply_history_message.id),
                    );
                    let retry_button = InlineKeyboardButton::callback("Retry", "/retry");
                    edit_message_text.entities = Some(parsed_content.entities);
                    edit_message_text.reply_markup = Some(
                        InlineKeyboardMarkup::default().append_row([show_raw_button, retry_button]),
                    );
                }
                if let Err(first_trial_err) = edit_message_text.await {
                    // TODO: test if the error is related to Markdown before
                    // fallback to raw contents.
                    error!(
                        "failed to send message (will fallback to raw contents): {}",
                        first_trial_err
                    );
                    true
                } else {
                    false
                }
            } else {
                true
            };

            if need_fallback {
                bot.edit_message_text(chat_id.to_owned(), sent_progress_msg.id, &res.content)
                    .await?;
            }

            session_mgr.with_mut_session(chat_id.clone(), |session| {
                let user_history_msg = session.prepare_history_message(content_msg);
                session.add_history_message(user_history_msg);
                session.add_history_message(reply_history_message);
                for x in &session.get_history_messages() {
                    log::info!("{}", get_msg_content(x).to_string());
                }
            });

            // TODO: maybe we need to handle the case that `reply_to_msg` is `None`.
            if let Some(from_username) = reply_to_msg
                .as_ref()
                .and_then(|m| m.from())
                .and_then(|u| u.username.as_ref())
            {
                let res = stats_mgr
                    .add_usage(from_username.to_owned(), res.token_usage as _)
                    .await;
                if let Err(err) = res {
                    error!("Failed to update stats: {}", err);
                }
            }
            Ok(())
        }
        Err(err) => {
            error!("Failed to request the model: {}", err);
            session_mgr.swap_session_pending_message(chat_id.clone(), Some(content_msg));
            let retry_button = InlineKeyboardButton::callback("Retry", "/retry");
            let reply_markup = InlineKeyboardMarkup::default().append_row([retry_button]);
            bot.edit_message_text(chat_id, sent_progress_msg.id, &config.i18n.api_error_prompt)
                .reply_markup(reply_markup)
                .await
                .map(|_| ())
        }
    };

    if let Err(err) = reply_result {
        error!("Failed to edit the final message: {}", err);
    }

    Ok(())
}

use async_std::io;
use async_std::prelude::*;
use std::time::Instant;

async fn stream_model_result(
    bot: &Bot,
    chat_id: &str,
    editing_msg: &Message,
    mut progress_bar: BrailleProgress,
    msgs: Vec<ChatCompletionRequestMessage>,
    openai_client: OpenAIClient,
    config: &SharedConfig,
) -> Result<ChatModelResult, Error> {
    let estimated_prompt_tokens = openai_client.estimate_prompt_tokens(&msgs);
    // log::info!("{:?}", msgs);
    let mut stream = openai_client.request_chat_model(msgs).await?;
    let mut lock = io::stdout();

    let mut timeout_times = 0;
    let mut antispam_start_time = Instant::now();
    let mut response_content = String::new();
    loop {
        tokio::select! {
            res = stream.next() => {
                // info!("{:#?}", res);
                if let Some(res) = res {
                    // let mut temp_logs: String = String::new();
                    match res {
                        Ok(response) => {
                            for chat_choice in &response.choices {
                                if let Some(ref content) = chat_choice.delta.content {
                                    response_content.push_str(content);
                                    lock.write(content.as_bytes()).await?;
                                    lock.flush().await?;
                                }
                                let elapsed_time = antispam_start_time.elapsed();
                                if elapsed_time >= Duration::from_secs(config.tg_edit_message_timeout) {
                                    progress_bar.advance_progress();

                                    let updated_text = format!("{}\n{}", response_content, progress_bar.current_string());
                                    let res = bot
                                        .edit_message_text(chat_id.to_owned(), editing_msg.id, updated_text)
                                        .await;
                                    match res {
                                        Ok(_) => {},
                                        Err(res) => {
                                            error!("Failed to edit the progress message: {}", res);
                                        }
                                    };

                                    // Reset antispam_start_time
                                    antispam_start_time = Instant::now();
                                }
                            }
                        }
                        Err(err) => {
                            response_content.push_str(&err.to_string());
                        }
                    }
                } else {
                    break
                }

                // Reset the timeout once the stream is resumed.
                timeout_times = 0;
            },
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                timeout_times += 1;
                if timeout_times >= config.openai_api_timeout {
                    return Err(anyhow!("Stream is timeout"));
                }
            }
        }
    }
    if !response_content.is_empty() {
        // TODO: OpenAI currently doesn't support to give the token usage
        // in stream mode. Therefore we need to estimate it locally.
        let last_response = ChatModelResult {
            content: response_content.clone(),
            token_usage: openai_client.estimate_tokens(&response_content) + estimated_prompt_tokens,
        };

        return Ok(last_response);
    }
    Err(anyhow!("Server returned empty response"))
}

async fn reset_session(
    bot: Bot,
    msg: Message,
    session_mgr: SessionManager,
    config: SharedConfig,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    session_mgr.reset_session(chat_id.to_string());
    let _ = bot.send_message(chat_id, &config.i18n.reset_prompt).await;
    Ok(())
}

pub(crate) struct Chat;

#[async_trait]
impl Module for Chat {
    async fn register_dependency(&mut self, dep_map: &mut DependencyMap) -> Result<(), Error> {
        let config: Arc<SharedConfig> = dep_map.get();

        dep_map.insert(SessionManager::new(config.as_ref().clone()));

        Ok(())
    }

    fn filter_handler(
        &self,
    ) -> Handler<'static, DependencyMap, HandlerResult, DpHandlerDescription> {
        dptree::entry()
            .branch(
                Update::filter_message()
                    // .filter_map(|msg: Message| msg.text().map(|text| MessageText(text.to_owned())))
                    .map(|msg: Message| msg.chat.id)
                    .branch(dptree::filter_async(handle_chat_message).endpoint(noop_handler)),
            )
            .branch(
                Update::filter_callback_query()
                    .branch(dptree::filter_async(handle_retry_action).endpoint(noop_handler))
                    .branch(dptree::filter_async(handle_show_raw_action).endpoint(noop_handler)),
            )
    }

    fn commands(&self) -> Vec<Command> {
        vec![Command::new(
            "reset",
            "Reset the current session",
            dptree::endpoint(reset_session),
        )]
    }
}
