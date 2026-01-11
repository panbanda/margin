//! AI/LLM provider implementations.
//!
//! This module provides a unified interface for interacting with various LLM providers.
//!
//! # Supported Providers
//!
//! - **OpenAI-compatible**: Works with OpenAI, vLLM, LM Studio, and other compatible endpoints
//! - **Anthropic**: Claude models via Anthropic's API
//! - **Ollama**: Local LLM inference via Ollama
//!
//! # Example
//!
//! ```rust,no_run
//! use margin::providers::ai::{
//!     LlmProvider, CompletionRequest, Message,
//!     OpenAiCompatibleProvider, AnthropicProvider, OllamaProvider,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Using OpenAI
//! let openai = OpenAiCompatibleProvider::openai("sk-...", "gpt-4o");
//!
//! // Using Anthropic Claude
//! let anthropic = AnthropicProvider::claude_sonnet("sk-ant-...");
//!
//! // Using local Ollama
//! let ollama = OllamaProvider::llama3();
//!
//! // All providers implement the same trait
//! let request = CompletionRequest::new(vec![Message::user("Hello!")])
//!     .with_system_prompt("You are a helpful assistant.");
//!
//! let response = openai.complete(&request).await?;
//! println!("Response: {}", response.text);
//! # Ok(())
//! # }
//! ```

mod anthropic;
mod ollama;
mod openai;
mod traits;

pub use anthropic::AnthropicProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiCompatibleProvider;
pub use traits::{
    CompletionRequest, CompletionResponse, CompletionStream, FinishReason, LlmError, LlmProvider,
    LlmResult, Message, Role, StreamChunk, TokenUsage,
};
