use std::{
    fmt::Write,
    sync::{Arc, RwLock},
};
mod client;
mod parser_mgr;
mod store;

#[cfg(all(feature = "diesel"))]
pub use store::diesel::DieselChannelMessagesStore;
pub use store::memory::MemoryChannelMessagesStore;
pub use store::UserChannelMessagesStore;

use anyhow::{Error, Ok};
use teloxide::{
    dispatching::dialogue::GetChatId,
    dptree::{
        self,
        di::{DependencyMap, DependencySupplier},
    },
    payloads::SendMessageSetters,
    requests::{Request, Requester},
    types::Message,
    Bot,
};
use tokio::sync::Mutex;

use crate::{
    database::DatabaseManager,
    env_config::{self, SharedConfig},
    // handlers::client::message_handler::ClientService,
    module_mgr::{Command, Module},
    modules::parser::client::message_handler::ClientService,
    types::HandlerResult,
    utils::dptree_ext::CommandArgs,
    utils::{sign_in_tg::SignInTg, types_and_constants::NUM_COPY_LAST_MESSAGES},
};

use self::parser_mgr::ParserManager;

async fn parse_data(bot: Bot, msg: Message, parser_mgr: ParserManager) -> HandlerResult {
    let (tx_telegram_code, rx_telegram_code) = tokio::sync::mpsc::channel::<String>(100);
    {
        *parser_mgr.tx_token.lock().await = Some(tx_telegram_code.clone());
    }

    // TODO: хранить файл с сессией в зависимости от chat_id
    // TODO: тянуть по chat_id нужный логин и пароль
    // Этот функционал нужно выносить из конфига в бд

    tokio::spawn(async move {
        // Initialize a telegram user.
        let _user = SignInTg::get_user_from_client(
            bot.clone(),
            &msg.chat.id.0,
            &parser_mgr.tg_client,
            parser_mgr.config.tg_api_phone.clone(),
            parser_mgr.config.tg_api_password.expose_secret().clone(),
            rx_telegram_code,
        )
        .await
        .unwrap();
        log::info!("Got a telegram user");

        // Get the "copy from" and "paste to" chats.
        let (copy_chat, _) =
            ClientService::get_chats_by_name(&parser_mgr.tg_client, parser_mgr.config.clone())
                .await
                .unwrap();
        // Try to unwrap the chats.
        // (It is ok for the program to panic at this stage, as if the error occurs,
        // there is no way to overcome it).
        let copy_chat = copy_chat.unwrap();
        log::info!(
            "Got a telegram chat: {:#?}, {:#?}",
            copy_chat.name(),
            copy_chat.id()
        );

        ClientService::init_copy_to_db_last_messages(
            &parser_mgr.tg_client,
            &copy_chat,
            Some(NUM_COPY_LAST_MESSAGES),
            parser_mgr.clone(),
        )
        .await;

        // let paste_chat = paste_chat.unwrap();

        // // Load the last N messages from the channel, if some of them were missed.
        // ClientService::init_copy_last_messages(
        //     &tg_client,
        //     &copy_chat,
        //     &paste_chat,
        //     NUM_COPY_LAST_MESSAGES,
        //     pg_connection_pool.clone(),
        // )
        // .await;

        // Start handling messages from the trusted channel.
        let client_listener = ClientService::listen_for_incoming_messages(
            parser_mgr.tg_client.clone(),
            copy_chat.id(),
            parser_mgr.clone(),
        ); // end client_listener

        // Start handling messages from the trusted channel.
        let old_age = std::time::Duration::from_secs(32 * 24 * 60 * 60);
        let wait_time_secs: u64 = 24 * 60 * 60;

        // Run the program till it is cancelled
        tokio::select! {
            _ = async move {
                // Run a separate task to remove "old" posts from the database.
                ClientService::remove_old_messages(old_age, wait_time_secs, bot.clone(), msg.chat.id.0, parser_mgr.clone()).await;
            } => {},
            _ = async move {
                // Run a separate task with Telegram client.
                client_listener.await.unwrap();
            } => {},
            _ = tokio::signal::ctrl_c() => {}
        }; // end tokio::select!
    });

    Ok(())
}

// async fn start(bot: Bot, dialogue: ParserDialogue, msg: Message) -> HandlerResult {
//     bot.send_message(msg.chat.id, "Let's start! What's your code?")
//         .await?;
//     dialogue.update(State::ReceiveCode).await?;
//     Ok(())
// }

async fn handle_code(
    bot: Bot,
    msg: Message,
    parser_mgr: ParserManager,
    args: CommandArgs,
) -> HandlerResult {
    let code = args.0;
    if code.is_empty() || code.contains(' ') {
        bot.send_message(msg.chat.id, "Invalid username").await?;
        return Ok(());
    }

    let tx = parser_mgr.tx_token.lock().await;
    match tx.as_ref() {
        Some(tx) => {
            tx.send(code.clone()).await.unwrap();
        }
        None => {
            bot.send_message(msg.chat.id, "No token").await?;
            return Ok(());
        }
    }

    // let key = msg.chat.id.0.to_string();

    // let arc_tasks_to_join = parser_mgr.tasks_to_join.clone();
    // let mut tasks_to_join = arc_tasks_to_join.lock().await;
    // {
    //     if let Some(tasks) = tasks_to_join.get_mut(&key) {
    //         tasks.push(code);
    //     } else {
    //         tasks_to_join.insert(key.clone(), vec![code]);
    //     }
    // }

    // info!("tasks_to_join: {:?}", tasks_to_join);

    Ok(())
}

pub(crate) struct Parser {
    db_mgr: DatabaseManager,
}

impl Parser {
    pub(crate) fn new(db_mgr: DatabaseManager) -> Self {
        Self { db_mgr }
    }
}

#[async_trait]
impl Module for Parser {
    async fn register_dependency(&mut self, dep_map: &mut DependencyMap) -> Result<(), Error> {
        let config: Arc<SharedConfig> = dep_map.get();
        // Initialize a telegram client.
        let tg_client = SignInTg::get_client(config.tg_api_id, config.tg_api_hash.expose_secret())
            .await
            .expect("Failed to get a telegram client");

        log::info!("Got a telegram client");
        let parser_mgr = ParserManager::with_db_manager(
            self.db_mgr.clone(),
            config.as_ref().clone(),
            tg_client.clone(),
        )
        .await?;
        dep_map.insert(parser_mgr);

        Ok(())
    }

    fn commands(&self) -> Vec<Command> {
        vec![
            Command::new(
                "parse_data",
                "Get all data from the trusted channel and parse it",
                dptree::endpoint(parse_data),
            ),
            Command::new(
                "code",
                "pass a verification code for telegram client.",
                dptree::endpoint(handle_code),
            ),
        ]
    }
}
