//! Vector storage for semantic search.
//!
//! Stores email embeddings and provides similarity search functionality.
//! Uses in-memory storage with optional SQLite persistence.

use anyhow::Result;
use std::collections::HashMap;

use crate::domain::EmailId;
use crate::embedding::Embedding;

/// In-memory vector store with similarity search.
///
/// Stores embedding vectors indexed by email ID and supports
/// nearest-neighbor search using cosine similarity.
#[derive(Debug, Default)]
pub struct VectorStore {
    /// Map of email IDs to their embeddings.
    embeddings: HashMap<EmailId, Embedding>,
}

impl VectorStore {
    /// Creates a new empty vector store.
    pub fn new() -> Self {
        Self {
            embeddings: HashMap::new(),
        }
    }

    /// Inserts or updates an embedding for the given email ID.
    pub fn insert(&mut self, email_id: &EmailId, embedding: Embedding) -> Result<()> {
        self.embeddings.insert(email_id.clone(), embedding);
        Ok(())
    }

    /// Retrieves the embedding for an email, if it exists.
    pub fn get(&self, email_id: &EmailId) -> Option<&Embedding> {
        self.embeddings.get(email_id)
    }

    /// Removes the embedding for an email.
    pub fn remove(&mut self, email_id: &EmailId) -> Option<Embedding> {
        self.embeddings.remove(email_id)
    }

    /// Returns whether an embedding exists for the given email.
    pub fn contains(&self, email_id: &EmailId) -> bool {
        self.embeddings.contains_key(email_id)
    }

    /// Returns the number of stored embeddings.
    pub fn len(&self) -> usize {
        self.embeddings.len()
    }

    /// Returns whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }

    /// Searches for the most similar embeddings to the query.
    ///
    /// Returns up to `limit` results as (EmailId, similarity_score) pairs,
    /// sorted by similarity in descending order.
    pub fn search(&self, query: &Embedding, limit: usize) -> Result<Vec<(EmailId, f32)>> {
        let mut scores: Vec<(EmailId, f32)> = self
            .embeddings
            .iter()
            .map(|(id, emb)| (id.clone(), query.cosine_similarity(emb)))
            .collect();

        // Sort by similarity descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scores.truncate(limit);
        Ok(scores)
    }

    /// Searches with a minimum similarity threshold.
    ///
    /// Only returns results with similarity >= min_similarity.
    pub fn search_with_threshold(
        &self,
        query: &Embedding,
        limit: usize,
        min_similarity: f32,
    ) -> Result<Vec<(EmailId, f32)>> {
        let mut scores: Vec<(EmailId, f32)> = self
            .embeddings
            .iter()
            .map(|(id, emb)| (id.clone(), query.cosine_similarity(emb)))
            .filter(|(_, score)| *score >= min_similarity)
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scores.truncate(limit);
        Ok(scores)
    }

    /// Clears all stored embeddings.
    pub fn clear(&mut self) {
        self.embeddings.clear();
    }

    /// Returns an iterator over all email IDs in the store.
    pub fn email_ids(&self) -> impl Iterator<Item = &EmailId> {
        self.embeddings.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_embedding(values: &[f32]) -> Embedding {
        Embedding::new(values.to_vec())
    }

    #[test]
    fn insert_and_get() {
        let mut store = VectorStore::new();
        let id = EmailId::from("email-1");
        let embedding = make_embedding(&[1.0, 0.0, 0.0]);

        store.insert(&id, embedding.clone()).unwrap();

        let retrieved = store.get(&id).unwrap();
        assert_eq!(retrieved.values, vec![1.0, 0.0, 0.0]);
    }

    #[test]
    fn insert_updates_existing() {
        let mut store = VectorStore::new();
        let id = EmailId::from("email-1");

        store.insert(&id, make_embedding(&[1.0, 0.0])).unwrap();
        store.insert(&id, make_embedding(&[0.0, 1.0])).unwrap();

        let retrieved = store.get(&id).unwrap();
        assert_eq!(retrieved.values, vec![0.0, 1.0]);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn remove() {
        let mut store = VectorStore::new();
        let id = EmailId::from("email-1");

        store.insert(&id, make_embedding(&[1.0])).unwrap();
        assert!(store.contains(&id));

        let removed = store.remove(&id);
        assert!(removed.is_some());
        assert!(!store.contains(&id));
    }

    #[test]
    fn len_and_is_empty() {
        let mut store = VectorStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        store
            .insert(&EmailId::from("email-1"), make_embedding(&[1.0]))
            .unwrap();
        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn search_returns_sorted_results() {
        let mut store = VectorStore::new();

        // Insert embeddings at different angles from query
        store
            .insert(&EmailId::from("exact"), make_embedding(&[1.0, 0.0]))
            .unwrap();
        store
            .insert(&EmailId::from("similar"), make_embedding(&[0.9, 0.1]))
            .unwrap();
        store
            .insert(&EmailId::from("different"), make_embedding(&[0.0, 1.0]))
            .unwrap();

        let query = make_embedding(&[1.0, 0.0]);
        let results = store.search(&query, 10).unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, EmailId::from("exact"));
        assert!((results[0].1 - 1.0).abs() < 0.0001);
    }

    #[test]
    fn search_respects_limit() {
        let mut store = VectorStore::new();

        for i in 0..10 {
            store
                .insert(
                    &EmailId::from(format!("email-{}", i)),
                    make_embedding(&[1.0]),
                )
                .unwrap();
        }

        let query = make_embedding(&[1.0]);
        let results = store.search(&query, 3).unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn search_with_threshold() {
        let mut store = VectorStore::new();

        store
            .insert(&EmailId::from("high"), make_embedding(&[1.0, 0.0]))
            .unwrap();
        store
            .insert(&EmailId::from("medium"), make_embedding(&[0.7, 0.7]))
            .unwrap();
        store
            .insert(&EmailId::from("low"), make_embedding(&[0.0, 1.0]))
            .unwrap();

        let query = make_embedding(&[1.0, 0.0]);
        let results = store.search_with_threshold(&query, 10, 0.8).unwrap();

        // Only "high" should have similarity >= 0.8 (medium has ~0.707)
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, EmailId::from("high"));
    }

    #[test]
    fn clear() {
        let mut store = VectorStore::new();

        store
            .insert(&EmailId::from("email-1"), make_embedding(&[1.0]))
            .unwrap();
        store
            .insert(&EmailId::from("email-2"), make_embedding(&[2.0]))
            .unwrap();

        store.clear();

        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn email_ids_iterator() {
        let mut store = VectorStore::new();

        store
            .insert(&EmailId::from("a"), make_embedding(&[1.0]))
            .unwrap();
        store
            .insert(&EmailId::from("b"), make_embedding(&[2.0]))
            .unwrap();

        let ids: Vec<_> = store.email_ids().collect();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn search_empty_store() {
        let store = VectorStore::new();
        let query = make_embedding(&[1.0, 0.0]);
        let results = store.search(&query, 10).unwrap();
        assert!(results.is_empty());
    }
}
