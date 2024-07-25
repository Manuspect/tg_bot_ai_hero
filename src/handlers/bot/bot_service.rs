// This file contains all the functionality necessary to interact
// with a user via Telegram bot.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};

use grammers_client::Client;
use tokio::sync::mpsc::Sender;

use teloxide::{
    adaptors::{throttle::Limits, Throttle},
    dispatching::{DefaultKey, Dispatcher, HandlerExt, UpdateFilterExt},
    dptree,
    payloads::SendMessageSetters,
    repls::CommandReplExt,
    requests::{Requester, RequesterExt, ResponseResult},
    types::{ChatId, Message, ParseMode, Update},
    utils::command::BotCommands,
    Bot, RequestError,
};
use tokio::task::JoinHandle;

use crate::{env_config, utils::app_error::AppError};

/// These are all commands that the bot can handle.
#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "start interaction with the bot.")]
    Start,
    #[command(description = "display this text.")]
    Help,
    #[command(description = "pass a verification code for telegram client.")]
    Code(String),
    #[command(description = "Service password for new trusted chat id")]
    ServicePassword(String),
} // end enum Command

pub async fn get_trusted_chat_id(tg_client: &Client, config: env_config::SharedRwConfig) -> i64 {
    let trusted_chat = tg_client
        .resolve_username(config.get().as_ref().unwrap().tg_trusted_user_name.as_str())
        .await;

    match trusted_chat {
        Ok(Some(chat)) => chat.id(),
        _ => {
            // Initialize a channel to communicate between concurrent tasks.
            let (tx_telegram_trusted_user_id, mut rx_telegram_trusted_user_id) =
                tokio::sync::mpsc::channel::<i64>(100);

            // Initialize a telegram bot.
            let bot = Arc::new(RwLock::new(
                BotService::new_bot_for_trusted_id(config.clone(), tx_telegram_trusted_user_id)
                    .await
                    .unwrap(),
            ));

            let (bot_handle, bot_shutdown_token) = BotService::spawn(bot);

            // // Set up bot listener (start listening for incoming commands)
            // let bot_command_handler =
            //     BotService::set_listener_for_new_trusted_id(&bot, tx_telegram_trusted_user_id)
            //         .await;
            // bot_command_handler.await.unwrap();

            // let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);
            let shutdown = Arc::new(AtomicBool::new(false));

            // let check_interval_secs = 2;
            // let sub_check_loop_handle = {
            //     let shutdown = shutdown.clone();
            //     tokio::task::spawn(async move {
            //         while !shutdown.load(Ordering::Acquire) {
            //             tokio::select! {
            //                _ = tokio::time::sleep(Duration::from_secs(check_interval_secs)) => {}
            //                _ = shutdown_rx.recv() => {
            //                    break
            //                }
            //             }
            //         }
            //     })
            // };

            info!("Wait for trusted_user_id");
            // Wait for a user that enter correct service password.
            let chat_id: i64 = rx_telegram_trusted_user_id.recv().await.unwrap();
            {
                let shutdown = shutdown.clone();
                shutdown.swap(true, Ordering::Relaxed);
                let _res = bot_shutdown_token.shutdown();
                // let _res = shutdown_tx.send(()).unwrap_or_else(|err| {
                //     // Makes the second Ctrl-C exit instantly
                //     info!("{:#?}", err);
                //     0
                //     // std::process::exit(0);
                // });
            }
            info!("Wait for shutdown of bot_for_trusted_id");

            if let Err(err) = tokio::try_join!(bot_handle) {
                panic!("{err}")
            }
            info!("Bot_for_trusted_id shutdown");
            chat_id
        }
    }
}

/// This structure contains all the functionality
/// necessary to drive a Telegram bot.
pub struct BotService {
    pub dispatcher: Option<Dispatcher<Arc<Throttle<Bot>>, RequestError, DefaultKey>>,
    pub tg: Arc<Throttle<Bot>>,
}

/// Implement all the functionality necessary to drive a Telegram
/// bot.
impl BotService {
    pub async fn new_bot_for_trusted_id(
        config: env_config::SharedRwConfig,
        tx_telegram_trusted_user_id: Sender<i64>,
    ) -> Result<Self, RequestError> {
        let tg = Arc::new(
            Bot::new(
                config
                    .get()
                    .as_ref()
                    .unwrap()
                    .telegram_bot_token
                    .expose_secret(),
            )
            .throttle(Limits::default()),
        );
        tg.set_my_commands(Command::bot_commands()).await?;

        let handler = Update::filter_message().branch(
            dptree::filter(
                |msg: Message, config: env_config::SharedRwConfig, sender: Sender<i64>| {
                    msg.from().map(|user| true).unwrap_or_default()
                },
            )
            .filter_command::<Command>()
            .endpoint(Self::answer_for_trusted_id),
        );

        let dispatcher = Some(
            Dispatcher::builder(tg.clone(), handler)
                .dependencies(dptree::deps![
                    config.clone(),
                    tx_telegram_trusted_user_id.clone()
                ])
                .build(),
        );

        let my_bot = BotService {
            dispatcher: dispatcher,
            tg: Arc::clone(&tg),
        };
        Ok(my_bot)
    }

    pub async fn new_bot(
        config: env_config::SharedRwConfig,
        tx_telegram_code: Sender<String>,
    ) -> Result<Self, RequestError> {
        let tg = Arc::new(
            Bot::new(
                config
                    .get()
                    .as_ref()
                    .unwrap()
                    .telegram_bot_token
                    .expose_secret(),
            )
            .throttle(Limits::default()),
        );
        tg.set_my_commands(Command::bot_commands()).await?;

        let handler = Update::filter_message().branch(
            dptree::filter(
                |msg: Message, config: env_config::SharedRwConfig, sender: Sender<String>| {
                    msg.from().map(|user| true).unwrap_or_default()
                },
            )
            .filter_command::<Command>()
            .endpoint(Self::answer),
        );

        let dispatcher = Some(
            Dispatcher::builder(tg.clone(), handler)
                .dependencies(dptree::deps![config.clone(), tx_telegram_code.clone()])
                .build(),
        );

        let my_bot = BotService {
            dispatcher: dispatcher,
            tg: Arc::clone(&tg),
        };
        Ok(my_bot)
    }

    pub fn spawn(
        bot: Arc<std::sync::RwLock<BotService>>,
    ) -> (
        tokio::task::JoinHandle<()>,
        teloxide::dispatching::ShutdownToken,
    ) {
        let shutdown_token = bot
            .read()
            .unwrap()
            .dispatcher
            .as_ref()
            .unwrap()
            .shutdown_token();
        let mut dispatcher = bot.write().unwrap().dispatcher.take().unwrap();

        (
            tokio::spawn(async move { dispatcher.dispatch().await }),
            shutdown_token,
        )
    }

    /// This function prompts a user for information.
    pub async fn inform_client(
        bot: Arc<Throttle<Bot>>,
        chat_id: i64,
        message: impl AsRef<str>,
    ) -> Result<(), AppError> {
        // Shadow chat ID for convenience.
        let chat_id = ChatId(chat_id);

        // Prompt a client for something.
        bot.send_message(chat_id, message.as_ref())
            .parse_mode(ParseMode::Html)
            .await
            .map_err(|error| AppError::new(error.to_string()))?;

        Ok(())
    } // end fn inform_client

    /// This function is a handler for bot commands.
    async fn answer_for_trusted_id(
        bot: Arc<Throttle<Bot>>,
        msg: Message,
        cmd: Command,
        config: env_config::SharedRwConfig,
        tx_telegram_trusted_user_id: Sender<i64>,
    ) -> ResponseResult<()> {
        // Handle a specific command.
        match cmd {
            Command::Start => {
                bot.send_message(msg.chat.id, "Welcome to <b>AI-Hero</b> bot!")
                    .parse_mode(ParseMode::Html)
                    .await?
            } // end Command::Start
            Command::Help => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .parse_mode(ParseMode::Html)
                    .await?
            } // end Command::Help
            Command::Code(code) => {
                bot.send_message(
                    msg.chat.id,
                    "Got the code, but trusted_user_id not set yet!",
                )
                .parse_mode(ParseMode::Html)
                .await?
            } // end Command::Code
            Command::ServicePassword(pass) => {
                // Send the verification trusted_user_id to the client authentication service.
                tx_telegram_trusted_user_id
                    .send(msg.chat.id.0)
                    .await
                    .unwrap();

                bot.send_message(msg.chat.id, "Got the password!\nTrying to log in...")
                    .parse_mode(ParseMode::Html)
                    .await?;
                bot.send_dice(msg.chat.id).await?
            } // end Command::Code
        }; // end match

        Ok(())
    }

    /// This function is a handler for bot commands.
    async fn answer(
        bot: Arc<Throttle<Bot>>,
        msg: Message,
        cmd: Command,
        config: env_config::SharedRwConfig,
        tx_telegram_code: Sender<String>,
    ) -> ResponseResult<()> {
        // Check if it is a trusted user.
        if msg.chat.id
            != ChatId(
                config
                    .get()
                    .as_ref()
                    .unwrap()
                    .tg_trusted_user_id
                    .unwrap_or(0),
            )
        {
            // This user has no rights to interact with the bot.
            bot.send_message(
                msg.chat.id,
                format!(
                    "Sorry, but you do not have a permission to interact with me, your chat id: {:#?}",
                    msg.chat.id
                ),
            )
            .await?;

            return Ok(());
        } // end if

        // Handle a specific command.
        match cmd {
            Command::Start => {
                bot.send_message(msg.chat.id, "Welcome to <b>AI-Hero</b> bot!")
                    .parse_mode(ParseMode::Html)
                    .await?
            } // end Command::Start
            Command::Help => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .parse_mode(ParseMode::Html)
                    .await?
            } // end Command::Help
            Command::Code(code) => {
                // Send the verification code to the client authentication service.
                tx_telegram_code.send(code).await.unwrap();

                bot.send_message(msg.chat.id, "Got the code!\nTrying to log in...")
                    .parse_mode(ParseMode::Html)
                    .await?;
                bot.send_dice(msg.chat.id).await?
            } // end Command::Code
            Command::ServicePassword(pass) => {
                bot.send_message(
                    msg.chat.id,
                    "Got the password, but trusted_user_id already set!",
                )
                .parse_mode(ParseMode::Html)
                .await?
            } // end Command::Code
        }; // end match

        Ok(())
    } // end impl BotService
} // end fn set_listener
