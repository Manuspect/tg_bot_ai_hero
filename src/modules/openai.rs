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
use futures::Stream;
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
        let req = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o")
            // .temperature(0.6)
            .max_tokens(self.config.max_tokens.unwrap_or(4096))
            .messages(msgs)
            .build()?;

        let stream = client.chat().create_stream(req).await?;
        Ok(stream)
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

#[derive(Debug)]
pub enum MsgContent {
    Text(String),
    ContentPart(Vec<async_openai::types::ChatCompletionRequestMessageContentPart>),
}

impl MsgContent {
    fn len(self) -> usize {
        match self {
            MsgContent::Text(str) => str.len(),
            MsgContent::ContentPart(content) => {
                content
                    .into_iter()
                    .map(|user_message_content_part| {
                        match user_message_content_part {
                        async_openai::types::ChatCompletionRequestMessageContentPart::Text(
                            content,
                        ) => content.text.clone(),
                        // TODO: implement image part
                        async_openai::types::ChatCompletionRequestMessageContentPart::ImageUrl(
                            image_url,
                        ) => format!("{:?}", image_url.image_url.url.clone())
                    }
                    })
                    .reduce(|acc, s| format!("{:?}{:?}", acc, s))
                    .unwrap_or("".to_string())
                    .len()
            }
        }
    }
}

impl ToString for MsgContent {
    fn to_string(&self) -> String {
        match self {
            MsgContent::Text(str) => str.clone(),
            MsgContent::ContentPart(user_message_content_array) => {
                user_message_content_array
                    .into_iter()
                    .map(|user_message_content_part| {
                        match user_message_content_part {
                        async_openai::types::ChatCompletionRequestMessageContentPart::Text(
                            content,
                        ) => content.text.clone(),
                        // TODO: implement image part
                        async_openai::types::ChatCompletionRequestMessageContentPart::ImageUrl(
                            image_url,
                        ) => format!("{:.5}", image_url.image_url.url.clone())
                    }
                    })
                    .reduce(|acc, s| format!("{:?}\n{:?}", acc, s))
                    .unwrap_or("".to_string())
            }
        }
        .clone()
    }
}

// impl std::fmt::Display for MsgContent {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         // let results = get_msg_content(&self.0);
//         let results = match self {
//             MsgContent::Text(str) => str,
//             MsgContent::ContentPart(user_message_content_array) => {
//                 &user_message_content_array
//                     .into_iter()
//                     .map(|user_message_content_part| {
//                         match user_message_content_part {
//                         async_openai::types::ChatCompletionRequestMessageContentPart::Text(
//                             content,
//                         ) => content.text.clone(),
//                         // TODO: implement image part
//                         async_openai::types::ChatCompletionRequestMessageContentPart::ImageUrl(
//                             image_url,
//                         ) => format!("{:.20?}", image_url.image_url.url.clone())
//                     }
//                     })
//                     .reduce(|acc, s| format!("{:?}\n{:?}", acc, s))
//                     .unwrap_or("".to_string())
//             }
//         };
//         write!(f, "\n{}", results)
//     }
// }

pub(crate) fn get_msg_content(msg: &ChatCompletionRequestMessage) -> MsgContent {
    match msg {
        ChatCompletionRequestMessage::System(msg) => MsgContent::Text(msg.content.clone()),
        ChatCompletionRequestMessage::User(msg) => match msg.content.clone() {
            async_openai::types::ChatCompletionRequestUserMessageContent::Text(content) => {
                MsgContent::Text(content)
            }
            async_openai::types::ChatCompletionRequestUserMessageContent::Array(
                user_message_content_array,
            ) => MsgContent::ContentPart(user_message_content_array),
        },
        ChatCompletionRequestMessage::Assistant(msg) => {
            MsgContent::Text(msg.content.clone().unwrap_or("".to_string()))
        }
        ChatCompletionRequestMessage::Tool(msg) => MsgContent::Text(msg.content.clone()),
        ChatCompletionRequestMessage::Function(msg) => {
            MsgContent::Text(msg.content.clone().unwrap_or("".to_string()))
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
