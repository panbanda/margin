//! Embedding engine for semantic search.
//!
//! Uses Candle to run embedding models locally for privacy-preserving
//! semantic search over email content.

use anyhow::Result;

use crate::domain::{Email, EmailId};
use crate::embedding::VectorStore;

/// A vector embedding representing text semantics.
///
/// The embedding dimensionality depends on the model used
/// (e.g., 384 for MiniLM, 768 for BERT base).
#[derive(Debug, Clone)]
pub struct Embedding {
    /// The embedding vector.
    pub values: Vec<f32>,
}

impl Embedding {
    /// Creates a new embedding from a vector of values.
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }

    /// Returns the dimensionality of this embedding.
    pub fn dimension(&self) -> usize {
        self.values.len()
    }

    /// Computes cosine similarity with another embedding.
    ///
    /// Returns a value between -1.0 and 1.0, where 1.0 means identical.
    pub fn cosine_similarity(&self, other: &Embedding) -> f32 {
        if self.values.len() != other.values.len() {
            return 0.0;
        }

        let dot: f32 = self
            .values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = self.values.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.values.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot / (norm_a * norm_b)
    }
}

/// Configuration for the embedding engine.
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// Path to the model weights.
    pub model_path: Option<String>,
    /// Model identifier for downloading from Hugging Face.
    pub model_id: String,
    /// Maximum sequence length for tokenization.
    pub max_seq_length: usize,
    /// Whether to use GPU acceleration if available.
    pub use_gpu: bool,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_path: None,
            model_id: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_seq_length: 256,
            use_gpu: false,
        }
    }
}

/// Engine for generating text embeddings using local ML models.
///
/// The engine uses Candle for inference, avoiding external API calls
/// to maintain user privacy.
pub struct EmbeddingEngine {
    config: EmbeddingConfig,
    vector_store: VectorStore,
    // In a full implementation, these would hold the loaded model:
    // model: Option<CandleModel>,
    // tokenizer: Option<Tokenizer>,
}

impl EmbeddingEngine {
    /// Creates a new embedding engine with the given configuration.
    pub fn new(config: EmbeddingConfig, vector_store: VectorStore) -> Self {
        Self {
            config,
            vector_store,
        }
    }

    /// Creates an embedding engine with default configuration.
    pub fn with_defaults(vector_store: VectorStore) -> Self {
        Self::new(EmbeddingConfig::default(), vector_store)
    }

    /// Initializes the model, loading weights from disk or downloading if needed.
    ///
    /// This should be called before using `embed()` or `index_email()`.
    pub async fn initialize(&mut self) -> Result<()> {
        // Stub: In full implementation, this would:
        // 1. Check if model exists at model_path
        // 2. Download from Hugging Face if not present
        // 3. Load model weights into Candle
        // 4. Initialize tokenizer
        tracing::info!(
            model_id = %self.config.model_id,
            "Initializing embedding model (stub)"
        );
        Ok(())
    }

    /// Generates an embedding for the given text.
    ///
    /// The text is tokenized, passed through the model, and the output
    /// is pooled to produce a fixed-size embedding vector.
    pub fn embed(&self, text: &str) -> Result<Embedding> {
        // Stub: In full implementation, this would:
        // 1. Tokenize text with max_seq_length truncation
        // 2. Run forward pass through the model
        // 3. Apply mean pooling over token embeddings
        // 4. Normalize the result

        // For now, return a deterministic pseudo-embedding based on text hash
        let hash = Self::simple_hash(text);
        let dimension = 384; // MiniLM dimension
        let values: Vec<f32> = (0..dimension)
            .map(|i| {
                let seed = hash.wrapping_add(i as u64);
                (seed as f32 / u64::MAX as f32) * 2.0 - 1.0
            })
            .collect();

        Ok(Embedding::new(values))
    }

    /// Searches for emails similar to the query embedding.
    ///
    /// Returns email IDs with their similarity scores, sorted by relevance.
    pub fn search(&self, query_embedding: &Embedding, limit: usize) -> Result<Vec<(EmailId, f32)>> {
        self.vector_store.search(query_embedding, limit)
    }

    /// Indexes an email by generating and storing its embedding.
    ///
    /// Combines subject and body text for a comprehensive representation.
    pub fn index_email(&mut self, email: &Email) -> Result<()> {
        let text = Self::email_to_text(email);
        let embedding = self.embed(&text)?;
        self.vector_store.insert(&email.id, embedding)?;
        Ok(())
    }

    /// Returns the underlying vector store.
    pub fn vector_store(&self) -> &VectorStore {
        &self.vector_store
    }

    /// Returns a mutable reference to the underlying vector store.
    pub fn vector_store_mut(&mut self) -> &mut VectorStore {
        &mut self.vector_store
    }

    /// Converts an email to indexable text.
    fn email_to_text(email: &Email) -> String {
        let mut parts = Vec::new();

        if let Some(subject) = &email.subject {
            parts.push(subject.clone());
        }

        if let Some(body) = &email.body_text {
            parts.push(body.clone());
        }

        parts.join(" ")
    }

    /// Simple non-cryptographic hash for deterministic stub embeddings.
    fn simple_hash(text: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AccountId, Address, MessageId, ThreadId};
    use chrono::Utc;

    fn make_test_email(id: &str, subject: &str, body: &str) -> Email {
        Email {
            id: EmailId::from(id),
            account_id: AccountId::from("test-account"),
            thread_id: ThreadId::from("thread-1"),
            message_id: MessageId::from("<test@example.com>"),
            in_reply_to: None,
            references: vec![],
            from: Address::new("sender@example.com"),
            to: vec![Address::new("recipient@example.com")],
            cc: vec![],
            bcc: vec![],
            subject: Some(subject.to_string()),
            body_text: Some(body.to_string()),
            body_html: None,
            snippet: body.chars().take(50).collect(),
            date: Utc::now(),
            is_read: false,
            is_starred: false,
            is_draft: false,
            labels: vec![],
            attachments: vec![],
        }
    }

    #[test]
    fn embedding_dimension() {
        let embedding = Embedding::new(vec![0.1, 0.2, 0.3]);
        assert_eq!(embedding.dimension(), 3);
    }

    #[test]
    fn cosine_similarity_identical() {
        let a = Embedding::new(vec![1.0, 0.0, 0.0]);
        let b = Embedding::new(vec![1.0, 0.0, 0.0]);
        let similarity = a.cosine_similarity(&b);
        assert!((similarity - 1.0).abs() < 0.0001);
    }

    #[test]
    fn cosine_similarity_orthogonal() {
        let a = Embedding::new(vec![1.0, 0.0]);
        let b = Embedding::new(vec![0.0, 1.0]);
        let similarity = a.cosine_similarity(&b);
        assert!(similarity.abs() < 0.0001);
    }

    #[test]
    fn cosine_similarity_opposite() {
        let a = Embedding::new(vec![1.0, 0.0]);
        let b = Embedding::new(vec![-1.0, 0.0]);
        let similarity = a.cosine_similarity(&b);
        assert!((similarity + 1.0).abs() < 0.0001);
    }

    #[test]
    fn cosine_similarity_mismatched_dims() {
        let a = Embedding::new(vec![1.0, 0.0]);
        let b = Embedding::new(vec![1.0, 0.0, 0.0]);
        let similarity = a.cosine_similarity(&b);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn embed_produces_consistent_output() {
        let store = VectorStore::new();
        let engine = EmbeddingEngine::with_defaults(store);

        let text = "Hello, world!";
        let emb1 = engine.embed(text).unwrap();
        let emb2 = engine.embed(text).unwrap();

        assert_eq!(emb1.values, emb2.values);
    }

    #[test]
    fn embed_different_texts_produce_different_embeddings() {
        let store = VectorStore::new();
        let engine = EmbeddingEngine::with_defaults(store);

        let emb1 = engine.embed("Hello").unwrap();
        let emb2 = engine.embed("Goodbye").unwrap();

        assert_ne!(emb1.values, emb2.values);
    }

    #[test]
    fn index_and_search_email() {
        let store = VectorStore::new();
        let mut engine = EmbeddingEngine::with_defaults(store);

        let email = make_test_email("email-1", "Meeting tomorrow", "Let's discuss the project.");
        engine.index_email(&email).unwrap();

        let query = engine.embed("project meeting").unwrap();
        let results = engine.search(&query, 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, EmailId::from("email-1"));
    }

    #[test]
    fn email_to_text_combines_subject_and_body() {
        let email = make_test_email("email-1", "Subject here", "Body content here");
        let text = EmbeddingEngine::email_to_text(&email);
        assert!(text.contains("Subject here"));
        assert!(text.contains("Body content here"));
    }

    #[test]
    fn default_config() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.model_id, "sentence-transformers/all-MiniLM-L6-v2");
        assert_eq!(config.max_seq_length, 256);
        assert!(!config.use_gpu);
    }
}
