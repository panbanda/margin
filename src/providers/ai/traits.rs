//! LLM provider trait and supporting types.

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;

/// Errors that can occur during LLM operations.
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Rate limited, retry after {retry_after_secs:?} seconds")]
    RateLimited { retry_after_secs: Option<u64> },

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Context length exceeded: {used} tokens used, {max} maximum")]
    ContextLengthExceeded { used: usize, max: usize },

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Provider not available: {0}")]
    Unavailable(String),
}

/// Result type for LLM operations.
pub type LlmResult<T> = Result<T, LlmError>;

/// Role of a message in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Request for a completion from an LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Optional system prompt to set context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    /// Conversation messages.
    pub messages: Vec<Message>,

    /// Sampling temperature (0.0 to 2.0, lower is more deterministic).
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Maximum tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Stop sequences that will halt generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

fn default_temperature() -> f32 {
    0.7
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            system_prompt: None,
            messages: Vec::new(),
            temperature: default_temperature(),
            max_tokens: None,
            stop: None,
        }
    }
}

impl CompletionRequest {
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            messages,
            ..Default::default()
        }
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}

/// Token usage statistics from a completion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of tokens in the prompt.
    pub prompt_tokens: usize,

    /// Number of tokens in the completion.
    pub completion_tokens: usize,

    /// Total tokens used.
    pub total_tokens: usize,
}

/// Reason why a completion finished.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural end of generation.
    Stop,

    /// Hit the max_tokens limit.
    Length,

    /// Content was filtered for safety.
    ContentFilter,

    /// Function/tool call was requested.
    ToolCalls,

    /// Unknown or provider-specific reason.
    #[serde(other)]
    Other,
}

/// Response from a completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Generated text content.
    pub text: String,

    /// Token usage statistics.
    pub tokens_used: TokenUsage,

    /// Why generation finished.
    pub finish_reason: FinishReason,
}

/// A streaming chunk of completion output.
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Text content of this chunk.
    pub text: String,

    /// If this is the final chunk, contains the finish reason.
    pub finish_reason: Option<FinishReason>,
}

/// Type alias for the streaming response.
pub type CompletionStream = Pin<Box<dyn Stream<Item = LlmResult<StreamChunk>> + Send>>;

/// Trait for LLM providers (OpenAI, Anthropic, Ollama, etc.).
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Returns the provider's name (e.g., "openai", "anthropic", "ollama").
    fn name(&self) -> &str;

    /// Performs a completion request and returns the full response.
    async fn complete(&self, request: &CompletionRequest) -> LlmResult<CompletionResponse>;

    /// Performs a streaming completion request.
    async fn stream_complete(&self, request: &CompletionRequest) -> LlmResult<CompletionStream>;

    /// Whether this provider supports function/tool calling.
    fn supports_function_calling(&self) -> bool;

    /// Maximum context length in tokens for the configured model.
    fn max_context_length(&self) -> usize;

    /// Returns the model identifier being used.
    fn model(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_constructors() {
        let system = Message::system("You are helpful.");
        assert_eq!(system.role, Role::System);
        assert_eq!(system.content, "You are helpful.");

        let user = Message::user("Hello");
        assert_eq!(user.role, Role::User);

        let assistant = Message::assistant("Hi there!");
        assert_eq!(assistant.role, Role::Assistant);
    }

    #[test]
    fn test_completion_request_builder() {
        let request = CompletionRequest::new(vec![Message::user("Test")])
            .with_system_prompt("Be helpful")
            .with_temperature(0.5)
            .with_max_tokens(100);

        assert_eq!(request.system_prompt, Some("Be helpful".to_string()));
        assert_eq!(request.temperature, 0.5);
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.messages.len(), 1);
    }

    #[test]
    fn test_completion_request_serialization() {
        let request = CompletionRequest::new(vec![
            Message::system("System prompt"),
            Message::user("Hello"),
        ]);

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CompletionRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.messages.len(), 2);
        assert_eq!(deserialized.messages[0].role, Role::System);
    }

    #[test]
    fn test_completion_response_serialization() {
        let response = CompletionResponse {
            text: "Hello there!".to_string(),
            tokens_used: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            finish_reason: FinishReason::Stop,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Hello there!"));

        let deserialized: CompletionResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.finish_reason, FinishReason::Stop);
        assert_eq!(deserialized.tokens_used.total_tokens, 15);
    }

    #[test]
    fn test_finish_reason_deserialization() {
        let stop: FinishReason = serde_json::from_str("\"stop\"").unwrap();
        assert_eq!(stop, FinishReason::Stop);

        let length: FinishReason = serde_json::from_str("\"length\"").unwrap();
        assert_eq!(length, FinishReason::Length);

        // Unknown reasons should map to Other
        let unknown: FinishReason = serde_json::from_str("\"something_else\"").unwrap();
        assert_eq!(unknown, FinishReason::Other);
    }

    #[test]
    fn test_token_usage_default() {
        let usage = TokenUsage::default();
        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.completion_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
    }
}
