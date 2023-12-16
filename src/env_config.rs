use paste::paste;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use std::{
    collections::HashSet,
    env,
    ops::Deref,
    sync::{Arc, RwLock},
};

use dotenv::dotenv;

const CONFIG_PATH_ENV: &str = "CONFIG_PATH";
pub const DEFAULT_LIMIT: u32 = 1;

#[derive(Debug, Deserialize)]
pub struct SecretString(Secret<String>);

impl SecretString {
    pub fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

impl Default for SecretString {
    fn default() -> Self {
        SecretString(Secret::new("".to_string()))
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub authorized_user_ids: Vec<u64>,
    pub telegram_bot_token: SecretString,
    pub check_interval_secs: u64,
    #[serde(default = "default_skip_initial_send")]
    pub skip_initial_send: bool,
    pub tg_api_id: i32,
    pub tg_api_hash: SecretString,
    pub tg_api_phone: String,
    pub tg_api_password: SecretString,
    pub tg_trusted_user_name: String,
    #[serde(default = "default_tg_trusted_user_id")]
    pub tg_trusted_user_id: Option<i64>,
    pub tg_copy_channel_name: String,
    pub tg_paste_channel_name: String,
    pub postgres_user: String,
    pub postgres_password: SecretString,
    pub postgres_db: String,
    pub database_url: String,

    /// The API key of your OpenAI account.
    /// JSON key: `openaiAPIKey`
    pub openai_api_key: String,

    /// A timeout in seconds for waiting for the OpenAI server response.
    /// JSON key: `openaiAPITimeout`
    #[serde(default = "default_openai_api_timeout")]
    pub openai_api_timeout: u64,

    /// A set of usernames that represents the admin users, who can use
    /// admin commands. You must specify this field to use admin features.
    /// JSON key: `adminUsernames`
    #[serde(default)]
    pub admin_usernames: HashSet<String>,

    /// The throttle interval (in milliseconds) for sending streamed
    /// chunks back to Telegram.
    /// JSON key: `streamThrottleInterval`
    #[serde(default = "default_stream_throttle_interval")]
    pub stream_throttle_interval: u64,

    /// Maximum number of messages in a single conversation.
    /// JSON key: `conversationLimit`
    #[serde(default = "default_conversation_limit")]
    pub conversation_limit: u64,

    /// The maximum number of tokens allowed for the generated answer.
    /// JSON key: `maxTokens`
    #[serde(default)]
    pub max_tokens: Option<u16>,

    /// A boolean value that indicates whether to parse and render the
    /// markdown contents. When set to `false`, the raw contents returned
    /// from OpenAI will be displayed. This is default to `false`.
    /// JSON key: `rendersMarkdown`
    #[serde(default = "default_renders_markdown")]
    pub renders_markdown: bool,

    /// A path for storing the database, [`None`] for in-memory database.
    /// JSON key: `databasePath`
    #[serde(default)]
    pub database_path: Option<String>,

    /// Strings for I18N.
    /// JSON key: `i18n`
    #[serde(default)]
    pub i18n: I18nStrings,
}

pub fn read_config() -> Config {
    dotenv().ok();
    env::var(CONFIG_PATH_ENV)
        .map_err(|_| format!("{CONFIG_PATH_ENV} environment variable not set"))
        .and_then(|config_path| std::fs::read(config_path).map_err(|e| e.to_string()))
        .and_then(|bytes| toml::from_slice(&bytes).map_err(|e| e.to_string()))
        .unwrap_or_else(|err| {
            log::error!("failed to read config: {err}");
            std::process::exit(1);
        })
}

/// A thread-safe reference-counting object that represents
/// a [`Config`] instance.
#[derive(Debug, Clone)]
pub struct SharedRwConfig {
    pub config: Arc<RwLock<Option<Config>>>,
}

impl SharedRwConfig {
    /// Constructs a new `SharedConfig`.
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(Some(config))),
        }
    }
    pub fn get(&self) -> std::sync::RwLockReadGuard<Option<Config>> {
        self.config.read().unwrap()
    }
}

/// A thread-safe reference-counting object that represents
/// a [`Config`] instance.
#[derive(Debug, Clone)]
pub struct SharedConfig {
    config: Arc<Config>,
}

impl SharedConfig {
    /// Constructs a new `SharedConfig`.
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

impl Deref for SharedConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        return self.config.as_ref();
    }
}

pub fn update_tg_trusted_user_id(config: SharedRwConfig, trusted_user_id: i64) {
    if let Some(config) = config.config.write().unwrap().as_mut() {
        config.tg_trusted_user_id = Some(trusted_user_id);
    }
}

fn default_skip_initial_send() -> bool {
    true
}

fn default_tg_trusted_user_id() -> Option<i64> {
    None
}

/// Strings for I18N.
#[derive(Debug, Clone, Deserialize)]
pub struct I18nStrings {
    /// A text to display when there are something wrong with the OpenAI service.
    /// JSON key: `apiErrorPrompt`
    #[serde(default = "default_api_error_prompt", rename = "apiErrorPrompt")]
    pub api_error_prompt: String,
    /// A text to display when the session is reset.
    /// JSON key: `resetPrompt`
    #[serde(default = "default_reset_prompt", rename = "resetPrompt")]
    pub reset_prompt: String,
    /// A text to display when the current user is not allowed to use the bot.
    /// JSON key: `notAllowedPrompt`
    #[serde(default = "default_not_allowed_prompt", rename = "notAllowedPrompt")]
    pub not_allowed_prompt: String,
}

macro_rules! define_defaults {
    ($ty_name:ident { $($name:ident: $ty:ty = $default:expr,)* }) => {
        define_defaults! { $($name: $ty = $default,)* }
        paste! {
            impl Default for $ty_name {
                fn default() -> Self {
                    Self {
                        $($name: [<default_ $name>](),)*
                    }
                }
            }
        }
    };
    ($($name:ident: $ty:ty = $default:expr,)*) => {
        paste! {
            $(
                fn [<default_ $name>]() -> $ty {
                    $default
                }
            )*
        }
    };
}

define_defaults! {
    openai_api_timeout: u64 = 10,
    stream_throttle_interval: u64 = 500,
    conversation_limit: u64 = 20,
    renders_markdown: bool = false,
}

define_defaults!(I18nStrings {
    api_error_prompt: String = "Hmm, something went wrong...".to_owned(),
    reset_prompt: String = "\u{26A0} Session is reset!".to_owned(),
    not_allowed_prompt: String = "Sadly, you are not allowed to use this bot currently.".to_owned(),
});
