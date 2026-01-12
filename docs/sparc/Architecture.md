# The Heap - Architecture Document

> SPARC Phase 3: Architecture
> Version: 0.1.0
> Last Updated: 2025-01-11

## Overview

The Heap is a native desktop email client built entirely in Rust, using gpui for the UI layer. The architecture prioritizes:

- **Performance**: Native code, GPU-accelerated rendering, efficient data structures
- **Privacy**: Local-first data storage, no telemetry, user-controlled AI
- **Modularity**: Clear separation of concerns for maintainability
- **Extensibility**: Well-defined interfaces for future expansion

## Technology Stack

### Core

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Language | Rust | Memory safety, performance, ecosystem |
| UI Framework | gpui | GPU-accelerated, Tailwind-style API, action system for shortcuts |
| Database | SQLite (rusqlite) | Embedded, reliable, SQL query capability |
| Async Runtime | tokio | Industry standard, excellent ecosystem |
| Serialization | serde + serde_json | De facto Rust standard |
| HTTP Client | reqwest | Async, TLS, connection pooling |

### Email Protocols

| Protocol | Library | Notes |
|----------|---------|-------|
| IMAP | async-imap | Async IMAP4rev1 |
| SMTP | lettre | Mature, async support |
| Gmail API | google-apis-rs or custom | OAuth 2.0, REST |
| MIME Parsing | mailparse | RFC 5322 compliant |

### AI & ML

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Local Inference | Candle | Pure Rust, no Python dependency |
| Embedding Models | BERT/MiniLM via Candle | Local semantic search |
| LLM Integration | OpenAI-compatible API | Universal compatibility |
| Vector Storage | SQLite + custom indexing | Keep stack simple |

### Security

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Credential Storage | keyring-rs | OS keychain integration |
| Encryption | ring | Audited, fast |
| TLS | rustls | Pure Rust, memory safe |
| OAuth | oauth2-rs | Standard flows |

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                The Heap                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Presentation Layer                           │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           │   │
│  │  │  Sidebar  │ │  Message  │ │  Reading  │ │  Compose  │           │   │
│  │  │   View    │ │   List    │ │   Pane    │ │   Editor  │           │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘           │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           │   │
│  │  │  Command  │ │  Settings │ │   Stats   │ │  Screener │           │   │
│  │  │  Palette  │ │   Panel   │ │ Dashboard │ │   Queue   │           │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘           │   │
│  │                         gpui Views & Components                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                     │                                       │
│                                     ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Application Layer                            │   │
│  │  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐              │   │
│  │  │    Actions    │ │     State     │ │    Events     │              │   │
│  │  │   Registry    │ │   Management  │ │     Bus       │              │   │
│  │  └───────────────┘ └───────────────┘ └───────────────┘              │   │
│  │  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐              │   │
│  │  │   Keybinding  │ │  Notification │ │   Telemetry   │              │   │
│  │  │    Manager    │ │    Service    │ │   Collector   │              │   │
│  │  └───────────────┘ └───────────────┘ └───────────────┘              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                     │                                       │
│                                     ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                          Domain Layer                                │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │   │
│  │  │  Email   │ │  Thread  │ │  Account │ │  Label   │ │  Contact │  │   │
│  │  │  Service │ │  Service │ │  Service │ │  Service │ │  Service │  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐              │   │
│  │  │   AI     │ │  Search  │ │  Sync    │ │ Screener │              │   │
│  │  │  Service │ │  Service │ │  Service │ │  Service │              │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                     │                                       │
│                                     ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                       Infrastructure Layer                           │   │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                 │   │
│  │  │    Email     │ │      AI      │ │   Storage    │                 │   │
│  │  │   Providers  │ │   Providers  │ │    Layer     │                 │   │
│  │  │              │ │              │ │              │                 │   │
│  │  │ ┌──────────┐ │ │ ┌──────────┐ │ │ ┌──────────┐ │                 │   │
│  │  │ │  Gmail   │ │ │ │  OpenAI  │ │ │ │  SQLite  │ │                 │   │
│  │  │ │   API    │ │ │ │   API    │ │ │ │  (main)  │ │                 │   │
│  │  │ └──────────┘ │ │ └──────────┘ │ │ └──────────┘ │                 │   │
│  │  │ ┌──────────┐ │ │ ┌──────────┐ │ │ ┌──────────┐ │                 │   │
│  │  │ │   IMAP   │ │ │ │Anthropic │ │ │ │ Postgres │ │                 │   │
│  │  │ │   SMTP   │ │ │ │   API    │ │ │ │  (BYOD)  │ │                 │   │
│  │  │ └──────────┘ │ │ └──────────┘ │ │ └──────────┘ │                 │   │
│  │  │              │ │ ┌──────────┐ │ │ ┌──────────┐ │                 │   │
│  │  │              │ │ │  Ollama  │ │ │ │ Keychain │ │                 │   │
│  │  │              │ │ │  (local) │ │ │ │          │ │                 │   │
│  │  │              │ │ └──────────┘ │ │ └──────────┘ │                 │   │
│  │  │              │ │ ┌──────────┐ │ │              │                 │   │
│  │  │              │ │ │  Candle  │ │ │              │                 │   │
│  │  │              │ │ │(embedded)│ │ │              │                 │   │
│  │  │              │ │ └──────────┘ │ │              │                 │   │
│  │  └──────────────┘ └──────────────┘ └──────────────┘                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Architecture

### Presentation Layer

#### View Hierarchy

```
App
├── MainWindow
│   ├── TitleBar (account switcher, search, status)
│   ├── Sidebar
│   │   ├── AccountList
│   │   ├── MailboxList
│   │   ├── LabelList
│   │   └── SmartViewList
│   ├── ContentArea
│   │   ├── MessageList
│   │   │   ├── MessageListItem (virtualized)
│   │   │   └── EmptyState
│   │   └── ReadingPane
│   │       ├── ThreadHeader
│   │       ├── MessageView (per message)
│   │       └── InlineComposer
│   └── StatusBar
├── CommandPalette (overlay)
├── ComposeWindow (pop-out)
├── SettingsWindow
└── StatsWindow
```

#### gpui Component Pattern

```rust
// Example: MessageListItem component
struct MessageListItem {
    thread_id: ThreadId,
    subject: SharedString,
    sender: SharedString,
    preview: SharedString,
    timestamp: DateTime<Utc>,
    is_unread: bool,
    is_starred: bool,
    is_selected: bool,
}

impl Render for MessageListItem {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .id(self.thread_id.to_string())
            .px_3()
            .py_2()
            .bg(if self.is_selected {
                cx.theme().colors().surface_elevated
            } else {
                cx.theme().colors().surface
            })
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .font_weight(if self.is_unread {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .child(self.subject.clone())
                    )
                    .child(self.timestamp.format("%b %d").to_string())
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().colors().text_secondary)
                    .child(format!("{} - {}", self.sender, self.preview))
            )
    }
}
```

#### Action System

```rust
// Define actions for keyboard shortcuts
actions!(
    heap,
    [
        Compose,
        Reply,
        ReplyAll,
        Forward,
        Archive,
        Trash,
        Star,
        Snooze,
        MarkRead,
        MarkUnread,
        NextMessage,
        PreviousMessage,
        OpenThread,
        GoToInbox,
        GoToStarred,
        GoToDrafts,
        OpenCommandPalette,
        Search,
    ]
);

// Register keybindings
fn register_keybindings(cx: &mut AppContext) {
    cx.bind_keys([
        KeyBinding::new("c", Compose, None),
        KeyBinding::new("r", Reply, None),
        KeyBinding::new("enter", ReplyAll, Some("MessageView")),
        KeyBinding::new("f", Forward, None),
        KeyBinding::new("e", Archive, None),
        KeyBinding::new("shift-3", Trash, None),
        KeyBinding::new("s", Star, None),
        KeyBinding::new("h", Snooze, None),
        KeyBinding::new("u", MarkRead, None),  // toggles
        KeyBinding::new("j", NextMessage, None),
        KeyBinding::new("k", PreviousMessage, None),
        KeyBinding::new("enter", OpenThread, Some("MessageList")),
        KeyBinding::new("g i", GoToInbox, None),
        KeyBinding::new("g s", GoToStarred, None),
        KeyBinding::new("g d", GoToDrafts, None),
        KeyBinding::new("cmd-k", OpenCommandPalette, None),
        KeyBinding::new("/", Search, None),
    ]);
}
```

### Application Layer

#### State Management

```rust
// Global application state
pub struct AppState {
    pub accounts: Vec<Account>,
    pub active_account_id: Option<AccountId>,
    pub active_view: ViewType,
    pub selected_threads: Vec<ThreadId>,
    pub settings: Settings,
    pub sync_status: SyncStatus,
    pub ai_status: AiStatus,
}

// View-specific state
pub struct MessageListState {
    pub threads: Vec<ThreadSummary>,
    pub selected_index: usize,
    pub scroll_position: f32,
    pub filter: Option<Filter>,
}

pub struct ReadingPaneState {
    pub thread: Option<Thread>,
    pub expanded_messages: HashSet<MessageId>,
    pub composer_visible: bool,
    pub composer_content: String,
}
```

#### Event Bus

```rust
// Domain events for cross-component communication
pub enum AppEvent {
    // Account events
    AccountAdded(AccountId),
    AccountRemoved(AccountId),
    AccountSyncStarted(AccountId),
    AccountSyncCompleted(AccountId, SyncResult),

    // Email events
    EmailReceived(AccountId, Vec<EmailId>),
    EmailSent(AccountId, EmailId),
    EmailArchived(AccountId, Vec<ThreadId>),
    EmailTrashed(AccountId, Vec<ThreadId>),
    EmailStarred(AccountId, ThreadId, bool),

    // AI events
    AiTaskStarted(TaskId, AiTaskType),
    AiTaskCompleted(TaskId, AiResult),
    AiTaskFailed(TaskId, Error),

    // UI events
    NavigateTo(ViewType),
    SelectThread(ThreadId),
    OpenComposer(ComposerMode),
    ShowNotification(Notification),
}
```

### Domain Layer

#### Email Service

```rust
pub struct EmailService {
    providers: HashMap<AccountId, Box<dyn EmailProvider>>,
    storage: Arc<StorageLayer>,
    event_bus: EventBus,
}

impl EmailService {
    pub async fn fetch_threads(
        &self,
        account_id: &AccountId,
        view: ViewType,
        pagination: Pagination,
    ) -> Result<Vec<ThreadSummary>>;

    pub async fn get_thread(&self, thread_id: &ThreadId) -> Result<Thread>;

    pub async fn send_email(&self, draft: Draft) -> Result<EmailId>;

    pub async fn archive(&self, thread_ids: &[ThreadId]) -> Result<()>;

    pub async fn trash(&self, thread_ids: &[ThreadId]) -> Result<()>;

    pub async fn star(&self, thread_id: &ThreadId, starred: bool) -> Result<()>;

    pub async fn apply_label(
        &self,
        thread_ids: &[ThreadId],
        label_id: &LabelId,
    ) -> Result<()>;

    pub async fn snooze(
        &self,
        thread_id: &ThreadId,
        until: DateTime<Utc>,
    ) -> Result<()>;
}
```

#### AI Service

```rust
pub struct AiService {
    providers: HashMap<String, Box<dyn LlmProvider>>,
    embedding_engine: EmbeddingEngine,
    settings: AiSettings,
    telemetry: TelemetryCollector,
}

impl AiService {
    pub async fn summarize_thread(&self, thread: &Thread) -> Result<Summary> {
        let provider = self.get_provider_for_task(AiTaskType::Summary)?;
        let prompt = self.build_summary_prompt(thread);

        self.telemetry.record_ai_request(AiTaskType::Summary);

        let response = provider.complete(&prompt).await?;

        self.telemetry.record_ai_response(
            AiTaskType::Summary,
            response.tokens_used,
        );

        Ok(Summary::parse(response.text))
    }

    pub async fn draft_reply(
        &self,
        thread: &Thread,
        instructions: Option<&str>,
    ) -> Result<DraftSuggestion>;

    pub async fn semantic_search(
        &self,
        query: &str,
        account_ids: &[AccountId],
    ) -> Result<Vec<SearchResult>>;

    pub async fn categorize_email(&self, email: &Email) -> Result<Vec<Category>>;

    pub async fn analyze_sender(&self, sender: &str) -> Result<SenderAnalysis>;
}

// Embedding engine for local semantic search
pub struct EmbeddingEngine {
    model: CandleModel,
    vector_store: VectorStore,
}

impl EmbeddingEngine {
    pub fn embed(&self, text: &str) -> Result<Embedding>;

    pub fn search(
        &self,
        query_embedding: &Embedding,
        limit: usize,
    ) -> Result<Vec<(EmailId, f32)>>;

    pub fn index_email(&self, email: &Email) -> Result<()>;
}
```

#### Sync Service

```rust
pub struct SyncService {
    email_service: Arc<EmailService>,
    storage: Arc<StorageLayer>,
    settings: SyncSettings,
    scheduler: SyncScheduler,
}

impl SyncService {
    pub async fn sync_account(&self, account_id: &AccountId) -> Result<SyncResult> {
        let provider = self.email_service.get_provider(account_id)?;

        // Get local state
        let local_state = self.storage.get_sync_state(account_id).await?;

        // Fetch changes from server
        let changes = provider.fetch_changes_since(&local_state.last_sync).await?;

        // Apply changes locally
        for change in changes {
            match change {
                Change::NewEmail(email) => {
                    self.storage.insert_email(&email).await?;
                }
                Change::Updated(email_id, updates) => {
                    self.storage.update_email(&email_id, &updates).await?;
                }
                Change::Deleted(email_id) => {
                    self.storage.delete_email(&email_id).await?;
                }
            }
        }

        // Push local changes
        let pending = self.storage.get_pending_changes(account_id).await?;
        for change in pending {
            provider.push_change(&change).await?;
            self.storage.mark_change_synced(&change.id).await?;
        }

        // Update sync state
        self.storage.update_sync_state(account_id, SyncState::now()).await?;

        Ok(SyncResult {
            emails_received: changes.len(),
            emails_sent: pending.len(),
        })
    }

    pub fn start_background_sync(&self);

    pub fn stop_background_sync(&self);

    pub fn get_sync_status(&self, account_id: &AccountId) -> SyncStatus;
}
```

### Infrastructure Layer

#### Email Provider Trait

```rust
#[async_trait]
pub trait EmailProvider: Send + Sync {
    fn provider_type(&self) -> ProviderType;

    async fn authenticate(&mut self) -> Result<()>;

    async fn fetch_threads(
        &self,
        folder: &str,
        pagination: Pagination,
    ) -> Result<Vec<ThreadSummary>>;

    async fn fetch_thread(&self, thread_id: &str) -> Result<Thread>;

    async fn fetch_changes_since(
        &self,
        since: &DateTime<Utc>,
    ) -> Result<Vec<Change>>;

    async fn send_email(&self, email: &OutgoingEmail) -> Result<String>;

    async fn archive(&self, thread_ids: &[String]) -> Result<()>;

    async fn trash(&self, thread_ids: &[String]) -> Result<()>;

    async fn star(&self, thread_id: &str, starred: bool) -> Result<()>;

    async fn mark_read(&self, thread_id: &str, read: bool) -> Result<()>;

    async fn apply_label(&self, thread_id: &str, label: &str) -> Result<()>;

    async fn fetch_labels(&self) -> Result<Vec<Label>>;

    async fn push_change(&self, change: &PendingChange) -> Result<()>;
}

// Gmail implementation
pub struct GmailProvider {
    client: GmailClient,
    token: OAuth2Token,
    account_id: AccountId,
}

// IMAP implementation
pub struct ImapProvider {
    imap_client: ImapClient,
    smtp_client: SmtpClient,
    config: ImapConfig,
    account_id: AccountId,
}
```

#### LLM Provider Trait

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse>;

    async fn stream_complete(
        &self,
        request: &CompletionRequest,
    ) -> Result<impl Stream<Item = Result<String>>>;

    fn supports_function_calling(&self) -> bool;

    fn max_context_length(&self) -> usize;
}

pub struct CompletionRequest {
    pub system_prompt: Option<String>,
    pub messages: Vec<Message>,
    pub temperature: f32,
    pub max_tokens: Option<usize>,
}

pub struct CompletionResponse {
    pub text: String,
    pub tokens_used: TokenUsage,
    pub finish_reason: FinishReason,
}

// OpenAI-compatible implementation (works with OpenAI, Ollama, vLLM, etc.)
pub struct OpenAiCompatibleProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
}

// Anthropic implementation
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}
```

#### Storage Layer

```rust
pub struct StorageLayer {
    db: SqlitePool,
    keychain: KeychainAccess,
}

impl StorageLayer {
    // Email operations
    pub async fn insert_email(&self, email: &Email) -> Result<()>;
    pub async fn get_email(&self, email_id: &EmailId) -> Result<Email>;
    pub async fn update_email(&self, email_id: &EmailId, updates: &EmailUpdates) -> Result<()>;
    pub async fn delete_email(&self, email_id: &EmailId) -> Result<()>;
    pub async fn search_emails(&self, query: &SearchQuery) -> Result<Vec<EmailSummary>>;

    // Thread operations
    pub async fn get_thread(&self, thread_id: &ThreadId) -> Result<Thread>;
    pub async fn get_threads(&self, view: ViewType, pagination: Pagination) -> Result<Vec<ThreadSummary>>;

    // Account operations
    pub async fn get_accounts(&self) -> Result<Vec<Account>>;
    pub async fn insert_account(&self, account: &Account) -> Result<()>;
    pub async fn delete_account(&self, account_id: &AccountId) -> Result<()>;

    // Sync state
    pub async fn get_sync_state(&self, account_id: &AccountId) -> Result<SyncState>;
    pub async fn update_sync_state(&self, account_id: &AccountId, state: SyncState) -> Result<()>;
    pub async fn get_pending_changes(&self, account_id: &AccountId) -> Result<Vec<PendingChange>>;

    // Credentials (via keychain)
    pub async fn store_credential(&self, key: &str, value: &str) -> Result<()>;
    pub async fn get_credential(&self, key: &str) -> Result<Option<String>>;
    pub async fn delete_credential(&self, key: &str) -> Result<()>;
}
```

## Data Models

### Database Schema

```sql
-- Accounts
CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    display_name TEXT,
    provider_type TEXT NOT NULL,  -- 'gmail', 'imap'
    provider_config TEXT NOT NULL, -- JSON
    sync_enabled INTEGER DEFAULT 1,
    sync_interval_seconds INTEGER DEFAULT 300,
    signature TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Emails
CREATE TABLE emails (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    thread_id TEXT NOT NULL,
    message_id TEXT NOT NULL,  -- RFC 5322 Message-ID
    in_reply_to TEXT,
    references TEXT,  -- JSON array
    from_address TEXT NOT NULL,
    from_name TEXT,
    to_addresses TEXT NOT NULL,  -- JSON array
    cc_addresses TEXT,  -- JSON array
    bcc_addresses TEXT,  -- JSON array
    subject TEXT,
    body_text TEXT,
    body_html TEXT,
    snippet TEXT,
    date TEXT NOT NULL,
    is_read INTEGER DEFAULT 0,
    is_starred INTEGER DEFAULT 0,
    is_draft INTEGER DEFAULT 0,
    labels TEXT,  -- JSON array
    raw_headers TEXT,  -- JSON
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_emails_account ON emails(account_id);
CREATE INDEX idx_emails_thread ON emails(thread_id);
CREATE INDEX idx_emails_date ON emails(date DESC);
CREATE INDEX idx_emails_from ON emails(from_address);

-- Threads (denormalized for performance)
CREATE TABLE threads (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    subject TEXT,
    snippet TEXT,
    participant_emails TEXT NOT NULL,  -- JSON array
    participant_names TEXT,  -- JSON array
    last_message_date TEXT NOT NULL,
    message_count INTEGER DEFAULT 1,
    unread_count INTEGER DEFAULT 0,
    is_starred INTEGER DEFAULT 0,
    labels TEXT,  -- JSON array
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_threads_account ON threads(account_id);
CREATE INDEX idx_threads_date ON threads(last_message_date DESC);

-- Labels
CREATE TABLE labels (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    name TEXT NOT NULL,
    color TEXT,
    is_system INTEGER DEFAULT 0,
    provider_id TEXT,  -- Provider's label ID
    created_at TEXT NOT NULL
);

-- Attachments
CREATE TABLE attachments (
    id TEXT PRIMARY KEY,
    email_id TEXT NOT NULL REFERENCES emails(id),
    filename TEXT NOT NULL,
    content_type TEXT,
    size_bytes INTEGER,
    content_id TEXT,  -- For inline attachments
    is_inline INTEGER DEFAULT 0,
    local_path TEXT,  -- Path to cached file
    created_at TEXT NOT NULL
);

-- Drafts
CREATE TABLE drafts (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    reply_to_thread_id TEXT,
    reply_to_message_id TEXT,
    to_addresses TEXT,  -- JSON array
    cc_addresses TEXT,
    bcc_addresses TEXT,
    subject TEXT,
    body_markdown TEXT,
    body_html TEXT,
    attachments TEXT,  -- JSON array of attachment IDs
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Contacts (extracted from emails)
CREATE TABLE contacts (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT,
    frequency INTEGER DEFAULT 1,
    last_contacted TEXT,
    is_vip INTEGER DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_contacts_email ON contacts(email);

-- Screener
CREATE TABLE screener_entries (
    id TEXT PRIMARY KEY,
    sender_email TEXT NOT NULL,
    sender_name TEXT,
    first_email_id TEXT REFERENCES emails(id),
    status TEXT NOT NULL,  -- 'pending', 'approved', 'rejected'
    ai_analysis TEXT,  -- JSON
    decided_at TEXT,
    created_at TEXT NOT NULL
);

-- Screener rules
CREATE TABLE screener_rules (
    id TEXT PRIMARY KEY,
    rule_type TEXT NOT NULL,  -- 'domain_allow', 'domain_block', 'pattern'
    pattern TEXT NOT NULL,
    action TEXT NOT NULL,  -- 'approve', 'reject'
    created_at TEXT NOT NULL
);

-- Snooze
CREATE TABLE snoozed (
    id TEXT PRIMARY KEY,
    thread_id TEXT NOT NULL REFERENCES threads(id),
    snooze_until TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_snoozed_until ON snoozed(snooze_until);

-- Sync state
CREATE TABLE sync_state (
    account_id TEXT PRIMARY KEY REFERENCES accounts(id),
    last_sync TEXT,
    last_history_id TEXT,  -- Gmail-specific
    last_uid_validity INTEGER,  -- IMAP-specific
    last_uid INTEGER  -- IMAP-specific
);

-- Pending changes (offline queue)
CREATE TABLE pending_changes (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    change_type TEXT NOT NULL,
    payload TEXT NOT NULL,  -- JSON
    created_at TEXT NOT NULL
);

-- Vector embeddings (for semantic search)
CREATE TABLE embeddings (
    email_id TEXT PRIMARY KEY REFERENCES emails(id),
    embedding BLOB NOT NULL,  -- Binary float array
    created_at TEXT NOT NULL
);

-- Telemetry (local stats)
CREATE TABLE telemetry_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    payload TEXT,  -- JSON
    timestamp TEXT NOT NULL
);

CREATE INDEX idx_telemetry_timestamp ON telemetry_events(timestamp);

-- Aggregated stats
CREATE TABLE daily_stats (
    date TEXT NOT NULL,
    account_id TEXT,  -- NULL for global stats
    emails_received INTEGER DEFAULT 0,
    emails_sent INTEGER DEFAULT 0,
    emails_archived INTEGER DEFAULT 0,
    emails_trashed INTEGER DEFAULT 0,
    time_in_app_seconds INTEGER DEFAULT 0,
    sessions INTEGER DEFAULT 0,
    ai_summaries INTEGER DEFAULT 0,
    ai_drafts INTEGER DEFAULT 0,
    ai_searches INTEGER DEFAULT 0,
    ai_tokens_used INTEGER DEFAULT 0,
    PRIMARY KEY (date, account_id)
);

-- Settings
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### Domain Types

```rust
// Core identifiers
pub struct AccountId(pub String);
pub struct ThreadId(pub String);
pub struct EmailId(pub String);
pub struct MessageId(pub String);  // RFC 5322
pub struct LabelId(pub String);

// Account
pub struct Account {
    pub id: AccountId,
    pub email: String,
    pub display_name: Option<String>,
    pub provider_type: ProviderType,
    pub provider_config: ProviderConfig,
    pub sync_enabled: bool,
    pub sync_interval: Duration,
    pub signature: Option<String>,
}

pub enum ProviderType {
    Gmail,
    Imap,
}

pub enum ProviderConfig {
    Gmail {
        // OAuth tokens stored in keychain
    },
    Imap {
        imap_host: String,
        imap_port: u16,
        smtp_host: String,
        smtp_port: u16,
        use_tls: bool,
    },
}

// Email
pub struct Email {
    pub id: EmailId,
    pub account_id: AccountId,
    pub thread_id: ThreadId,
    pub message_id: MessageId,
    pub in_reply_to: Option<MessageId>,
    pub references: Vec<MessageId>,
    pub from: Address,
    pub to: Vec<Address>,
    pub cc: Vec<Address>,
    pub bcc: Vec<Address>,
    pub subject: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub snippet: String,
    pub date: DateTime<Utc>,
    pub is_read: bool,
    pub is_starred: bool,
    pub is_draft: bool,
    pub labels: Vec<LabelId>,
    pub attachments: Vec<Attachment>,
}

pub struct Address {
    pub email: String,
    pub name: Option<String>,
}

pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub is_inline: bool,
}

// Thread
pub struct Thread {
    pub id: ThreadId,
    pub account_id: AccountId,
    pub subject: Option<String>,
    pub snippet: String,
    pub participants: Vec<Address>,
    pub messages: Vec<Email>,
    pub last_message_date: DateTime<Utc>,
    pub unread_count: u32,
    pub is_starred: bool,
    pub labels: Vec<LabelId>,
}

pub struct ThreadSummary {
    pub id: ThreadId,
    pub account_id: AccountId,
    pub subject: Option<String>,
    pub snippet: String,
    pub from: Address,
    pub last_message_date: DateTime<Utc>,
    pub message_count: u32,
    pub unread_count: u32,
    pub is_starred: bool,
    pub labels: Vec<LabelId>,
}

// AI types
pub struct Summary {
    pub text: String,
    pub key_points: Vec<String>,
    pub action_items: Vec<String>,
}

pub struct DraftSuggestion {
    pub content: String,
    pub confidence: f32,
}

pub struct SenderAnalysis {
    pub likely_type: SenderType,
    pub reasoning: String,
    pub suggested_action: ScreenerAction,
}

pub enum SenderType {
    KnownContact,
    Newsletter,
    Marketing,
    Recruiter,
    Support,
    Unknown,
}

pub enum ScreenerAction {
    Approve,
    Reject,
    Review,
}
```

## Configuration

### Application Settings

```rust
pub struct Settings {
    pub appearance: AppearanceSettings,
    pub accounts: Vec<AccountSettings>,
    pub ai: AiSettings,
    pub notifications: NotificationSettings,
    pub sync: SyncSettings,
    pub keybindings: KeybindingSettings,
    pub privacy: PrivacySettings,
}

pub struct AppearanceSettings {
    pub theme: Theme,
    pub font_family: String,
    pub font_size: u8,
    pub density: Density,
    pub sidebar_width: u32,
    pub reading_pane_width: u32,
}

pub enum Theme {
    Dark,
    Light,
    System,
}

pub enum Density {
    Compact,
    Default,
    Relaxed,
}

pub struct AiSettings {
    pub enabled: bool,
    pub default_provider: String,
    pub providers: HashMap<String, ProviderSettings>,
    pub summary_settings: SummarySettings,
    pub compose_settings: ComposeSettings,
    pub search_settings: SearchSettings,
}

pub struct ProviderSettings {
    pub api_key_keychain_id: String,
    pub base_url: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<usize>,
}

pub struct SummarySettings {
    pub enabled: bool,
    pub provider: Option<String>,
    pub system_prompt: String,
    pub max_length: usize,
}

pub struct ComposeSettings {
    pub enabled: bool,
    pub provider: Option<String>,
    pub system_prompt: String,
    pub tone: Tone,
    pub learn_from_sent: bool,
}

pub enum Tone {
    Formal,
    Casual,
    Brief,
    Detailed,
    Custom(String),
}

pub struct NotificationSettings {
    pub enabled: bool,
    pub new_email: NewEmailNotification,
    pub snooze_reminders: bool,
    pub sound_enabled: bool,
    pub quiet_hours: Option<QuietHours>,
}

pub enum NewEmailNotification {
    All,
    VipOnly,
    None,
}

pub struct PrivacySettings {
    pub read_receipts_enabled: bool,
    pub external_content_enabled: bool,
    pub telemetry_retention_days: u32,
}
```

### Configuration File

Settings stored in `~/.config/heap/settings.json` (or XDG equivalent):

```json
{
  "appearance": {
    "theme": "dark",
    "font_family": "Inter",
    "font_size": 14,
    "density": "default"
  },
  "ai": {
    "enabled": true,
    "default_provider": "anthropic",
    "providers": {
      "anthropic": {
        "model": "claude-3-5-sonnet-20241022",
        "temperature": 0.7
      },
      "ollama": {
        "base_url": "http://localhost:11434",
        "model": "llama3.2"
      }
    },
    "summary_settings": {
      "enabled": true,
      "system_prompt": "Summarize this email thread concisely..."
    },
    "compose_settings": {
      "enabled": true,
      "tone": "casual",
      "system_prompt": "Draft a reply matching my communication style..."
    }
  },
  "notifications": {
    "enabled": true,
    "new_email": "vip_only",
    "sound_enabled": false
  },
  "privacy": {
    "read_receipts_enabled": false,
    "external_content_enabled": false,
    "telemetry_retention_days": 90
  }
}
```

## Directory Structure

```
heap/
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── Cargo.toml
├── Cargo.lock
├── LICENSE                    # AGPL-3.0
├── CHANGELOG.md
├── build.rs
├── docs/
│   └── sparc/
│       ├── Specification.md
│       ├── Architecture.md
│       ├── Pseudocode.md
│       ├── Refinement.md
│       └── Completion.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── app/
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   ├── actions.rs
│   │   ├── events.rs
│   │   └── keybindings.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── theme.rs
│   │   ├── components/
│   │   │   ├── mod.rs
│   │   │   ├── button.rs
│   │   │   ├── input.rs
│   │   │   ├── list.rs
│   │   │   └── ...
│   │   └── views/
│   │       ├── mod.rs
│   │       ├── main_window.rs
│   │       ├── sidebar.rs
│   │       ├── message_list.rs
│   │       ├── reading_pane.rs
│   │       ├── composer.rs
│   │       ├── command_palette.rs
│   │       ├── settings.rs
│   │       └── stats.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── email.rs
│   │   ├── thread.rs
│   │   ├── account.rs
│   │   ├── label.rs
│   │   ├── contact.rs
│   │   └── screener.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── email_service.rs
│   │   ├── thread_service.rs
│   │   ├── account_service.rs
│   │   ├── sync_service.rs
│   │   ├── ai_service.rs
│   │   ├── search_service.rs
│   │   ├── notification_service.rs
│   │   └── telemetry_service.rs
│   ├── providers/
│   │   ├── mod.rs
│   │   ├── email/
│   │   │   ├── mod.rs
│   │   │   ├── trait.rs
│   │   │   ├── gmail.rs
│   │   │   └── imap.rs
│   │   └── ai/
│   │       ├── mod.rs
│   │       ├── trait.rs
│   │       ├── openai.rs
│   │       ├── anthropic.rs
│   │       └── ollama.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── database.rs
│   │   ├── keychain.rs
│   │   ├── migrations/
│   │   │   ├── mod.rs
│   │   │   └── v001_initial.rs
│   │   └── queries/
│   │       ├── mod.rs
│   │       ├── emails.rs
│   │       ├── threads.rs
│   │       └── ...
│   ├── embedding/
│   │   ├── mod.rs
│   │   ├── engine.rs
│   │   ├── models.rs
│   │   └── vector_store.rs
│   └── config/
│       ├── mod.rs
│       └── settings.rs
├── tests/
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── email_service_test.rs
│   │   ├── sync_test.rs
│   │   └── ai_test.rs
│   └── ui/
│       ├── mod.rs
│       └── ...
├── assets/
│   ├── icons/
│   └── fonts/
└── scripts/
    ├── setup.sh
    └── build-release.sh
```

## Security Considerations

### Credential Storage

```rust
// All credentials stored via OS keychain
pub struct KeychainAccess {
    service_name: String,  // "com.panbanda.heap"
}

impl KeychainAccess {
    pub fn store(&self, key: &str, value: &str) -> Result<()> {
        keyring::Entry::new(&self.service_name, key)?
            .set_password(value)?;
        Ok(())
    }

    pub fn retrieve(&self, key: &str) -> Result<Option<String>> {
        match keyring::Entry::new(&self.service_name, key)?.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        keyring::Entry::new(&self.service_name, key)?.delete_password()?;
        Ok(())
    }
}
```

### Network Security

- All connections use TLS (rustls)
- Certificate validation enabled by default
- No HTTP fallback
- OAuth tokens refreshed proactively

### Data Security

- Database file permissions: 600 (owner read/write only)
- No sensitive data in logs
- Memory cleared after credential use
- API keys never serialized to disk (only keychain reference)

## Performance Considerations

### Message List Virtualization

```rust
// Only render visible items + buffer
pub struct VirtualizedList {
    items: Vec<ThreadSummary>,
    visible_range: Range<usize>,
    item_height: f32,
    buffer_count: usize,  // Items to render above/below viewport
}

impl VirtualizedList {
    fn visible_items(&self) -> impl Iterator<Item = &ThreadSummary> {
        let start = self.visible_range.start.saturating_sub(self.buffer_count);
        let end = (self.visible_range.end + self.buffer_count).min(self.items.len());
        self.items[start..end].iter()
    }
}
```

### Caching Strategy

```rust
// LRU cache for frequently accessed data
pub struct CacheLayer {
    threads: LruCache<ThreadId, Thread>,
    emails: LruCache<EmailId, Email>,
    summaries: LruCache<ThreadId, Summary>,
}

// Cache invalidation on mutations
impl CacheLayer {
    pub fn invalidate_thread(&mut self, thread_id: &ThreadId) {
        self.threads.pop(thread_id);
        self.summaries.pop(thread_id);
    }
}
```

### Async Operations

- All I/O operations async via tokio
- UI never blocks on network/disk
- Background sync doesn't impact responsiveness
- AI operations show loading state, cancellable

## Licensing

The Heap is licensed under the **Apache License 2.0**.

```
The Heap - A keyboard-driven, AI-augmented email client
Copyright 2025 Jonathan Reyes <me@jonathanreyes.com>

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

## Testing Strategy

### Coverage Requirement

**Minimum 95% test coverage enforced via CI.**

Coverage is measured using `cargo-llvm-cov` and reported on every PR.

### Unit Tests
- Domain logic (email parsing, threading)
- Provider implementations (mock servers)
- Storage queries
- AI prompt generation
- All public API surfaces

### Integration Tests
- Full sync cycle with test account
- Offline queue processing
- Search indexing and retrieval
- Database migrations

### UI Tests
- gpui TestAppContext for component testing
- Keyboard navigation flows
- Action dispatching
- Accessibility compliance

### End-to-End Tests
- Full user flows with real (test) accounts
- Performance benchmarks
- Cross-platform verification

### Test Commands

```bash
# Run all tests
cargo test

# Run with coverage
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Run specific test suite
cargo test --package heap-core
cargo test --package heap-ui

# Run benchmarks
cargo bench
```

## CI/CD Pipeline

### GitHub Actions Workflows

#### `.github/workflows/ci.yml` - Continuous Integration

Runs on every push and pull request.

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -Dwarnings

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --all-features --workspace

  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-features --workspace -- -D warnings

  test:
    name: Test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install dependencies (Linux)
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libxkbcommon-dev libwayland-dev
      - run: cargo test --all-features --workspace

  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-llvm-cov
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libxkbcommon-dev libwayland-dev
      - name: Generate coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
      - name: Check coverage threshold
        run: |
          COVERAGE=$(cargo llvm-cov --all-features --workspace --json | jq '.data[0].totals.lines.percent')
          echo "Coverage: $COVERAGE%"
          if (( $(echo "$COVERAGE < 95" | bc -l) )); then
            echo "Coverage $COVERAGE% is below 95% threshold"
            exit 1
          fi
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: true

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@cargo-audit
      - run: cargo audit
```

#### `.github/workflows/release.yml` - Release Automation

Triggered on version tags (e.g., `v0.1.0`).

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
      version: ${{ steps.get_version.outputs.version }}
    steps:
      - uses: actions/checkout@v4
      - name: Get version
        id: get_version
        run: echo "version=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT
      - name: Create Release
        id: create_release
        uses: softprops/action-gh-release@v1
        with:
          draft: true
          generate_release_notes: true

  build-macos:
    name: Build macOS
    needs: create-release
    runs-on: macos-latest
    strategy:
      matrix:
        target: [x86_64-apple-darwin, aarch64-apple-darwin]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      - name: Create DMG
        run: |
          mkdir -p dist
          # Bundle application
          cargo install cargo-bundle
          cargo bundle --release --target ${{ matrix.target }}
          # Create DMG
          hdiutil create -volname "The Heap" -srcfolder "target/${{ matrix.target }}/release/bundle/osx/The Heap.app" -ov -format UDZO dist/heap-${{ needs.create-release.outputs.version }}-${{ matrix.target }}.dmg
      - name: Upload macOS artifact
        uses: softprops/action-gh-release@v1
        with:
          files: dist/heap-${{ needs.create-release.outputs.version }}-${{ matrix.target }}.dmg

  build-linux:
    name: Build Linux
    needs: create-release
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [x86_64-unknown-linux-gnu]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libxkbcommon-dev libwayland-dev
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      - name: Create AppImage
        run: |
          mkdir -p dist
          # Install appimagetool
          wget -O appimagetool https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
          chmod +x appimagetool
          # Create AppDir structure
          mkdir -p AppDir/usr/bin AppDir/usr/share/applications AppDir/usr/share/icons/hicolor/256x256/apps
          cp target/${{ matrix.target }}/release/heap AppDir/usr/bin/
          cp assets/icons/heap.png AppDir/usr/share/icons/hicolor/256x256/apps/
          cp assets/heap.desktop AppDir/usr/share/applications/
          ln -s usr/bin/heap AppDir/AppRun
          ln -s usr/share/icons/hicolor/256x256/apps/heap.png AppDir/heap.png
          # Build AppImage
          ./appimagetool AppDir dist/heap-${{ needs.create-release.outputs.version }}-${{ matrix.target }}.AppImage
      - name: Upload Linux artifact
        uses: softprops/action-gh-release@v1
        with:
          files: dist/heap-${{ needs.create-release.outputs.version }}-${{ matrix.target }}.AppImage

  publish-release:
    name: Publish Release
    needs: [create-release, build-macos, build-linux]
    runs-on: ubuntu-latest
    steps:
      - name: Publish release
        uses: softprops/action-gh-release@v1
        with:
          draft: false
```

### Release Process

1. **Version bump**: Update `Cargo.toml` version
2. **Changelog**: Update CHANGELOG.md
3. **Tag**: `git tag v0.1.0 && git push origin v0.1.0`
4. **Automated**: GitHub Actions builds and creates draft release
5. **Review**: Verify artifacts, edit release notes
6. **Publish**: Mark release as non-draft

### Versioning

Follows [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

Pre-1.0: Minor version bumps may include breaking changes.

## Deployment

### Build Process

```bash
# Development
cargo build

# Release (optimized)
cargo build --release

# macOS app bundle
cargo bundle --release

# Run tests with coverage
cargo llvm-cov --all-features --workspace
```

### Distribution

- **Direct download**: .dmg (macOS), .AppImage (Linux) from GitHub Releases
- **Homebrew** (macOS): `brew install --cask heap`
- **AUR** (Arch Linux): `yay -S heap`
- **Flatpak** (Linux): `flatpak install heap`

### Updates

- Manual update check in-app
- Optional auto-update (user preference)
- Delta updates where possible
- Release notifications via GitHub

## Future Considerations

Items explicitly out of scope for v1 but architecturally considered:

1. **Plugin System**: Service traits designed for future plugin loading
2. **Mobile**: Data models portable, UI would need reimplementation
3. **Team Features**: Account model supports shared accounts conceptually
4. **E2E Encryption**: Message model has space for encryption metadata
5. **JMAP**: Provider trait can accommodate new protocols
6. **API Server**: Core services could expose HTTP API

## References

- [gpui Documentation](https://github.com/zed-industries/zed/tree/main/crates/gpui)
- [Candle Examples](https://github.com/huggingface/candle/tree/main/candle-examples)
- [Gmail API Reference](https://developers.google.com/gmail/api/reference/rest)
- [IMAP RFC 3501](https://tools.ietf.org/html/rfc3501)
- [SQLite Best Practices](https://www.sqlite.org/np1queryprob.html)

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1.0 | 2025-01-11 | - | Initial architecture |
