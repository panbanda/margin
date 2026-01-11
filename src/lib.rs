//! margin - A keyboard-driven, AI-augmented email client
//!
//! This crate provides the core functionality for the margin email client,
//! including email protocol handling, AI services, and storage management.

pub mod app;
pub mod config;
pub mod domain;
pub mod embedding;
pub mod providers;
pub mod services;
pub mod storage;
pub mod ui;

pub use app::App;
