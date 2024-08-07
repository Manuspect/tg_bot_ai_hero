// This file contains the functionality necessary to
// log in into the user's telegram account.

use anyhow::Error;
use grammers_client::{types::User, Client, Config, SignInError};
use grammers_session::Session;
use std::io::{BufRead, Write};
use std::path::Path;
use std::sync::Arc;
use std::{env, io};
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::{Request, Requester};
use teloxide::{adaptors::Throttle, Bot};
use tokio::sync::mpsc::Receiver;

use crate::env_config;
use crate::types::HandlerResult;

use super::{app_error::AppError, types_and_constants::SESSION_FILE};

/// This structure contains all the functionality necessary to
/// log into the user's telegram account.
pub struct SignInTg;

/// Implement all the functionality for the struct.
impl SignInTg {
    /// This function returns a client for the user.
    pub async fn get_client(api_id: i32, api_hash: &String) -> Result<Client, AppError> {
        Ok(Client::connect(Config {
            session: Session::load_file_or_create(SESSION_FILE)
                .map_err(|error| AppError::new(error.to_string()))?,
            api_id,
            api_hash: api_hash.clone(),
            params: Default::default(),
        })
        .await
        .map_err(|error| AppError::new(error.to_string()))?)
    } // end fn get_client

    /// This function gets a user from a client.
    pub async fn get_user_from_client(
        bot: Bot,
        chat_id: &i64,
        client: &Client,
        tg_api_phone: String,
        tg_api_password: String,
        mut rx_telegram_code: Receiver<String>,
    ) -> Result<User, Error> {
        // Result to return.
        let user: User;

        // Check if the program is not authorized in telegram yet.
        while !client.is_authorized().await? {
            // Authorize in telegram.

            log::info!("A user is not authorized, trying to log in...");

            // Request a verification code and prompt user to enter it.
            let token = client.request_login_code(tg_api_phone.as_str()).await?;

            // Inform a user that we need a verification token.

            bot
                .send_message(
                    teloxide::types::ChatId(chat_id.clone()),
                    "⚠️ <b>Enter the verification code.</b>\n\n\
                🔑 Use <code>/code</code> command to do this.\n\n\
                🤫 <del><i>Enter the code from right to left to avoid Telegram blocking the code.</i></del>\n\n\
                ❗ <b>Warning:</b> The system will not work until you do this!\n\n\
                ",
                )
                .parse_mode(teloxide::types::ParseMode::Html)
                .send()
                .await?;

            // Wait for a user to enter the verification code.
            let code: String = rx_telegram_code.recv().await.unwrap();

            // Reverse the code (Necessary to overcome Telegram blocking the shared code).
            let code: String = code.chars().rev().collect();

            // Get a telegram user instance.
            match client.sign_in(&token, &code).await {
                // The program signed in successfully.
                Ok(_) => (),
                // There is a Two-Factor verification to pass.
                // Use Two-Factor verification password.
                Err(SignInError::PasswordRequired(password_token)) => {
                    log::info!("2FA required...");

                    client
                        .check_password(password_token, tg_api_password.as_bytes())
                        .await?;
                }
                Err(SignInError::InvalidCode) => {
                    bot.send_message(
                        teloxide::types::ChatId(chat_id.clone()),
                        format!("The code is invalid, please, try again"),
                    )
                    .send()
                    .await?;
                    continue;
                }
                // Failed to sing in.
                Err(err) => {
                    bot.send_message(
                        teloxide::types::ChatId(chat_id.clone()),
                        format!("Failed to log in as a user :(\n{err}"),
                    )
                    .send()
                    .await?;
                    panic!("Failed to sign in as a user :(\n{err}");
                } // end Err
            }; // end match

            // Save the session to the file.
            // Note: It prevents the program from re-login via 2FA and
            // messages.
            client.session().save_to_file(SESSION_FILE)?;

            log::info!("Session is saved to file");
        } // end while

        // The program has been already authorized.
        // Get the client instance.
        user = client.get_me().await?;

        // Inform the user that the login was successful.
        bot.send_message(teloxide::types::ChatId(chat_id.clone()), "Logged in!")
            .send()
            .await?;

        log::info!("Logged in!");

        Ok(user)
    } // end fn get_user_from_client
} // end impl SignIn
