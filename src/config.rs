use std::env;
use tracing::info;

#[derive(Clone, Debug)]
pub struct Config {
    pub teloxide_token: String,
    pub openapi_baseurl: String,
    pub openapi_token: String,
    pub system_msg: String,
    pub openai_model: String,
}

impl Config {
    pub fn load_from_env() -> Result<Self, String> {
        // Attempt to load .env file. This is useful for local development.
        // It's okay if it fails in production where env vars are set directly.
        match dotenvy::dotenv() {
            Ok(path) => info!("Loaded .env file from path: {}", path.display()),
            Err(_) => info!("No .env file found or failed to load. Relying on environment variables."),
        };

        Ok(Self {
            teloxide_token: get_env_var("TELOXIDE_TOKEN")?,
            openapi_baseurl: get_env_var("OPENAPI_BASEURL")?,
            openapi_token: get_env_var("OPENAPI_TOKEN")?,
            system_msg: get_env_var("SYSTEM_MSG")?,
            openai_model: get_env_var("OPENAI_MODEL")?,
        })
    }
}

fn get_env_var(var_name: &str) -> Result<String, String> {
    env::var(var_name).map_err(|e| format!("Environment variable '{}' not found: {}", var_name, e))
}