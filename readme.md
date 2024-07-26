# AI Hero project

Персонажи для виртуального мира на базе OpenAI с неограниченными возможностями для людей и бизнеса.

## Get Started

### Setup env

### Rename files

```
mv example.env .env
mv example.config.toml config.toml
```



### Musthave params for config.toml:
```
# API key of a telegram bot that serves as a user interface with the admin client.
telegram_bot_token = "some_telegram_bot_token"


# telegram client credentials.
tg_api_id = "tg_api_id"
tg_api_hash = "some_tg_api_hash"
tg_api_phone = "some_tg_api_phone"
tg_api_password = "some_tg_api_password"

# the credentials of a trusted telegram user to interact with.
tg_trusted_user_name = "@user_name"

# the channel name to copy the information from.
tg_copy_channel_name = "None"

admin_usernames = ["user_name"]

openai_api_key = "some_openai_api_key"

```

## Docker start

```
docker compose up
```

## Locall start

```
cargo install diesel_cli

mkdir -p ./data

diesel setup

mkdir data
```

```
cargo run
```