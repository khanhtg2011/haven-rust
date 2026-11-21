# Haven Migration & Architecture Notes (Node.js/Electron -> Idiomatic Rust)

## 1. What Haven Does
Haven is a self-hosted, private community communication platform (a decentralized, zero-telemetry alternative to Discord).
It encompasses:
- **Haven Server**:
  - Full text chat with channels, threads, replies, reactions, pins, mentions, search, and Markdown/code formatting.
  - End-to-end encrypted direct messaging (E2EE DMs) with client-side key generation and ratcheted key storage.
  - Multi-party WebRTC voice & video calls, screen sharing, and soundboard, with peer-to-peer mesh signaling and an optional server-side voice relay.
  - Rich moderation and administration: role-based access control (RBAC), channel permissions, IP/user bans, timed mutes, automod link inspection, audit logs, and scheduled backups.
  - Dynamic customization: plugins, themes, custom emoji/stickers, and server branding.
  - Third-party integrations: Bot API / webhooks with audio streaming into voice channels, OIDC SSO, push notifications (FCM & UnifiedPush), and Ferry (two-way Discord bridge).
  - Storage: SQLite via `better-sqlite3` storing 76 relational tables with schema migrations.
- **Haven Desktop**:
  - Native desktop client (originally Electron) for Windows, Linux, and macOS.
  - Per-app audio loopback capture (WASAPI on Windows, PulseAudio/PipeWire on Linux) for crystal-clear screen sharing audio.
  - Low-latency floating voice overlay displaying active speakers, talking indicators, mute/deafen status.
  - Global push-to-talk (PTT) and push-to-mute (PTM) keybindings via low-level keyboard/mouse hooks.
  - Native system tray, cross-platform notifications, auto-updates, and integrated local server lifecycle manager (`server:start`, `server:stop`).

---

## 2. Architecture and Module Map

### 2.1 Haven Server
Original Node.js layout -> Rust Workspace Member (`crates/haven-server` & `crates/haven-core`):

| Node.js Source Module | Description | Rust Module Target (`haven-server` / `haven-core`) |
| :--- | :--- | :--- |
| `server.js` | Entry point, HTTP/HTTPS listener, middleware, Socket.IO binding | `haven-server::main`, `haven-server::server`, `haven-server::app` |
| `src/database.js`, `src/db/` | SQLite database connection, PRAGMA setup, 76 schema tables & migrations | `haven-core::db`, `haven-core::models` (`rusqlite` / `sqlx-sqlite`) |
| `src/auth.js` | JWT token validation, bcrypt password hashing, TOTP MFA, guest tokens | `haven-core::auth`, `haven-core::crypto` (`jsonwebtoken`, `argon2`/`bcrypt`, `totp-rs`) |
| `src/socketHandlers/*.js` | Realtime Socket.IO events (506 events: chat, voice mesh, presence, E2EE) | `haven-server::ws::handlers`, `haven-server::signaling` |
| `src/routes/*.js` | REST APIs (178 routes: `/api/auth`, `/api/channels`, `/api/messages`, etc.) | `haven-server::routes::*` (`axum` handlers & routers) |
| `src/botVoice.js`, `botAudio.js` | Bot voice channel access, audio transcoding & PCM/Opus streaming | `haven-server::bots::voice`, `haven-server::bots::audio` |
| `src/automod.js`, `blockedNames.js` | Text filtering, fuzzy regex pattern checks, username sanitization | `haven-core::automod`, `haven-core::sanitizer` |
| `src/voiceRelay/` | Optional server-side WebRTC voice router (Mediasoup / RTP forwarder) | `haven-server::voice_relay` (`webrtc-rs` / RTP router) |
| `src/activity.js` | Member presence, rich presence (Steam, Spotify, Last.fm) | `haven-server::presence` |
| `src/tunnel.js` | Cloudflare / LocalTunnel remote tunnel management | `haven-server::tunnel` |

### 2.2 Haven Desktop
Original Electron layout -> Rust Workspace Member (`crates/haven-desktop`):

| Electron Source Module | Description | Rust Target (`crates/haven-desktop`) |
| :--- | :--- | :--- |
| `src/main/main.js` | Application lifecycle, window creation, single-instance lock | `haven-desktop::app`, `haven-desktop::window` (`tao` / `wry` or native GUI) |
| `src/main/audio-capture.js` | WASAPI loopback audio capture for screen sharing audio | `haven-desktop::audio_capture` (`cpal` / `windows-rs` WASAPI) |
| `src/main/voice-overlay.js` | Transparent, click-through floating voice overlay | `haven-desktop::overlay` |
| `src/main/screen-source.js` | Display & window capture enumeration | `haven-desktop::screen_capture` |
| `src/main/media-keys.js`, `ptt*.js` | Global hotkey hooks for Push-to-Talk | `haven-desktop::hotkeys` (`rdev` or OS hooks) |
| `src/main/server-manager.js` | Starting, stopping, and monitoring the embedded Haven server | `haven-desktop::server_process` |
| `src/main/ipc-policy.js` | IPC message validation & URL security boundaries | `haven-desktop::ipc` |

---

## 3. Public API and CLI Surface

### 3.1 REST API Surface (178 Routes)
- **Health & Info**: `GET /api/version`, `GET /api/public-config`, `GET /api/port-check`, `GET /api/ice-servers`
- **Auth**: `POST /api/auth/register`, `POST /api/auth/login`, `POST /api/auth/guest`, `POST /api/auth/logout`, `POST /api/auth/mfa/verify`
- **Channels**: `GET /api/channels`, `POST /api/channels`, `DELETE /api/channels/:id`, `PATCH /api/channels/:id`
- **Messages**: `GET /api/channels/:id/messages`, `POST /api/channels/:id/messages`, `DELETE /api/messages/:id`, `POST /api/messages/:id/reactions`
- **Bot Audio / Webhooks**: `GET /api/bot/voice-channels`, `POST /api/bot/voice-play`, `POST /api/webhooks`, `POST /api/webhooks/:token`
- **Admin**: `GET /api/admin/settings`, `POST /api/admin/settings`, `GET /api/admin/audit-log`, `POST /api/admin/backup`, `POST /api/admin/restore`

### 3.2 Realtime WebRTC & Socket Events (506 Events)
- **Signaling**: `voice-join`, `voice-leave`, `voice-offer`, `voice-answer`, `voice-ice-candidate`, `voice-speaking`
- **Chat**: `send-message`, `edit-message`, `delete-message`, `add-reaction`, `user-typing`
- **State**: `initial-state`, `channel-joined`, `user-profile`, `member-list`, `presence-update`

### 3.3 Haven CLI Surface
- `haven server --config <PATH> --port <PORT> --data-dir <DIR>`
- `haven backup create / restore`
- `haven user create-admin / reset-password`

---

## 4. Dependencies & Crate Selection (Phase 2)

| Crate | Version | Reason for Choice |
| :--- | :--- | :--- |
| `tokio` | `1.x` | Industry-standard asynchronous runtime with multithreaded work-stealing scheduler. |
| `axum` | `0.8.x` | High-performance, ergonomic async web framework built directly on `tokio`, `tower`, and `hyper`. |
| `tower` & `tower-http` | `0.6.x` | Production-grade middleware for CORS, tracing, compression, rate-limiting, and static file serving. |
| `serde` & `serde_json` | `1.0.x` | Zero-copy and efficient serialization/deserialization for REST APIs and WebSocket payloads. |
| `rusqlite` / `sqlx` | `0.32.x` | Robust SQLite driver with prepared statement caching and WAL support matching `better-sqlite3`. |
| `tokio-tungstenite` | `0.26.x` | High-throughput, low-overhead WebSocket transport for realtime chat and WebRTC signaling. |
| `webrtc` | `0.12.x` | Pure Rust WebRTC implementation for voice mesh bridging, RTP forwarding, and bot voice streams. |
| `jsonwebtoken` | `9.x` | Fast, cryptographic JWT authentication token generation and verification. |
| `argon2` / `bcrypt` | `0.5.x` | Secure password hashing compatible with existing Haven bcrypt hashes. |
| `totp-rs` | `5.6.x` | RFC 6238 compliant Two-Factor Authentication (MFA) validation. |
| `clap` | `4.5.x` | Declarative, high-performance CLI argument parsing with derive macros. |
| `anyhow` & `thiserror` | `1.0.x` | Idiomatic error handling separating domain errors (`thiserror`) from application errors (`anyhow`). |
| `tracing` & `tracing-subscriber` | `0.3.x` | Structured asynchronous logging, diagnostics, and span tracing. |
| `cpal` | `0.15.x` | Cross-platform audio I/O library for desktop WASAPI/PulseAudio audio capture. |
| `rdev` / OS hooks | `0.5.x` | Low-latency global keyboard and mouse event listening for Push-To-Talk. |
| `rayon` | `1.10.x` | Data-parallel processing for CPU-intensive tasks like image transcoding and fuzzy string matching. |
| `ahash` | `0.8.x` | Fast, non-cryptographic hasher for high-throughput in-memory routing and hash sets. |

---

## 5. Side Effects & Resource Boundaries
- **File I/O**:
  - SQLite database in `data/haven.db` (WAL mode enabled, thread-safe connection pooling).
  - Media & Attachment storage in `data/uploads/` with size guard limits and MIME verification.
  - Server configuration in `data/settings.json` or `.env`.
- **Network**:
  - Dual-protocol web server: HTTP/1.1 and HTTP/2 with optional TLS termination.
  - WebSocket connection multiplexing over `/socket.io/` or `/ws`.
  - WebRTC ICE/STUN/TURN UDP/TCP bindings for voice and video media.
- **Concurrency & Threads**:
  - Asynchronous event loop managed by Tokio.
  - Dedicated background worker threads for heavy transcoding and database sync operations.

---

## 6. Hard-to-Port Parts & Mitigations
1. **Socket.IO Protocol Parity**:
   - *Challenge*: Haven web client uses `socket.io-client@4.8.1` which relies on Engine.IO packet framing (`42["event", {...}]`).
   - *Mitigation*: Implement an Engine.IO v4 / Socket.IO v4 framing layer over `tokio-tungstenite` so the existing Haven web UI, mobile apps, and bots connect seamlessly without client modifications.
2. **Audio Loopback Capture on Windows & Linux**:
   - *Challenge*: Electron desktop used a C++ node-gyp addon for Windows WASAPI loopback and Linux PipeWire stream routing.
   - *Mitigation*: Implement native WASAPI loopback in Rust via `cpal` / `windows` crate, and PipeWire stream capture on Linux.
3. **Database Schema Parity**:
   - *Challenge*: 76 tables with dynamic triggers and foreign keys.
   - *Mitigation*: Maintain identical SQL schema migrations and column definitions so existing SQLite databases can be read directly by the Rust server.

---

## 7. Migration Order (Leaf Modules to Entry Point)
1. **Layer 1: Leaf Domain Models & Utilities (`haven-core`)**:
   - `types`, `error`, `config`, `utils` (slash parsing, blocked names sanitization, emoji parsing).
2. **Layer 2: Database Layer (`haven-core::db`)**:
   - Schema creation, migrations, connection pool, CRUD repositories for users, channels, messages, roles, audit logs, webhooks.
3. **Layer 3: Authentication & Security (`haven-core::auth`)**:
   - Password verification, JWT claims, TOTP MFA, permission bits, role gate evaluation.
4. **Layer 4: Realtime Signaling & WebSockets (`haven-server::ws`)**:
   - Socket framing, session connection handshake, room pub-sub, WebRTC mesh signaling.
5. **Layer 5: REST API Services (`haven-server::routes`)**:
   - Axum routers for all public, user, channel, admin, and bot endpoints.
6. **Layer 6: Bot Voice & Media Engine (`haven-server::bots`)**:
   - Voice channel join/leave, PCM/Opus packetization, audio streaming.
7. **Layer 7: Desktop Client Engine (`haven-desktop`)**:
   - Hotkeys (PTT), WASAPI/PipeWire audio capture, overlay window, server process manager.
8. **Layer 8: Entry Points & CLI (`haven-server::main`, `haven-desktop::main`)**:
   - CLI flags, configuration loading, graceful shutdown on SIGINT/SIGTERM.
