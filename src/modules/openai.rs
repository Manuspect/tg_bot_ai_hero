use std::pin::Pin;
use std::sync::Arc;

use anyhow::{Error, Ok};
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::{
    ChatCompletionRequestMessage, CreateChatCompletionRequestArgs,
    CreateChatCompletionStreamResponse,
};
use async_openai::Client;
use futures::{future, Stream, StreamExt};
use teloxide::dptree::di::{DependencyMap, DependencySupplier};

use crate::{env_config::SharedConfig, module_mgr::Module};

pub(crate) type ChatModelStream = Pin<Box<dyn Stream<Item = ChatModelResult> + Send>>;
pub(crate) type ChatModelStream1 = Pin<
    Box<
        dyn Stream<Item = Result<CreateChatCompletionStreamResponse, OpenAIError>>
            + std::marker::Send,
    >,
>;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ChatModelResult {
    pub content: String,
    pub token_usage: u32,
}

#[derive(Clone)]
pub(crate) struct OpenAIClient {
    client: Client<OpenAIConfig>,
    config: SharedConfig,
}

impl OpenAIClient {
    pub(crate) async fn request_chat_model(
        &self,
        msgs: Vec<ChatCompletionRequestMessage>,
    ) -> Result<ChatModelStream1, Error> {
        let client = &self.client;
        log::info!("{msgs:#?}");
        let req = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            // .temperature(0.6)
            .max_tokens(self.config.max_tokens.unwrap_or(4096))
            .messages(msgs)
            .build()?;
        log::info!("{req:#?}");

        let stream = client.chat().create_stream(req).await?;
        Ok(stream)
        // Ok(stream
        //     .scan(ChatModelResult::default(), |acc, cur| {
        //         let content = cur
        //             .as_ref()
        //             .ok()
        //             .and_then(|resp| resp.choices.first())
        //             .and_then(|choice| choice.delta.content.as_ref());
        //         if let Some(content) = content {
        //             acc.content.push_str(content);
        //         }
        //         future::ready(Some(acc.clone()))
        //     })
        //     .boxed())
    }

    pub(crate) fn estimate_prompt_tokens(&self, msgs: &Vec<ChatCompletionRequestMessage>) -> u32 {
        let mut text_len = 0;
        for msg in msgs {
            let content = get_msg_content(msg);
            text_len += content.len();
        }
        ((text_len as f64) * 1.4) as _
    }

    pub(crate) fn estimate_tokens(&self, text: &str) -> u32 {
        let text_len = text.len();
        ((text_len as f64) * 1.4) as _
    }
}

pub(crate) fn get_msg_content(msg: &ChatCompletionRequestMessage) -> String {
    match msg {
        ChatCompletionRequestMessage::System(msg) => msg.content.clone().unwrap_or("".to_string()),
        ChatCompletionRequestMessage::User(msg) => {
            if let Some(user_message_content) = msg.content.clone() {
                match user_message_content {
                    async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        content,
                    ) => content,
                    async_openai::types::ChatCompletionRequestUserMessageContent::Array(
                        user_message_content_array,
                    ) => {
                        user_message_content_array.into_iter().map(
                        |user_message_content_part| {
                            match user_message_content_part {
                                async_openai::types::ChatCompletionRequestMessageContentPart::Text(content) => {
                                    content.text
                                },
                                async_openai::types::ChatCompletionRequestMessageContentPart::Image(_) => "".to_string(),
                            }
                        }).reduce(|acc, s| format!("{acc}{s}")).unwrap_or("".to_string())
                    }
                }
            } else {
                "".to_string()
            }
        }
        ChatCompletionRequestMessage::Assistant(msg) => {
            msg.content.clone().unwrap_or("".to_string())
        }
        ChatCompletionRequestMessage::Tool(msg) => msg.content.clone().unwrap_or("".to_string()),
        ChatCompletionRequestMessage::Function(msg) => {
            msg.content.clone().unwrap_or("".to_string())
        }
    }
}
pub(crate) struct OpenAI;

#[async_trait]
impl Module for OpenAI {
    async fn register_dependency(&mut self, dep_map: &mut DependencyMap) -> Result<(), Error> {
        let config: Arc<SharedConfig> = dep_map.get();
        let openai_client = OpenAIClient {
            client: Client::with_config(
                OpenAIConfig::new().with_api_key(config.openai_api_key.clone()),
            ),
            config: config.as_ref().clone(),
        };
        dep_map.insert(openai_client);

        Ok(())
    }
}
