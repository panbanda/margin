//! OpenAI-compatible provider implementation.
//!
//! Works with OpenAI, Ollama, vLLM, LM Studio, and other compatible endpoints.

use async_trait::async_trait;
use futures::Stream;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};

use super::traits::{
    CompletionRequest, CompletionResponse, CompletionStream, FinishReason, LlmError, LlmProvider,
    LlmResult, Message, Role, StreamChunk, TokenUsage,
};

/// Default base URL for OpenAI API.
const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

/// Context lengths for common OpenAI models.
fn model_context_length(model: &str) -> usize {
    match model {
        m if m.starts_with("gpt-4o") => 128_000,
        m if m.starts_with("gpt-4-turbo") => 128_000,
        m if m.starts_with("gpt-4-32k") => 32_768,
        m if m.starts_with("gpt-4") => 8_192,
        m if m.starts_with("gpt-3.5-turbo-16k") => 16_384,
        m if m.starts_with("gpt-3.5") => 4_096,
        m if m.starts_with("o1") => 128_000,
        // Default for unknown models (many local models use 4k-8k)
        _ => 4_096,
    }
}

/// Models that support function calling.
fn supports_functions(model: &str) -> bool {
    model.starts_with("gpt-4") || model.starts_with("gpt-3.5-turbo") || model.starts_with("gpt-4o")
}

/// OpenAI API request format.
#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

impl From<&Message> for OpenAiMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: match msg.role {
                Role::System => "system".to_string(),
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
            },
            content: msg.content.clone(),
        }
    }
}

/// OpenAI API response format.
#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

/// OpenAI streaming response format.
#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiDelta {
    content: Option<String>,
}

/// OpenAI API error response.
#[derive(Debug, Deserialize)]
struct OpenAiError {
    error: OpenAiErrorDetail,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAiErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<String>,
}

/// Provider for OpenAI-compatible APIs.
///
/// Works with:
/// - OpenAI API (api.openai.com)
/// - Ollama (localhost:11434)
/// - vLLM
/// - LM Studio
/// - Any other OpenAI-compatible endpoint
pub struct OpenAiCompatibleProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
    context_length: usize,
}

impl OpenAiCompatibleProvider {
    /// Creates a new provider for OpenAI's API.
    pub fn openai(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let model = model.into();
        let context_length = model_context_length(&model);

        Self {
            client: reqwest::Client::new(),
            base_url: OPENAI_BASE_URL.to_string(),
            api_key: Some(api_key.into()),
            model,
            context_length,
        }
    }

    /// Creates a new provider for a custom endpoint.
    pub fn custom(
        base_url: impl Into<String>,
        api_key: Option<String>,
        model: impl Into<String>,
    ) -> Self {
        let model = model.into();
        let context_length = model_context_length(&model);

        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key,
            model,
            context_length,
        }
    }

    /// Overrides the context length (useful for local models with custom context).
    pub fn with_context_length(mut self, length: usize) -> Self {
        self.context_length = length;
        self
    }

    /// Overrides the HTTP client (useful for custom timeouts or proxies).
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(ref api_key) = self.api_key {
            if let Ok(value) = HeaderValue::from_str(&format!("Bearer {}", api_key)) {
                headers.insert(AUTHORIZATION, value);
            }
        }

        headers
    }

    fn build_request(&self, request: &CompletionRequest, stream: bool) -> OpenAiRequest {
        let mut messages: Vec<OpenAiMessage> = Vec::new();

        // Add system prompt as first message if present
        if let Some(ref system) = request.system_prompt {
            messages.push(OpenAiMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        }

        // Add conversation messages
        messages.extend(request.messages.iter().map(OpenAiMessage::from));

        OpenAiRequest {
            model: self.model.clone(),
            messages,
            temperature: Some(request.temperature),
            max_tokens: request.max_tokens,
            stop: request.stop.clone(),
            stream,
        }
    }

    fn parse_finish_reason(reason: Option<&str>) -> FinishReason {
        match reason {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("content_filter") => FinishReason::ContentFilter,
            Some("tool_calls") | Some("function_call") => FinishReason::ToolCalls,
            _ => FinishReason::Other,
        }
    }

    async fn handle_error_response(&self, response: reqwest::Response) -> LlmError {
        let status = response.status().as_u16();

        // Check for rate limiting
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

        // Try to parse error body
        if let Ok(error) = response.json::<OpenAiError>().await {
            if status == 401 || error.error.code.as_deref() == Some("invalid_api_key") {
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
impl LlmProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &str {
        "openai-compatible"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn complete(&self, request: &CompletionRequest) -> LlmResult<CompletionResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_request(request, false);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let api_response: OpenAiResponse = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        let choice = api_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| LlmError::InvalidResponse("No choices in response".to_string()))?;

        let text = choice.message.content.unwrap_or_default();

        let tokens_used = api_response
            .usage
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            })
            .unwrap_or_default();

        Ok(CompletionResponse {
            text,
            tokens_used,
            finish_reason: Self::parse_finish_reason(choice.finish_reason.as_deref()),
        })
    }

    async fn stream_complete(&self, request: &CompletionRequest) -> LlmResult<CompletionStream> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_request(request, true);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let stream = response.bytes_stream();
        Ok(Box::pin(OpenAiStream::new(stream)))
    }

    fn supports_function_calling(&self) -> bool {
        supports_functions(&self.model)
    }

    fn max_context_length(&self) -> usize {
        self.context_length
    }
}

/// Stream wrapper for OpenAI SSE responses.
struct OpenAiStream<S> {
    inner: S,
    buffer: String,
}

impl<S> OpenAiStream<S> {
    fn new(inner: S) -> Self {
        Self {
            inner,
            buffer: String::new(),
        }
    }

    fn parse_line(&self, line: &str) -> Option<LlmResult<StreamChunk>> {
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with(':') {
            return None;
        }

        // Parse SSE data line
        let data = line.strip_prefix("data: ")?;

        // Check for stream end
        if data == "[DONE]" {
            return None;
        }

        // Parse JSON chunk
        match serde_json::from_str::<OpenAiStreamChunk>(data) {
            Ok(chunk) => {
                let choice = chunk.choices.into_iter().next()?;
                let text = choice.delta.content.unwrap_or_default();
                let finish_reason = choice
                    .finish_reason
                    .as_deref()
                    .map(|r| OpenAiCompatibleProvider::parse_finish_reason(Some(r)));

                Some(Ok(StreamChunk {
                    text,
                    finish_reason,
                }))
            }
            Err(e) => Some(Err(LlmError::InvalidResponse(format!(
                "Failed to parse stream chunk: {}",
                e
            )))),
        }
    }
}

impl<S, E> Stream for OpenAiStream<S>
where
    S: Stream<Item = Result<bytes::Bytes, E>> + Unpin,
    E: std::error::Error,
{
    type Item = LlmResult<StreamChunk>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Check buffer for complete lines
            if let Some(newline_pos) = self.buffer.find('\n') {
                let line = self.buffer[..newline_pos].trim().to_string();
                self.buffer = self.buffer[newline_pos + 1..].to_string();

                if let Some(result) = self.parse_line(&line) {
                    return Poll::Ready(Some(result));
                }
                continue;
            }

            // Need more data from the stream
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
                    // Process any remaining buffer
                    if !self.buffer.is_empty() {
                        let line = std::mem::take(&mut self.buffer);
                        if let Some(result) = self.parse_line(line.trim()) {
                            return Poll::Ready(Some(result));
                        }
                    }
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
        assert_eq!(model_context_length("gpt-4o-2024-05-13"), 128_000);
        assert_eq!(model_context_length("gpt-4-turbo-preview"), 128_000);
        assert_eq!(model_context_length("gpt-4"), 8_192);
        assert_eq!(model_context_length("gpt-3.5-turbo"), 4_096);
        assert_eq!(model_context_length("llama-3-8b"), 4_096);
    }

    #[test]
    fn test_supports_functions() {
        assert!(supports_functions("gpt-4o"));
        assert!(supports_functions("gpt-4-turbo"));
        assert!(supports_functions("gpt-3.5-turbo"));
        assert!(!supports_functions("llama-3"));
        assert!(!supports_functions("mistral"));
    }

    #[test]
    fn test_openai_request_serialization() {
        let request = CompletionRequest::new(vec![Message::user("Hello")])
            .with_system_prompt("Be helpful")
            .with_temperature(0.7)
            .with_max_tokens(100);

        let provider = OpenAiCompatibleProvider::openai("test-key", "gpt-4");
        let openai_request = provider.build_request(&request, false);

        let json = serde_json::to_string(&openai_request).unwrap();
        assert!(json.contains("gpt-4"));
        assert!(json.contains("Be helpful"));
        assert!(json.contains("Hello"));
        assert!(!json.contains("stream"));
    }

    #[test]
    fn test_openai_request_with_stream() {
        let request = CompletionRequest::new(vec![Message::user("Hi")]);
        let provider = OpenAiCompatibleProvider::openai("key", "gpt-4");
        let openai_request = provider.build_request(&request, true);

        let json = serde_json::to_string(&openai_request).unwrap();
        assert!(json.contains("\"stream\":true"));
    }

    #[test]
    fn test_openai_response_parsing() {
        let json = r#"{
            "choices": [{
                "message": {"content": "Hello there!"},
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        }"#;

        let response: OpenAiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(
            response.choices[0].message.content,
            Some("Hello there!".to_string())
        );
        assert_eq!(response.usage.as_ref().unwrap().total_tokens, 15);
    }

    #[test]
    fn test_parse_finish_reason() {
        assert_eq!(
            OpenAiCompatibleProvider::parse_finish_reason(Some("stop")),
            FinishReason::Stop
        );
        assert_eq!(
            OpenAiCompatibleProvider::parse_finish_reason(Some("length")),
            FinishReason::Length
        );
        assert_eq!(
            OpenAiCompatibleProvider::parse_finish_reason(Some("content_filter")),
            FinishReason::ContentFilter
        );
        assert_eq!(
            OpenAiCompatibleProvider::parse_finish_reason(Some("tool_calls")),
            FinishReason::ToolCalls
        );
        assert_eq!(
            OpenAiCompatibleProvider::parse_finish_reason(Some("unknown")),
            FinishReason::Other
        );
        assert_eq!(
            OpenAiCompatibleProvider::parse_finish_reason(None),
            FinishReason::Other
        );
    }

    #[test]
    fn test_custom_provider() {
        let provider =
            OpenAiCompatibleProvider::custom("http://localhost:11434/v1", None, "llama3")
                .with_context_length(8192);

        assert_eq!(provider.base_url, "http://localhost:11434/v1");
        assert!(provider.api_key.is_none());
        assert_eq!(provider.model, "llama3");
        assert_eq!(provider.max_context_length(), 8192);
    }

    #[test]
    fn test_provider_trait_methods() {
        let provider = OpenAiCompatibleProvider::openai("test", "gpt-4o");
        assert_eq!(provider.name(), "openai-compatible");
        assert_eq!(provider.model(), "gpt-4o");
        assert!(provider.supports_function_calling());
        assert_eq!(provider.max_context_length(), 128_000);
    }

    #[test]
    fn test_stream_chunk_parsing() {
        let json = r#"{"choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let chunk: OpenAiStreamChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
        assert!(chunk.choices[0].finish_reason.is_none());
    }

    #[test]
    fn test_trailing_slash_removal() {
        let provider =
            OpenAiCompatibleProvider::custom("http://localhost:11434/v1/", None, "llama3");
        assert_eq!(provider.base_url, "http://localhost:11434/v1");
    }
}
