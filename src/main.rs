use ai_hero::env_config::{self, SharedConfig, SharedRwConfig};
use ai_hero::subcommands::error::CliError;
use flexi_logger::{DeferredNow, LogSpecBuilder, Logger};
use log::*;

// use ai_hero::{
//     handlers::bot::bot_service::get_trusted_chat_id,
//     utils::{database_service::DatabaseService, sign_in_tg::SignInTg},
// };

use log::error;
use log::Record;
use std::ffi::OsString;

fn setup_logging(log_level: log::LevelFilter) -> Result<(), CliError> {
    let mut log_spec_builder = LogSpecBuilder::new();
    log_spec_builder.default(log_level);
    log_spec_builder.module("reqwest", log::LevelFilter::Warn);
    log_spec_builder.module("hyper", log::LevelFilter::Warn);
    log_spec_builder.module("mio", log::LevelFilter::Warn);
    log_spec_builder.module("want", log::LevelFilter::Warn);

    match Logger::with(log_spec_builder.build())
        .log_to_stdout()
        .start()
    {
        Ok(_) => {}
        #[cfg(test)]
        // `FlexiLoggerError::Log` means the logger has already been initialized; this will happen
        // when `run` is called more than once in the tests.
        Err(flexi_logger::FlexiLoggerError::Log(_)) => {}
        Err(err) => panic!("Failed to start logger: {}", err),
    }

    Ok(())
}

// log format for cli that will only show the log message
pub fn log_format(
    w: &mut dyn std::io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(w, "{}", record.args(),)
}

async fn start<I: IntoIterator<Item = T>, T: Into<OsString> + Clone>(args: I) {
    // // Set up logger.
    // let _logger = Logger::try_with_str("info")
    //     .unwrap()
    //     .log_to_file(FileSpec::default().directory("./data/logs").basename("log"))
    //     .log_to_stdout()
    //     .write_mode(WriteMode::BufferAndFlush)
    //     .rotate(
    //         Criterion::Age(Age::Day),
    //         Naming::Timestamps,
    //         Cleanup::KeepLogFiles(10),
    //     )
    //     .append()
    //     .start()
    //     .unwrap();

    // // Initialize a database connection pool.
    // let pg_connection_pool = Arc::new(DatabaseService::create_database_connection_pool(
    //     config.clone(),
    // ));

    // // Initialize a telegram client.
    // let tg_client = SignInTg::get_client(
    //     config.get().as_ref().unwrap().tg_api_id,
    //     config.get().as_ref().unwrap().tg_api_hash.expose_secret(),
    // )
    // .await
    // .expect("Failed to get a telegram client");

    // info!("Got a telegram client");

    // let trusted_user_id = get_trusted_chat_id(&tg_client, config.clone()).await;

    // env_config::update_tg_trusted_user_id(config.clone(), trusted_user_id);

    // info!("Got a telegram trusted_user_id: {}", trusted_user_id);

    // // ai_hero::handlers::parser::parse_data(
    // //     config.clone(),
    // //     &tg_client,
    // //     trusted_user_id,
    // //     Arc::clone(&pg_connection_pool),
    // // )
    // // .await;

    setup_logging(log::LevelFilter::Info).unwrap();
    let config = SharedRwConfig::new(env_config::read_config());
    info!("Starting... {config:#?}");
    if let Some(config) = config.config.write().unwrap().take() {
        let config = SharedConfig::new(config);
        ai_hero::app::run(config).await;
    } else {
        error!("config not esxists");
    };
} // end fn main

#[tokio::main]
async fn main() {
    start(std::env::args_os()).await;
}

// Deprecated:
// let mut app = clap_app!(myapp =>
//     (name: APP_NAME)
//     (version: VERSION)
//     (author: "KirSR")
//     (about: "Command line for Ai Hero")
//     (@arg verbose: -v +multiple +global "Log verbosely")
//     (@arg quiet: -q --quiet +global "Do not display output")
//     (@setting SubcommandRequiredElseHelp)
// );
// #[cfg(feature = "database")]
// {
//     app = app.subcommand(
//         SubCommand::with_name("database")
//             .about("Database commands")
//             .setting(AppSettings::SubcommandRequiredElseHelp)
//             .subcommand(
//                 SubCommand::with_name("migrate")
//                     .about("Runs database migrations Splinter")
//                     .arg(
//                         Arg::with_name("connect")
//                             .short("C")
//                             .takes_value(true)
//                             .help("Database connection URI"),
//                     ),
//             ),
//     );
// }

// let matches = match app.get_matches_from_safe(args) {
//     Ok(matches) => matches,
//     Err(err) => {
//         error!("{}", err);
//         process::exit(1);
//     }
// };

// let log_level = match matches.occurrences_of("verbose") {
//     0 => log::LevelFilter::Warn,
//     1 => log::LevelFilter::Info,
//     2 => log::LevelFilter::Debug,
//     _ => log::LevelFilter::Trace,
// };
// setup_logging(log_level).unwrap();
// let config = SharedRwConfig::new(env_config::read_config());

// info!("Starting... {config:#?}");

// let mut subcommands = SubcommandActions::new().with_command(
//     "database",
//     SubcommandActions::new().with_command("migrate", subcommands::MigrateAction),
// );

// subcommands.run(Some(&matches)).unwrap_or_else(|e| {
//     error!("{}", e);
//     process::exit(1);
// });
