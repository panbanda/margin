//! Ollama provider implementation.
//!
//! Ollama exposes an OpenAI-compatible API, so this is a thin wrapper
//! around OpenAiCompatibleProvider with Ollama-specific defaults.

use super::openai::OpenAiCompatibleProvider;
use super::traits::{
    CompletionRequest, CompletionResponse, CompletionStream, LlmProvider, LlmResult,
};
use async_trait::async_trait;

/// Default Ollama API URL.
const OLLAMA_DEFAULT_URL: &str = "http://localhost:11434/v1";

/// Context lengths for common Ollama models.
fn model_context_length(model: &str) -> usize {
    match model {
        m if m.starts_with("llama3.2") => 128_000,
        m if m.starts_with("llama3.1") => 128_000,
        m if m.starts_with("llama3") => 8_192,
        m if m.starts_with("llama2") => 4_096,
        m if m.starts_with("mistral") => 32_768,
        m if m.starts_with("mixtral") => 32_768,
        m if m.starts_with("codellama") => 16_384,
        m if m.starts_with("phi") => 2_048,
        m if m.starts_with("gemma2") => 8_192,
        m if m.starts_with("gemma") => 8_192,
        m if m.starts_with("qwen2.5") => 32_768,
        m if m.starts_with("qwen2") => 32_768,
        m if m.starts_with("qwen") => 8_192,
        m if m.starts_with("deepseek") => 16_384,
        _ => 4_096,
    }
}

/// Provider for Ollama's local LLM server.
///
/// Ollama serves models locally and provides an OpenAI-compatible API.
/// This provider wraps OpenAiCompatibleProvider with Ollama-specific defaults.
pub struct OllamaProvider {
    inner: OpenAiCompatibleProvider,
}

impl OllamaProvider {
    /// Creates a new Ollama provider with default localhost URL.
    pub fn new(model: impl Into<String>) -> Self {
        Self::with_url(OLLAMA_DEFAULT_URL, model)
    }

    /// Creates a new Ollama provider with a custom URL.
    pub fn with_url(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        let model = model.into();
        let context_length = model_context_length(&model);

        let inner = OpenAiCompatibleProvider::custom(base_url, None, model)
            .with_context_length(context_length);

        Self { inner }
    }

    /// Creates a provider with llama3.2 model.
    pub fn llama3() -> Self {
        Self::new("llama3.2")
    }

    /// Creates a provider with mistral model.
    pub fn mistral() -> Self {
        Self::new("mistral")
    }

    /// Creates a provider with codellama model.
    pub fn codellama() -> Self {
        Self::new("codellama")
    }

    /// Overrides the context length.
    pub fn with_context_length(mut self, length: usize) -> Self {
        self.inner = self.inner.with_context_length(length);
        self
    }

    /// Overrides the HTTP client.
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.inner = self.inner.with_client(client);
        self
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn model(&self) -> &str {
        self.inner.model()
    }

    async fn complete(&self, request: &CompletionRequest) -> LlmResult<CompletionResponse> {
        self.inner.complete(request).await
    }

    async fn stream_complete(&self, request: &CompletionRequest) -> LlmResult<CompletionStream> {
        self.inner.stream_complete(request).await
    }

    fn supports_function_calling(&self) -> bool {
        // Most Ollama models don't support function calling
        // This could be extended to check specific models that do
        false
    }

    fn max_context_length(&self) -> usize {
        self.inner.max_context_length()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_context_length() {
        assert_eq!(model_context_length("llama3.2"), 128_000);
        assert_eq!(model_context_length("llama3.1"), 128_000);
        assert_eq!(model_context_length("llama3"), 8_192);
        assert_eq!(model_context_length("llama2"), 4_096);
        assert_eq!(model_context_length("mistral"), 32_768);
        assert_eq!(model_context_length("mixtral-8x7b"), 32_768);
        assert_eq!(model_context_length("codellama"), 16_384);
        assert_eq!(model_context_length("phi3"), 2_048);
        assert_eq!(model_context_length("gemma2:9b"), 8_192);
        assert_eq!(model_context_length("qwen2.5:32b"), 32_768);
        assert_eq!(model_context_length("unknown-model"), 4_096);
    }

    #[test]
    fn test_default_provider() {
        let provider = OllamaProvider::new("llama3.2");
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.model(), "llama3.2");
        assert_eq!(provider.max_context_length(), 128_000);
        assert!(!provider.supports_function_calling());
    }

    #[test]
    fn test_convenience_constructors() {
        let llama = OllamaProvider::llama3();
        assert_eq!(llama.model(), "llama3.2");

        let mistral = OllamaProvider::mistral();
        assert_eq!(mistral.model(), "mistral");

        let codellama = OllamaProvider::codellama();
        assert_eq!(codellama.model(), "codellama");
    }

    #[test]
    fn test_custom_url() {
        let provider = OllamaProvider::with_url("http://192.168.1.100:11434/v1", "llama3.2");
        assert_eq!(provider.model(), "llama3.2");
    }

    #[test]
    fn test_custom_context_length() {
        let provider = OllamaProvider::new("custom-model").with_context_length(16_384);
        assert_eq!(provider.max_context_length(), 16_384);
    }
}
