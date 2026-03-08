# HKB (Holy Kern Base) — Codemap

A personal productivity tool written in Rust, structured as a Cargo workspace with 5 crates.

---

## Workspace Layout

```
hkb/
├── Cargo.toml              # Workspace root (5 members)
├── hkb_client/             # TUI application (binary)
├── hkb_daemon/             # Background daemon (binary)
├── hkb_daemon_core/        # IPC transport layer (library)
├── hkb_core/               # Shared core library
└── hkb_date/               # Date/time parsing library
```

**Dependency graph:**

```
hkb_client ──┐
hkb_daemon ──┼──► hkb_core
             ├──► hkb_date
             └──► hkb_daemon_core ──► hkb_core
```

---

## `hkb_client` — TUI Application

Entry point: `src/main.rs`. A terminal UI app using **ratatui** + **crossterm** running at ~60 FPS.

| Module | Purpose |
|---|---|
| `main.rs` | Bootstrap (DB init, daemon connection, event loop) |
| `app_state.rs` | Global mutable state: current view, edit mode |
| `apps/main.rs` | Main app screen |
| `apps/reminders.rs` | Reminders screen (router) |
| `apps/reminders/reminders_create.rs` | Create reminder form |
| `apps/reminders/reminders_list.rs` | List reminders view |
| `components/button.rs` | Button widget |
| `components/input.rs` | Input widget |
| `components/image.rs` | Image rendering widget |
| `components/navigation.rs` | Top navigation bar |
| `events.rs` | Global crossterm event queue |
| `focus.rs` | Focus management |
| `singleton.rs` | Global `mpsc` sender to daemon |
| `terminal.rs` | Terminal setup/teardown |
| `utils/bounded_value.rs` | Numeric value with min/max bounds |

**Views:** `AppView::Main` and `AppView::Reminders`.

---

## `hkb_daemon` — Background Service

Entry point: `src/main.rs`. A long-running **tokio** async process.

| Responsibility | Detail |
|---|---|
| Unix socket server | Listens on `~/.local/share/hkb/hkb.sock` |
| Reminder notifications | Polls DB every 10s; fires OS notifications (via `notify-rust`) at 1 min, 5 min, 15 min, 30 min intervals |
| Reminder cleanup | Deletes reminders older than 1 day every 5 minutes |
| Audio | Plays `notification.wav` via **rodio** on reminder trigger |

`src/audio.rs` — audio playback module.

---

## `hkb_daemon_core` — IPC Transport

Shared by both client and daemon. Communication over a Unix domain socket.

| File | Purpose |
|---|---|
| `server.rs` | `Server` — binds/accepts on the Unix socket |
| `client.rs` | `Client` — connects, sends/reads `Event`s, queues events for async flush |
| `frame.rs` | Binary framing protocol: 16 KB frames with metadata (size, frame number, related frames count); serializes events via `serde_json` |

**Frame format:** `[size: u16][frame_number: u8][related_frames: u8][data: 16380 bytes]`

**Events:**
- `Event::ReminderCreated(ReminderData)`
- `Event::ReminderDeleted(i64)`

---

## `hkb_core` — Shared Library

| Module | Purpose |
|---|---|
| `database/` | Diesel ORM wrapper; supports SQLite (default) or MySQL via features; global `Mutex<Connection>`; `init_database` runs embedded migrations |
| `database/models/reminders.rs` | Diesel model for the `reminders` table |
| `database/services/reminders.rs` | CRUD service: fetch/create/delete reminders with query options |
| `database/schema.rs` | Diesel-generated schema |
| `dtos/reminders.rs` | `ReminderData` DTO + `fakes` module for tests |
| `algorithms/huffman.rs` | Huffman encoding implementation |
| `data_structures/binary_tree.rs` | Generic binary tree |
| `data_structures/min_heap.rs` | Min-heap (used by Huffman) |
| `decoders/jpeg_decoder.rs` | Custom JPEG decoder |
| `logger.rs` | Wraps `log4rs`; supports file and stdout appenders |

**Database features:** `sqlite-database` (used everywhere) or `mysql-database`.

---

## `hkb_date` — Date/Time Library

| File | Purpose |
|---|---|
| `date.rs` | `SimpleDate` — local/UTC date wrapper with arithmetic |
| `duration.rs` | `Duration` enum: `Minute(u64)`, `Day(u64)`, etc. |
| `human_date_parsing.rs` | `HumanDateParser` — parses natural language dates |
| `grammar/human_date.pest` | PEG grammar (via **pest**) for human date strings |

---

## Key Cross-Cutting Concerns

- **IPC:** client ↔ daemon communicate over a Unix socket; events are framed, JSON-serialized
- **Database:** single SQLite file at `~/.local/share/hkb/db`; migrations embedded at compile time
- **Concurrency:** `parking_lot::Mutex` for the global DB connection; `tokio` for async I/O
- **Logging:** `log4rs` with structured targets (`CLIENT`, `DAEMON`, `CORE_DATABASE`, etc.)
