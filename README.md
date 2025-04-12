# Rust Example Telegram Bot

This is a Telegram bot written in Rust that:

*   Listens for messages mentioning its username (`@your_bot_username`).
*   Ignores messages without text, replying "i don't see text".
*   Forwards the text content of mentioning messages to an OpenAI-compatible Large Language Model (LLM) API.
*   Includes a configurable system message in the prompt sent to the LLM.
*   Replies to the original Telegram message with the LLM's response.
*   Handles LLM API errors gracefully by informing the user.
*   Uses `tracing` for logging.
*   Is designed to run in a Docker container.
*   Includes a GitHub Actions workflow to automatically build and push the Docker image to GitHub Container Registry (GHCR) at `ghcr.io/korjavin/rustexamplebot`.

## Prerequisites

*   Docker
*   A Telegram Bot Token (get one from BotFather on Telegram)
*   Access to an OpenAI-compatible API (like OpenAI's API or a self-hosted alternative) including:
    *   API Base URL
    *   API Token
    *   The name of the model you want to use

## Environment Variables

The bot requires the following environment variables to be set:

| Variable          | Description                                                                 | Example                                    |
| :---------------- | :-------------------------------------------------------------------------- | :----------------------------------------- |
| `TELOXIDE_TOKEN`  | Your Telegram Bot Token from BotFather.                                     | `123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11` |
| `OPENAPI_BASEURL` | Base URL for the OpenAI-compatible API (without `/chat/completions`).       | `https://api.openai.com/v1`                |
| `OPENAPI_TOKEN`   | Your API token for the OpenAI-compatible API.                               | `sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxx`          |
| `SYSTEM_MSG`      | System message to guide the LLM's behavior.                                 | `You are a helpful assistant.`             |
| `OPENAI_MODEL`    | The specific model name to use for the LLM API call.                        | `gpt-4`                                    |

Create a `.env` file in the project root (or set these in your deployment environment) based on the `.env.example` file.

```bash
# .env file content
TELOXIDE_TOKEN=YOUR_TELEGRAM_BOT_TOKEN_HERE
OPENAPI_BASEURL=YOUR_OPENAI_COMPATIBLE_API_BASE_URL
OPENAPI_TOKEN=YOUR_API_TOKEN_HERE
SYSTEM_MSG="You are a helpful assistant."
OPENAI_MODEL=YOUR_MODEL_NAME_HERE
```

## Running Locally with Docker

1.  **Build the Docker image:**
    ```bash
    docker build -t rust-example-bot .
    ```
2.  **Run the Docker container, passing the environment variables:**
    ```bash
    docker run --rm --env-file .env rust-example-bot
    ```
    *(Ensure your `.env` file is correctly populated)*

## Development

*   **Check:** `cargo check`
*   **Build:** `cargo build`
*   **Run (requires .env file):** `cargo run`

## CI/CD

A GitHub Actions workflow is configured in `.github/workflows/docker-build-push.yml`. On every push to the `main` branch, it will:

1.  Build the Docker image.
2.  Push the image to GitHub Container Registry (GHCR) tagged with `latest` and the Git commit SHA. The image will be available at `ghcr.io/korjavin/rustexamplebot`.

*(Remember to replace `korjavin` with your actual GitHub username if you fork this repository).*