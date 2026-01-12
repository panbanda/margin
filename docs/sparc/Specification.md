# The Heap - Specification Document

> SPARC Phase 1: Specification
> Version: 0.1.0
> Last Updated: 2025-01-11

## Executive Summary

The Heap is a desktop email client for technical users, built in Rust with gpui. It prioritizes keyboard-driven workflows, local-first architecture, AI augmentation with full user control, and privacy by default.

The name reflects the product philosophy: emails pile up like a heap, and this client helps you efficiently work through them.

## Problem Statement

Existing email clients fall into two categories:

1. **Traditional clients** (Thunderbird, Apple Mail): Feature-complete but dated UX, poor keyboard support, no AI capabilities
2. **Modern SaaS clients** (Superhuman, Shortwave, Hey): Excellent UX but cloud-dependent, expensive subscriptions, privacy concerns, vendor lock-in

Technical users need:
- Speed and keyboard efficiency
- AI assistance without sending data to third parties
- Full control over their data and infrastructure
- Customization without compromise

## Target Users

### Primary: Technical Professionals
- Software engineers, DevOps, SREs, system administrators
- Comfortable with keyboard shortcuts and configuration
- Privacy-conscious, prefer local-first tools
- Already use tools like Neovim, Zed, Linear, or terminal-based workflows
- Manage multiple email accounts (work, personal, side projects)

### Secondary: Power Users
- Productivity enthusiasts who value keyboard efficiency
- Users leaving Superhuman/Hey due to cost or privacy concerns
- Self-hosters and open-source advocates

## Core Principles

### 1. Privacy First
- All data stored locally by default
- AI processing uses user-provided API keys or local models
- No telemetry leaves the device
- Read receipts and tracking disabled by default

### 2. Keyboard Driven
- Every action accessible via keyboard
- Superhuman-compatible shortcuts as defaults
- Fully customizable keybindings
- Command palette for discoverability

### 3. Local First
- Works offline with full functionality
- User controls sync behavior
- Bring your own database option
- No vendor lock-in

### 4. AI as Utility
- Augments user capability, never replaces judgment
- Customizable prompts and behavior
- Transparent token usage and costs
- Optional - works fully without AI

### 5. Value Over Flash
- Functional before aesthetic
- Performance over animation
- Density over whitespace (configurable)
- Substance over novelty

## Functional Requirements

### FR-1: Email Account Management

#### FR-1.1: Multi-Account Support
- Support unlimited email accounts
- Account types: Gmail (API), IMAP/SMTP (generic)
- Per-account configuration for sync, notifications, signature
- Unified inbox view across all accounts
- Per-account views available

#### FR-1.2: Authentication
- Gmail: OAuth 2.0 flow with local token storage
- IMAP/SMTP: Username/password with secure credential storage
- Support for app-specific passwords
- Re-authentication prompts when tokens expire

#### FR-1.3: Account Switching
- Sidebar account list with unread counts
- Keyboard shortcuts: `Ctrl+1`, `Ctrl+2`, etc.
- Visual indicator of active account context

### FR-2: Email Operations

#### FR-2.1: Reading
- Three-pane layout (sidebar, list, reading pane)
- Gmail-style threaded conversations
- Expand/collapse individual messages (`O` / `Shift+O`)
- Inline image rendering
- Attachment preview and download
- HTML email rendering with plain-text fallback
- External content blocking by default

#### FR-2.2: Composing
- Inline reply composer (default)
- Pop-out composer window
- Full-screen distraction-free mode
- Rich text editor with Slack-style toolbar
- Markdown storage, HTML rendering for send
- Draft auto-save (local)
- Multiple drafts support

#### FR-2.3: Organization
- Archive (`E`)
- Delete/Trash (`#` or `Shift+3`)
- Star/Flag (`S`)
- Labels/Folders with keyboard application (`L`)
- Move to label (`V`)
- Mark read/unread (`U`)
- Snooze with custom time (`H`)

#### FR-2.4: Search
- Full-text search across all accounts
- Gmail-style operators (from:, to:, subject:, has:attachment, etc.)
- Semantic/natural language search (AI-powered)
- Search within current view/label
- Saved searches as Smart Views

### FR-3: Navigation & Keyboard

#### FR-3.1: Core Navigation
| Action | Default Shortcut |
|--------|------------------|
| Command palette | `Cmd+K` |
| Search | `/` |
| Next message | `J` |
| Previous message | `K` |
| Open/Enter | `Enter` |
| Back | `Esc` |
| Select | `X` |
| Select range | `Shift+J/K` |

#### FR-3.2: Go-To Navigation
| Action | Default Shortcut |
|--------|------------------|
| Go to Inbox | `G I` |
| Go to Starred | `G S` |
| Go to Sent | `G T` |
| Go to Drafts | `G D` |
| Go to Archive | `G E` |
| Go to Label | `G L` |
| Go to Stats | `G A` |

#### FR-3.3: Actions
| Action | Default Shortcut |
|--------|------------------|
| Compose | `C` |
| Reply | `R` |
| Reply All | `Enter` (on message) |
| Forward | `F` |
| Archive | `E` |
| Trash | `#` |
| Star | `S` |
| Snooze | `H` |
| Mark read/unread | `U` |
| Undo | `Z` |
| Send | `Cmd+Enter` |

#### FR-3.4: Customization
- Full keybinding customization in settings
- Conflict detection and resolution
- Import/export keybinding configurations
- Vim mode option (modal editing in composer)

### FR-4: AI Features

#### FR-4.1: Provider Configuration
- OpenAI API (direct)
- Anthropic API (direct)
- OpenAI-compatible endpoints (Ollama, llama.cpp, vLLM, LM Studio)
- Per-feature model selection (e.g., fast model for summaries, capable model for compose)
- API key secure storage

#### FR-4.2: Email Summaries
- Thread summary at top of conversation
- Inbox digest summary
- Configurable summary length and style
- On-demand generation (not automatic by default)

#### FR-4.3: Smart Compose
- AI-assisted reply drafting
- Style learning from user's sent emails (local analysis)
- Tone adjustment (formal, casual, brief, detailed)
- Custom system prompts for voice/style
- Accept, edit, or reject suggestions

#### FR-4.4: Semantic Search
- Natural language queries ("find the email about the deployment last week")
- Local embedding generation (Candle + BERT/similar)
- Optional: hosted embedding providers (OpenAI, Pinecone)
- Bring your own embedding model support
- Hybrid: keyword + semantic ranking

#### FR-4.5: Categorization & Triage
- Auto-categorization suggestions (newsletters, receipts, etc.)
- Priority scoring
- "Needs reply" detection
- "Waiting for response" tracking
- Screener for first-time senders with AI suggestions

#### FR-4.6: Customization
- Editable system prompts for all AI features
- Temperature and parameter controls
- Per-feature enable/disable
- Prompt templates library

### FR-5: Screener (New Sender Triage)

#### FR-5.1: Behavior
- First-time senders quarantined to Screener queue
- AI provides context: sender analysis, suggested action, reasoning
- Single-keystroke approve/reject
- Silent rejection (sender not notified)
- Approved senders go to appropriate inbox

#### FR-5.2: Rules
- Domain allowlists (always approve from *@mycompany.com)
- Pattern-based rules
- Contact list integration (known contacts auto-approved)

### FR-6: Smart Views

#### FR-6.1: Built-in Views
- Inbox (unified and per-account)
- Needs Reply (detected by AI)
- Waiting For (sent emails awaiting response)
- Newsletters (auto-categorized)
- Receipts & Transactions
- Screener (new senders)

#### FR-6.2: Custom Views
- User-defined filter criteria
- Saved search as view
- AI-generated view suggestions

### FR-7: Sync & Offline

#### FR-7.1: Sync Behavior
- User-configurable sync intervals (1 min to manual)
- Per-account sync settings
- Bandwidth-conscious options (headers only, full sync, etc.)
- Background sync with status indicator

#### FR-7.2: Offline Capability
- Full read access to synced emails
- Compose and queue for send
- Search within synced content
- Actions queued and synced when online

#### FR-7.3: Conflict Resolution
- Server wins by default for external changes
- Local drafts preserved
- Conflict notification for edge cases

### FR-8: Usage Statistics

#### FR-8.1: Email Metrics
- Received/sent/archived counts (daily/weekly/monthly)
- By account breakdown
- Top correspondents
- Busiest hours/days

#### FR-8.2: Productivity Metrics
- Average response time
- Time to inbox zero
- Sessions and time in app
- Emails processed per session

#### FR-8.3: AI Metrics
- Summaries generated
- Compose assists used/accepted
- Semantic searches
- Token consumption by provider
- Estimated cost tracking

#### FR-8.4: Dashboard
- Accessible via `G A` or sidebar
- Configurable time ranges
- Export to JSON/CSV
- All data local, user can purge at will

### FR-9: Notifications

#### FR-9.1: Types
- New email (configurable: all, VIP only, none)
- Snooze reminders
- Sync errors
- AI completion (status bar)

#### FR-9.2: Delivery
- System native notifications (optional)
- In-app notifications (badges, toasts)
- Sound (optional, off by default)
- Quiet hours configuration

### FR-10: Settings & Configuration

#### FR-10.1: Appearance
- Dark mode (default)
- Light mode
- System preference follow
- Accent color customization
- Font selection (Inter default, configurable)
- Font size adjustment (`Cmd+`/`Cmd-`)
- Density: Compact, Default, Relaxed

#### FR-10.2: Accessibility
- Screen reader support (ARIA)
- High contrast mode
- Reduced motion option
- Keyboard-only navigation
- Focus indicators

#### FR-10.3: Data Management
- Database location configuration
- Backup/restore
- Export all data
- Import from other clients
- Cache management

## Non-Functional Requirements

### NFR-1: Performance
- App launch: < 500ms to interactive
- Message list render: < 100ms for 100 items
- Search results: < 500ms for local, < 2s for semantic
- Compose load: < 50ms
- Memory: < 500MB baseline, < 1GB with large mailboxes
- Smooth 60fps scrolling

### NFR-2: Security
- Credentials encrypted at rest (OS keychain integration)
- API keys never logged or transmitted
- No external network calls except to configured email/AI providers
- Regular dependency audits
- Memory-safe (Rust)

### NFR-3: Reliability
- Graceful degradation when offline
- No data loss on crash (auto-save drafts)
- Automatic recovery from sync failures
- Comprehensive error logging (local only)

### NFR-4: Maintainability
- Modular architecture
- **95% minimum test coverage** (enforced via CI)
- Clear separation of concerns
- Documentation for contributors

### NFR-5: Quality Assurance
- Continuous Integration via GitHub Actions
- All PRs require passing CI (build, lint, test, coverage)
- Automated release pipeline
- Code coverage reports on every PR
- Security audit checks (cargo-audit)

### NFR-6: Platform Support
- macOS (primary, gpui native)
- Linux (secondary, gpui supported)
- Windows: Not supported in v1 (gpui limitation)

## User Scenarios

### US-1: Morning Email Triage

**Actor**: Software engineer starting workday
**Goal**: Process overnight emails efficiently

**Flow**:
1. Launch The Heap, sees unified inbox with unread count
2. Press `G I` to ensure in inbox
3. Sees AI-generated inbox digest summary
4. Uses `J`/`K` to navigate, `O` to expand threads
5. For each email: `E` to archive, `R` to reply, `H` to snooze
6. Checks Screener (`G C`), approves/rejects new senders
7. Reaches inbox zero in 15 minutes

### US-2: Composing a Complex Reply

**Actor**: User replying to a technical thread
**Goal**: Write a detailed response with references

**Flow**:
1. Opens thread, reads context
2. Presses `R` for inline reply
3. Clicks AI button, selects "Draft reply"
4. Reviews AI suggestion, edits for accuracy
5. Uses toolbar to add code block and link
6. Presses `Cmd+Enter` to send

### US-3: Finding an Old Email

**Actor**: User looking for a specific conversation
**Goal**: Locate email from months ago

**Flow**:
1. Presses `/` to open search
2. Types "deployment issue march" (natural language)
3. Semantic search surfaces relevant threads
4. Refines with `from:alice` operator if needed
5. Opens result, finds needed information

### US-4: Setting Up New Account

**Actor**: New user adding work email
**Goal**: Configure Gmail account with work settings

**Flow**:
1. Opens Settings > Accounts > Add Account
2. Selects "Gmail"
3. Completes OAuth flow in browser
4. Configures sync preferences (last 30 days)
5. Sets work signature
6. Assigns keyboard shortcut `Ctrl+1`
7. Account syncs in background

### US-5: Offline Usage

**Actor**: User on airplane without connectivity
**Goal**: Process emails and draft responses

**Flow**:
1. Opens The Heap, sees "Offline" indicator in status bar
2. Reads fully synced emails normally
3. Archives emails, changes queued
4. Composes reply, saved as draft
5. Lands, connects to wifi
6. The Heap syncs automatically, sends queued items

### US-6: Reviewing Usage Statistics

**Actor**: User curious about email habits
**Goal**: Understand time spent on email

**Flow**:
1. Presses `G A` to open stats dashboard
2. Sees weekly trend: 4.2 hours in email
3. Notes busiest time is 9-10 AM
4. Checks AI usage: 12k tokens, ~$0.15 estimated
5. Exports data for personal tracking

## Success Metrics

### Adoption
- User can reach inbox zero 2x faster than previous client
- 90% of actions completed via keyboard
- Setup time < 10 minutes for first account

### Performance
- 95th percentile app launch < 500ms
- Zero data loss incidents
- < 1% crash rate

### Satisfaction
- Users report high control over email workflow
- AI features rated as "helpful, not intrusive"
- Privacy-conscious users feel comfortable

## Out of Scope (v1)

- Windows support (gpui limitation)
- Mobile applications
- Plugin/extension system
- Calendar application (AI awareness only)
- Team/collaboration features
- End-to-end encryption (PGP/S/MIME)
- Microsoft Graph API (Outlook/O365)
- JMAP protocol support
- Public API for external integrations

## Licensing

The Heap is licensed under the **Apache License 2.0**.

This ensures:
- Permissive license allowing commercial use
- Patent grant protection
- Clear attribution requirements
- Freedom to use, modify, and distribute

## Open Questions

1. **Distribution**: Direct download, Homebrew, package managers?
2. **Embedding model default**: Which model offers best size/quality tradeoff for local use?
3. **Signature management**: Per-account, per-recipient rules, or simple?

## References

- [Superhuman Keyboard Shortcuts](https://quickref.me/superhuman.html)
- [Hey Email Features](https://www.hey.com/features/)
- [Shortwave AI Features](https://www.shortwave.com/)
- [gpui Framework](https://github.com/zed-industries/zed/tree/main/crates/gpui)
- [Candle ML Framework](https://github.com/huggingface/candle)
- [Linear Design Philosophy](https://linear.app/now/how-we-redesigned-the-linear-ui)
- [Command Palette UX](https://blog.superhuman.com/how-to-build-a-remarkable-command-palette/)

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1.0 | 2025-01-11 | - | Initial specification |
