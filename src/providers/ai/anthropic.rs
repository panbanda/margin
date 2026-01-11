//! Anthropic Claude API provider implementation.

use async_trait::async_trait;
use futures::Stream;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};

use super::traits::{
    CompletionRequest, CompletionResponse, CompletionStream, FinishReason, LlmError, LlmProvider,
    LlmResult, Message, Role, StreamChunk, TokenUsage,
};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Context lengths for Claude models.
fn model_context_length(model: &str) -> usize {
    match model {
        m if m.contains("claude-3-5") => 200_000,
        m if m.contains("claude-3") => 200_000,
        m if m.contains("claude-2.1") => 200_000,
        m if m.contains("claude-2") => 100_000,
        _ => 100_000,
    }
}

/// Anthropic API request format.
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

impl From<&Message> for AnthropicMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: match msg.role {
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
                // System messages are handled separately in Anthropic API
                Role::System => "user".to_string(),
            },
            content: msg.content.clone(),
        }
    }
}

/// Anthropic API response format.
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

/// Anthropic streaming event types.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum AnthropicStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: AnthropicMessageStart },

    #[serde(rename = "content_block_start")]
    ContentBlockStart { content_block: AnthropicContent },

    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { delta: AnthropicDelta },

    #[serde(rename = "content_block_stop")]
    ContentBlockStop,

    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: AnthropicMessageDelta,
        usage: Option<AnthropicDeltaUsage>,
    },

    #[serde(rename = "message_stop")]
    MessageStop,

    #[serde(rename = "ping")]
    Ping,

    #[serde(rename = "error")]
    Error { error: AnthropicErrorDetail },
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnthropicMessageStart {
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnthropicMessageDelta {
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnthropicDeltaUsage {
    output_tokens: usize,
}

/// Anthropic API error response.
#[derive(Debug, Deserialize)]
struct AnthropicError {
    error: AnthropicErrorDetail,
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorDetail {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

/// Provider for Anthropic's Claude API.
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    context_length: usize,
}

impl AnthropicProvider {
    /// Creates a new Anthropic provider.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let model = model.into();
        let context_length = model_context_length(&model);

        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model,
            context_length,
        }
    }

    /// Creates a provider with Claude 3.5 Sonnet (recommended for most tasks).
    pub fn claude_sonnet(api_key: impl Into<String>) -> Self {
        Self::new(api_key, "claude-3-5-sonnet-20241022")
    }

    /// Creates a provider with Claude 3.5 Haiku (fast and cost-effective).
    pub fn claude_haiku(api_key: impl Into<String>) -> Self {
        Self::new(api_key, "claude-3-5-haiku-20241022")
    }

    /// Creates a provider with Claude 3 Opus (most capable).
    pub fn claude_opus(api_key: impl Into<String>) -> Self {
        Self::new(api_key, "claude-3-opus-20240229")
    }

    /// Overrides the HTTP client.
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key).unwrap_or_else(|_| HeaderValue::from_static("")),
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(ANTHROPIC_VERSION),
        );
        headers
    }

    fn build_request(&self, request: &CompletionRequest, stream: bool) -> AnthropicRequest {
        // Filter out system messages and convert the rest
        let messages: Vec<AnthropicMessage> = request
            .messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(AnthropicMessage::from)
            .collect();

        // Combine system prompt with any system messages from the conversation
        let system_prompt = {
            let system_messages: Vec<&str> = request
                .messages
                .iter()
                .filter(|m| m.role == Role::System)
                .map(|m| m.content.as_str())
                .collect();

            match (&request.system_prompt, system_messages.is_empty()) {
                (Some(prompt), true) => Some(prompt.clone()),
                (Some(prompt), false) => {
                    Some(format!("{}\n\n{}", prompt, system_messages.join("\n\n")))
                }
                (None, false) => Some(system_messages.join("\n\n")),
                (None, true) => None,
            }
        };

        AnthropicRequest {
            model: self.model.clone(),
            messages,
            max_tokens: request.max_tokens.unwrap_or(4096),
            system: system_prompt,
            temperature: Some(request.temperature),
            stop_sequences: request.stop.clone(),
            stream,
        }
    }

    fn parse_finish_reason(reason: Option<&str>) -> FinishReason {
        match reason {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("stop_sequence") => FinishReason::Stop,
            Some("tool_use") => FinishReason::ToolCalls,
            _ => FinishReason::Other,
        }
    }

    async fn handle_error_response(&self, response: reqwest::Response) -> LlmError {
        let status = response.status().as_u16();

        if status == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());

            return LlmError::RateLimited {
                retry_after_secs: retry_after,
            };
        }

        if let Ok(error) = response.json::<AnthropicError>().await {
            if status == 401 || error.error.error_type == "authentication_error" {
                return LlmError::AuthenticationError(error.error.message);
            }
            return LlmError::ApiError {
                status,
                message: error.error.message,
            };
        }

        LlmError::ApiError {
            status,
            message: format!("HTTP {}", status),
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn complete(&self, request: &CompletionRequest) -> LlmResult<CompletionResponse> {
        let body = self.build_request(request, false);

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        let text = api_response
            .content
            .into_iter()
            .filter(|c| c.content_type == "text")
            .filter_map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        let tokens_used = TokenUsage {
            prompt_tokens: api_response.usage.input_tokens,
            completion_tokens: api_response.usage.output_tokens,
            total_tokens: api_response.usage.input_tokens + api_response.usage.output_tokens,
        };

        Ok(CompletionResponse {
            text,
            tokens_used,
            finish_reason: Self::parse_finish_reason(api_response.stop_reason.as_deref()),
        })
    }

    async fn stream_complete(&self, request: &CompletionRequest) -> LlmResult<CompletionStream> {
        let body = self.build_request(request, true);

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let stream = response.bytes_stream();
        Ok(Box::pin(AnthropicStream::new(stream)))
    }

    fn supports_function_calling(&self) -> bool {
        // Claude 3+ models support tool use
        self.model.contains("claude-3")
    }

    fn max_context_length(&self) -> usize {
        self.context_length
    }
}

/// Stream wrapper for Anthropic SSE responses.
struct AnthropicStream<S> {
    inner: S,
    buffer: String,
    finished: bool,
}

impl<S> AnthropicStream<S> {
    fn new(inner: S) -> Self {
        Self {
            inner,
            buffer: String::new(),
            finished: false,
        }
    }

    fn parse_event(&self, data: &str) -> Option<LlmResult<StreamChunk>> {
        match serde_json::from_str::<AnthropicStreamEvent>(data) {
            Ok(event) => match event {
                AnthropicStreamEvent::ContentBlockDelta { delta } => {
                    if delta.delta_type == "text_delta" {
                        Some(Ok(StreamChunk {
                            text: delta.text.unwrap_or_default(),
                            finish_reason: None,
                        }))
                    } else {
                        None
                    }
                }
                AnthropicStreamEvent::MessageDelta { delta, .. } => {
                    let finish_reason = delta
                        .stop_reason
                        .as_deref()
                        .map(|r| AnthropicProvider::parse_finish_reason(Some(r)));

                    Some(Ok(StreamChunk {
                        text: String::new(),
                        finish_reason,
                    }))
                }
                AnthropicStreamEvent::Error { error } => Some(Err(LlmError::ApiError {
                    status: 0,
                    message: error.message,
                })),
                _ => None,
            },
            Err(e) => Some(Err(LlmError::InvalidResponse(format!(
                "Failed to parse stream event: {}",
                e
            )))),
        }
    }
}

impl<S, E> Stream for AnthropicStream<S>
where
    S: Stream<Item = Result<bytes::Bytes, E>> + Unpin,
    E: std::error::Error,
{
    type Item = LlmResult<StreamChunk>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.finished {
            return Poll::Ready(None);
        }

        loop {
            // Look for complete event in buffer
            if let Some(event_end) = self.buffer.find("\n\n") {
                let event_text = self.buffer[..event_end].to_string();
                self.buffer = self.buffer[event_end + 2..].to_string();

                // Parse SSE format: "event: type\ndata: json"
                let mut event_type = None;
                let mut data = None;

                for line in event_text.lines() {
                    if let Some(value) = line.strip_prefix("event: ") {
                        event_type = Some(value.to_string());
                    } else if let Some(value) = line.strip_prefix("data: ") {
                        data = Some(value.to_string());
                    }
                }

                if event_type.as_deref() == Some("message_stop") {
                    self.finished = true;
                    return Poll::Ready(None);
                }

                if let Some(data) = data {
                    if let Some(result) = self.parse_event(&data) {
                        return Poll::Ready(Some(result));
                    }
                }
                continue;
            }

            // Need more data
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    if let Ok(text) = std::str::from_utf8(&bytes) {
                        self.buffer.push_str(text);
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(LlmError::StreamError(e.to_string()))));
                }
                Poll::Ready(None) => {
                    self.finished = true;
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_context_length() {
        assert_eq!(model_context_length("claude-3-5-sonnet-20241022"), 200_000);
        assert_eq!(model_context_length("claude-3-opus-20240229"), 200_000);
        assert_eq!(model_context_length("claude-2.1"), 200_000);
        assert_eq!(model_context_length("claude-2"), 100_000);
        assert_eq!(model_context_length("unknown"), 100_000);
    }

    #[test]
    fn test_anthropic_request_serialization() {
        let request = CompletionRequest::new(vec![Message::user("Hello")])
            .with_system_prompt("Be helpful")
            .with_temperature(0.7)
            .with_max_tokens(100);

        let provider = AnthropicProvider::new("test-key", "claude-3-5-sonnet-20241022");
        let anthropic_request = provider.build_request(&request, false);

        let json = serde_json::to_string(&anthropic_request).unwrap();
        assert!(json.contains("claude-3-5-sonnet"));
        assert!(json.contains("Be helpful"));
        assert!(json.contains("Hello"));
        assert!(!json.contains("stream"));
    }

    #[test]
    fn test_anthropic_response_parsing() {
        let json = r#"{
            "content": [{"type": "text", "text": "Hello there!"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: AnthropicResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.content.len(), 1);
        assert_eq!(response.content[0].text, Some("Hello there!".to_string()));
        assert_eq!(response.stop_reason, Some("end_turn".to_string()));
        assert_eq!(response.usage.input_tokens, 10);
        assert_eq!(response.usage.output_tokens, 5);
    }

    #[test]
    fn test_parse_finish_reason() {
        assert_eq!(
            AnthropicProvider::parse_finish_reason(Some("end_turn")),
            FinishReason::Stop
        );
        assert_eq!(
            AnthropicProvider::parse_finish_reason(Some("max_tokens")),
            FinishReason::Length
        );
        assert_eq!(
            AnthropicProvider::parse_finish_reason(Some("stop_sequence")),
            FinishReason::Stop
        );
        assert_eq!(
            AnthropicProvider::parse_finish_reason(Some("tool_use")),
            FinishReason::ToolCalls
        );
        assert_eq!(
            AnthropicProvider::parse_finish_reason(None),
            FinishReason::Other
        );
    }

    #[test]
    fn test_convenience_constructors() {
        let sonnet = AnthropicProvider::claude_sonnet("key");
        assert_eq!(sonnet.model, "claude-3-5-sonnet-20241022");

        let haiku = AnthropicProvider::claude_haiku("key");
        assert_eq!(haiku.model, "claude-3-5-haiku-20241022");

        let opus = AnthropicProvider::claude_opus("key");
        assert_eq!(opus.model, "claude-3-opus-20240229");
    }

    #[test]
    fn test_provider_trait_methods() {
        let provider = AnthropicProvider::claude_sonnet("test");
        assert_eq!(provider.name(), "anthropic");
        assert_eq!(provider.model(), "claude-3-5-sonnet-20241022");
        assert!(provider.supports_function_calling());
        assert_eq!(provider.max_context_length(), 200_000);
    }

    #[test]
    fn test_system_message_handling() {
        // System prompt should be extracted to the 'system' field
        let request = CompletionRequest::new(vec![
            Message::system("System context"),
            Message::user("User message"),
        ])
        .with_system_prompt("Top level system");

        let provider = AnthropicProvider::new("key", "claude-3-5-sonnet-20241022");
        let anthropic_request = provider.build_request(&request, false);

        // System messages should be combined
        let system = anthropic_request.system.unwrap();
        assert!(system.contains("Top level system"));
        assert!(system.contains("System context"));

        // Only non-system messages should be in the messages array
        assert_eq!(anthropic_request.messages.len(), 1);
        assert_eq!(anthropic_request.messages[0].role, "user");
    }

    #[test]
    fn test_stream_event_parsing() {
        let delta_json =
            r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":"Hello"}}"#;
        let event: AnthropicStreamEvent = serde_json::from_str(delta_json).unwrap();

        match event {
            AnthropicStreamEvent::ContentBlockDelta { delta } => {
                assert_eq!(delta.delta_type, "text_delta");
                assert_eq!(delta.text, Some("Hello".to_string()));
            }
            _ => panic!("Expected ContentBlockDelta"),
        }
    }
}
