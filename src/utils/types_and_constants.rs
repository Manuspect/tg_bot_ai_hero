// This file contains some custom types.

// This is a session file with an authentication token for Telegram client.
pub const SESSION_FILE: &str = "./data/tg.session";

// This is a path to the downloaded media (if the message forwarding is restricted).
pub const MEDIA_PATH: &str = "./data/media/media_file";

// This is a number of last messages to load from the "copy from" chat
// when the program starts.
pub const NUM_COPY_LAST_MESSAGES: usize = 20;

// Max number of character in a telegram message.
pub const MAX_TG_MESSAGE_LENGTH: usize = 4096;

// Max number of characters in telegram captions.
pub const MAX_TG_CAPTIONS_LENGTH: usize = 1024;
