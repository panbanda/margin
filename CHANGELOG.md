# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure following SPARC methodology
- Domain layer types (Email, Thread, Account, Label, Contact, Screener)
- Storage layer with SQLite schema and queries
- Email provider trait with Gmail and IMAP stub implementations
- AI provider trait with OpenAI, Anthropic, and Ollama implementations
- Service layer (EmailService, AiService, SyncService)
- Configuration system with appearance, AI, notification, sync, and privacy settings
- Embedding engine for semantic search
- Basic gpui application scaffold
- GitHub Actions CI workflow with 95% coverage requirement
- Lefthook pre-commit hooks for formatting and linting
