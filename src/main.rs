use std::sync::{Arc, RwLock};

use ai_hero::env_config::{self, SharedConfig, SharedRwConfig};
use flexi_logger::{Age, Cleanup, Criterion, FileSpec, Logger, Naming, WriteMode};

use ai_hero::{
    handlers::bot::bot_service::get_trusted_chat_id,
    utils::{database_service::DatabaseService, sign_in_tg::SignInTg},
};

#[tokio::main]
async fn main() {
    // Set up logger.
    let _logger = Logger::try_with_str("info")
        .unwrap()
        .log_to_file(FileSpec::default().directory("./data/logs").basename("log"))
        .log_to_stdout()
        .write_mode(WriteMode::BufferAndFlush)
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(10),
        )
        .append()
        .start()
        .unwrap();

    let config = SharedRwConfig::new(env_config::read_config());

    log::info!("Starting... {config:#?}");

    // Initialize a database connection pool.
    let pg_connection_pool = Arc::new(DatabaseService::create_database_connection_pool(
        config.clone(),
    ));

    // Initialize a telegram client.
    let tg_client = SignInTg::get_client(
        config.get().as_ref().unwrap().tg_api_id,
        config.get().as_ref().unwrap().tg_api_hash.expose_secret(),
    )
    .await
    .expect("Failed to get a telegram client");

    log::info!("Got a telegram client");

    let trusted_user_id = get_trusted_chat_id(&tg_client, config.clone()).await;

    env_config::update_tg_trusted_user_id(config.clone(), trusted_user_id);

    log::info!("Got a telegram trusted_user_id: {}", trusted_user_id);

    // ai_hero::handlers::parser::parse_data(
    //     config.clone(),
    //     &tg_client,
    //     trusted_user_id,
    //     Arc::clone(&pg_connection_pool),
    // )
    // .await;

    if let Some(config) = config.config.write().unwrap().take() {
        let config = SharedConfig::new(config);
        ai_hero::app::run(config).await;
    } else {
        log::error!("config not esxists");
    };
} // end fn main
