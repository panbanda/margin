//! Email and AI provider implementations.
//!
//! This module contains provider traits and implementations for external services:
//!
//! - [`email`] - Email providers (Gmail API, IMAP/SMTP)
//! - [`ai`] - AI/LLM providers (OpenAI, Anthropic, Ollama)

pub mod ai;
pub mod email;
