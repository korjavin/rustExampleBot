mod config;

use config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::Me};
use tracing::{error, info, instrument}; // Added instrument

// --- Structs for OpenAI API Interaction ---

#[derive(Serialize, Debug)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    // Add other fields if needed, like usage
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: ChatMessage,
    // Add other fields like finish_reason if needed
}

// --- Bot Logic ---

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    info!("Starting bot...");

    // Load configuration from environment variables
    let cfg = match Config::load_from_env() {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to load configuration: {}", err);
            // Consider exiting more gracefully or providing defaults if applicable
            panic!("Configuration error: {}", err);
        }
    };

    // Create the bot instance
    let bot = Bot::new(cfg.teloxide_token.clone());

    // Fetch bot's own information (like username)
    let me = match bot.get_me().await {
        Ok(me_info) => {
            info!("Successfully fetched bot info: id={}, username={:?}, first_name={}",
                  me_info.id, me_info.username(), me_info.first_name);
            if me_info.username.is_none() {
                error!("Bot username is not set! Mentioning will not work.");
                // Decide if you want to panic or continue without mention support
            }
            me_info
        }
        Err(e) => {
            error!("Failed to get bot info: {}", e);
            panic!("Could not get bot info: {}", e);
        }
    };

    // Create reqwest client
    let client = Client::new();

    // Set up the dispatcher
    let handler = Update::filter_message()
        .branch(dptree::endpoint(message_handler));

    info!("Dispatcher setup complete. Starting polling...");

    Dispatcher::builder(bot, handler)
        // Pass the configuration, reqwest client, and bot's Me info to the handler
        .dependencies(dptree::deps![cfg, client, me])
        .enable_ctrlc_handler() // Allows stopping the bot with Ctrl+C
        .build()
        .dispatch()
        .await;

    info!("Bot stopped.");
}

#[instrument(skip_all, fields(chat_id = %msg.chat.id, user_id = %msg.from().map(|u| u.id.to_string()).unwrap_or_else(|| "unknown".into())))]
async fn message_handler(
    bot: Bot,
    msg: Message,
    cfg: Config,
    client: Client,
    me: Me, // Bot's own info
) -> ResponseResult<()> {
    // --- 1. Check for Text ---
    let text = match msg.text() {
        Some(t) => t,
        None => {
            info!("Received message without text.");
            bot.send_message(msg.chat.id, "i don't see text")
                .reply_to_message_id(msg.id)
                .await?;
            return Ok(());
        }
    };

    // --- 2. Check for Mention ---
    let username = me.username();
    if username.is_empty() {
        // Bot has no username, so cannot be mentioned.
        // This was logged at startup. Silently ignore the message.
        return Ok(()); // Ignore the message
    } else {
        // Bot has a username, check if it's mentioned in the text
        let bot_username_mention = format!("@{}", username);
        if !text.contains(&bot_username_mention) {
            // Username exists, but it's not mentioned in this message
            info!("Message does not mention the bot ({}). Ignoring.", bot_username_mention);
            return Ok(()); // Ignore the message
        }
        // Mention found, log and proceed
        info!("Bot mention '{}' found in message.", bot_username_mention);
    }
    // --- If execution reaches here, the bot has a username AND it was mentioned ---

    info!("Received relevant message: '{}'", text);

    // --- 3. Prepare LLM Request ---
    let api_request = ChatCompletionRequest {
        model: cfg.openai_model.clone(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: cfg.system_msg.clone(),
            },
            ChatMessage {
                role: "user".to_string(),
                // Consider cleaning the text (e.g., removing the bot's username)
                content: text.to_string(),
            },
        ],
    };

    let api_url = format!("{}/chat/completions", cfg.openapi_baseurl.trim_end_matches('/'));

    info!("Sending request to LLM API at {}: {:?}", api_url, api_request);

    // --- 4. Send Request & Handle Response ---
    match client
        .post(&api_url)
        .bearer_auth(&cfg.openapi_token)
        .json(&api_request)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ChatCompletionResponse>().await {
                    Ok(completion) => {
                        if let Some(choice) = completion.choices.first() {
                            let reply_text = &choice.message.content;
                            info!("Received LLM reply: '{}'", reply_text);
                            bot.send_message(msg.chat.id, reply_text)
                                .reply_to_message_id(msg.id)
                                .await?;
                            info!("Successfully replied to user.");
                        } else {
                            error!("LLM API response had no choices.");
                            bot.send_message(msg.chat.id, "Sorry, I couldn't get a response from the AI.")
                                .reply_to_message_id(msg.id)
                                .await?;
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse LLM API response: {}", e);
                        bot.send_message(msg.chat.id, "Sorry, I couldn't understand the AI's response.")
                            .reply_to_message_id(msg.id)
                            .await?;
                    }
                }
            } else {
                let status = response.status();
                let error_body = response.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
                error!(
                    "LLM API request failed with status {}: {}",
                    status, error_body
                );
                bot.send_message(msg.chat.id, "Sorry, I couldn't process your request at the moment.")
                    .reply_to_message_id(msg.id)
                    .await?;
            }
        }
        Err(e) => {
            error!("Failed to send request to LLM API: {}", e);
            bot.send_message(msg.chat.id, "Sorry, I couldn't process your request at the moment.")
                .reply_to_message_id(msg.id)
                .await?;
        }
    }

    Ok(())
}
