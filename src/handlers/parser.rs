use std::sync::{Arc, RwLock};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

use crate::{
    env_config,
    handlers::{bot::bot_service::BotService, client::message_handler::ClientService},
    utils::{sign_in_tg::SignInTg, types_and_constants::NUM_COPY_LAST_MESSAGES},
};

pub async fn parse_data(
    config: env_config::SharedRwConfig,
    tg_client: &grammers_client::Client,
    trusted_user_id: i64,
    pg_connection_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
) {
    // Initialize a channel to communicate between concurrent tasks.
    let (tx_telegram_code, rx_telegram_code) = tokio::sync::mpsc::channel::<String>(100);

    // Initialize a telegram bot.
    let bot = Arc::new(RwLock::new(
        BotService::new_bot(config.clone(), tx_telegram_code)
            .await
            .unwrap(),
    ));

    // Set up bot listener (start listening for incoming commands)
    let (bot_handle, bot_shutdown_token) = BotService::spawn(Arc::clone(&bot));

    // Initialize a telegram user.
    let _user = SignInTg::get_user_from_client(
        Arc::clone(&bot.read().unwrap().tg),
        &trusted_user_id,
        &tg_client,
        config.clone(),
        rx_telegram_code,
    )
    .await
    .unwrap();

    log::info!("Got a telegram user");

    // Get the "copy from" and "paste to" chats.
    let (copy_chat, _) = ClientService::get_chats_by_name(&tg_client, config.clone())
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
        &tg_client,
        &copy_chat,
        Some(NUM_COPY_LAST_MESSAGES),
        pg_connection_pool.clone(),
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
        tg_client.clone(),
        copy_chat.id(),
        pg_connection_pool.clone(),
    ); // end client_listener

    // Determine the age of an "old messages" post.
    let old_age = chrono::Duration::days(32);

    // Determine the frequency to remove "old messages" post with (in seconds).
    let wait_time_secs: u64 = 24 * 60 * 60;

    // Clone the bot instance for "Legacy message remover".
    let bot_clone = bot.clone();

    // Run the program till it is cancelled
    tokio::select! {
        _ = async move {
            // Run a separate task with bot command listener.
            bot_handle.await.unwrap();
        } => {},
        _ = async move {
            // Run a separate task with Telegram client.
            client_listener.await.unwrap();
        } => {},
        _ = async move {
            // Run a separate task to remove "old" posts from the database.
            ClientService::remove_old_messages(pg_connection_pool, old_age, wait_time_secs, Arc::clone(&bot_clone.read().unwrap().tg), trusted_user_id).await;
        } => {},
        _ = tokio::signal::ctrl_c() => {}
    } // end tokio::select!
}
