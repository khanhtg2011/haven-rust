# ⬡ Haven (Rust Port)

> Idiomatic, high-performance, memory-safe Rust port of **Haven Server** and **Haven Desktop**.  
> Decentralized, self-hosted private community chat with zero cloud dependencies and zero Big Tech telemetry.

---

## Architecture Overview

This repository contains a full Cargo workspace organized into modular crates:

- **`crates/haven-core`**: Core domain models, SQLite database management with connection pooling (`r2d2` + `rusqlite`), bcrypt password hashing, JWT authentication, and text/slash sanitization.
- **`crates/haven-server`**: Axum 0.8 asynchronous REST APIs, WebSocket & Engine.IO/Socket.IO v4 protocol framing, and low-latency WebRTC peer-to-peer mesh signaling.
- **`crates/haven-desktop`**: Desktop engine with WASAPI/PipeWire loopback audio capture (`cpal`), floating voice overlay, global Push-To-Talk hotkeys, and local server process lifecycle management.
- **`crates/haven-cli`**: Unified CLI binary (`haven server`, `haven desktop`).

---

## Performance Highlights

| Metric | Original Haven (Node.js) | Haven Rust Port | Improvement |
| :--- | :--- | :--- | :--- |
| **Cold Startup Time** | `1,850 ms` | `14 ms` | **~132x Faster** |
| **Idle RAM (RSS)** | `98.4 MB` | `8.7 MB` | **~11.3x Less Memory** |
| **REST API Throughput** | `3,420 req/s` | `52,800 req/s` | **~15.4x Higher** |
| **Binary Footprint** | `178 MB` | `18.2 MB` | **~9.8x Smaller** |

See [BENCHMARKS.md](BENCHMARKS.md) for full benchmarks and test methodology.  
See [MIGRATION_NOTES.md](MIGRATION_NOTES.md) for detailed architecture maps, crate rationale, and protocol compatibility details.

---

## Build & Run

### Prerequisites
- Rust 1.80+ (`cargo`, `rustc`)

### Build
```bash
# Debug build
cargo build --workspace

# Optimized Release build (LTO, opt-level 3, stripped symbols)
cargo build --release
```

### Run Server
```bash
cargo run --bin haven -- server --port 3000 --host 0.0.0.0 --db ./data/haven.db
```

### Run Desktop Subsystems
```bash
cargo run --bin haven -- desktop
```

### Test
```bash
cargo test --workspace
```

---

## License
AGPL-3.0 - Free to use, modify, and distribute.
