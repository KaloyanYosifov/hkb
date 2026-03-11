# HKB (Holy Kern Base) -- Project Mindmap & Rewrite Blueprint

> A personal productivity tool with a client-daemon architecture, currently focused on **reminder management** with plans for broader daily-workflow tooling. This document is designed to enable a full reimplementation in any programming language from a clean slate.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [High-Level Architecture](#2-high-level-architecture)
3. [Module Dependency Graph](#3-module-dependency-graph)
4. [Module 1: Date/Time Library (`hkb_date`)](#4-module-1-datetime-library-hkb_date)
5. [Module 2: Core Library (`hkb_core`)](#5-module-2-core-library-hkb_core)
6. [Module 3: IPC Transport Layer (`hkb_daemon_core`)](#6-module-3-ipc-transport-layer-hkb_daemon_core)
7. [Module 4: Background Daemon (`hkb_daemon`)](#7-module-4-background-daemon-hkb_daemon)
8. [Module 5: TUI Client (`hkb_client`)](#8-module-5-tui-client-hkb_client)
9. [Database Design](#9-database-design)
10. [IPC Protocol Specification](#10-ipc-protocol-specification)
11. [Human Date Parsing Grammar](#11-human-date-parsing-grammar)
12. [Feature Catalog](#12-feature-catalog)
13. [File System Layout & Conventions](#13-file-system-layout--conventions)
14. [Testing Strategy](#14-testing-strategy)
15. [CI/CD Pipeline](#15-cicd-pipeline)
16. [Reimplementation Checklist](#16-reimplementation-checklist)

---

## 1. Project Overview

**Name:** HKB (Holy Kern Base)

**Purpose:** A personal productivity application that simplifies daily workflow management. The current implementation focuses on **reminders** -- creating, listing, notifying, and cleaning up time-based reminders through a terminal UI that communicates with a background daemon process.

**Current Language:** Rust

**Original Goals:**
- Learn Rust, networking, and synchronization
- Build a stable, daily-use productivity tool
- Replace ad-hoc scripts accumulated over years of work
- Implement a client-daemon architecture with IPC

**Architecture Style:** Client-Daemon with shared database. The client is a TUI (Terminal User Interface) app; the daemon is a long-running background process. They communicate via Unix domain sockets and share a SQLite database.

**Future Vision (from architecture diagram):** The project envisions:
- A remote **Server** component with its own DB and synchronization
- A **CLI** client in addition to the TUI (GUI)
- Daemon-to-server synchronization

---

## 2. High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        CLIENT SIDE                           │
│                                                              │
│  ┌─────────────┐      Unix Socket      ┌─────────────────┐  │
│  │  TUI Client │ ◄──────────────────► │  Background      │  │
│  │  (hkb_client)│     (IPC Events)     │  Daemon          │  │
│  │             │                       │  (hkb_daemon)    │  │
│  │  - Render UI│                       │  - Notifications │  │
│  │  - CRUD ops │                       │  - Polling       │  │
│  │  - Input    │                       │  - Cleanup       │  │
│  │  - Navigation│                      │  - Audio alerts  │  │
│  └──────┬──────┘                       └────────┬─────────┘  │
│         │                                       │            │
│         │           ┌───────────┐               │            │
│         └──────────►│  SQLite   │◄──────────────┘            │
│                     │  Database │                             │
│                     └───────────┘                             │
│                                                              │
│  Shared Libraries:                                           │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐   │
│  │  hkb_core    │ │  hkb_date    │ │  hkb_daemon_core   │   │
│  │  (DB, DTOs,  │ │  (Date ops,  │ │  (Client, Server,  │   │
│  │   algorithms,│ │   parsing,   │ │   Frame protocol)  │   │
│  │   logger)    │ │   duration)  │ │                    │   │
│  └──────────────┘ └──────────────┘ └────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

**Key Architectural Decisions:**
1. Both client and daemon access the same SQLite database directly
2. The daemon is notified of changes via IPC events (not DB polling for new items)
3. The daemon polls the DB independently for time-based notification scheduling
4. All shared logic (DB access, DTOs, date utilities) lives in library modules

---

## 3. Module Dependency Graph

```
hkb_client ──┬──► hkb_core (with sqlite-database feature)
             ├──► hkb_date
             └──► hkb_daemon_core

hkb_daemon ──┬──► hkb_core (with sqlite-database feature)
             ├──► hkb_date
             └──► hkb_daemon_core

hkb_daemon_core ──► hkb_core (no DB feature, DTOs only)

hkb_core ──► hkb_date
```

**Build order (from leaf to root):** `hkb_date` → `hkb_core` → `hkb_daemon_core` → `hkb_daemon` / `hkb_client`

---

## 4. Module 1: Date/Time Library (`hkb_date`)

### Purpose
Standalone date/time library providing:
- A `SimpleDate` wrapper around native date/time
- Duration arithmetic (add/subtract minutes, hours, days, weeks, months, years)
- Natural language date parsing ("In 5 minutes", "Next Monday at 13:00", "Tomorrow")

### 4.1 `SimpleDate` (date.rs)

A date/time struct that wraps a naive datetime and a timezone enum.

**Fields:**
- `date`: Naive datetime (no timezone info baked in)
- `timezone`: Enum of `UTC` or `Local`

**Constructors:**
- `now()` -- current UTC time
- `local()` -- current local time
- `parse_from_rfc3339(string)` -- parse RFC 3339 date string
- `parse_from_str(string, format)` -- parse with custom format string

**Mutation Methods:**
- `set_year(i32)`, `set_month(u32)`, `set_day(u32)`, `set_hour(u32)`, `set_minute(u32)`, `set_second(u32)`
- `set_ymd(year, month, day)` -- set date part
- `set_hms(hour, minute, second)` -- set time part
- `set_ymdhms(...)` -- set both
- `set_start_of_day()` -- set to 00:00:00
- `set_end_of_day()` -- set to 23:59:59
- `start_of_day()` / `end_of_day()` -- chained versions returning new Self

**Arithmetic:**
- `add_duration(Duration)` -- add a duration to the date
- `sub_duration(Duration)` -- subtract a duration
- `Sub<SimpleDate>` operator -- returns a `std::time::Duration` (the absolute positive difference)

**Query Methods:**
- `year()`, `month()`, `day()`, `hour()`, `minute()`, `second()`
- `get_timezone()` -- returns the timezone enum
- `format(format_string)` -- format to string

**Serialization:**
- `Display` trait outputs RFC 3339 with seconds precision (`2024-04-14T08:00:00Z`)
- Implements `Serialize` and `Deserialize` (serde)
- Implements `Clone`, `Copy`, `Eq`, `PartialEq`

### 4.2 `Duration` (duration.rs)

An enum representing calendar-aware durations.

**Variants:**
- `Minute(u32)`
- `Hour(u32)`
- `Day(u32)`
- `Week(u32)`
- `Month(u32)` -- calendar month addition (handles year rollover)
- `Year(u32)` -- calendar year addition

**Key Behaviors:**
- `from_string(name, value)` -- factory from string ("minute", "hour", "day", "week", "month", "year")
- `get_value()` -- extracts the inner u32
- `Display` -- outputs `"Minute-5"`, `"Day-3"`, etc.
- Implements `Add<&Duration>` and `Sub<&Duration>` for NaiveDateTime
- Month and Year arithmetic is calendar-aware (e.g., adding 1 month to Jan 31 goes to Feb 28/29; handles year boundaries)

**`HumanizedDuration` trait:**
- Implemented for `std::time::Duration`
- `to_human_string()` -- converts seconds into "X days Y hours Z minutes" format
- Only includes non-zero units; singular/plural handling
- Returns empty string for durations less than 1 minute

### 4.3 Human Date Parser (human_date_parsing.rs + grammar/human_date.pest)

A natural language date parser using PEG grammar.

**Supported Sentence Patterns:**

| Pattern | Example | Behavior |
|---------|---------|----------|
| `IN` | "In 5 minutes", "In 3 months" | Add numeric duration to start date |
| `IN_ALT` | "In two days", "In three days at 5:00" | Cardinal number + days, optional time |
| `AT` | "At 05:00", "At 13:00 on the 11th of December" | Set time, optional date |
| `ON` | "On 5th of May", "On the 1st of August at 13:45" | Set specific date, optional time |
| `NEXT` | "Next Monday", "Next Week at 16:00", "Next Month" | Next weekday/week/month, optional time |
| `TOMORROW` | "Tomorrow", "Tomorrow at 15:35" | Next day, optional time |

**Grammar Rules (PEG):**
- `number`: single ASCII digit
- `time`: 1-2 digit number (for hour and minute)
- `day`: ordinal numbers (1st, 2nd, 3rd, 4th-31st)
- `month`: full month names (january through december)
- `duration`: second, minute, hour, day, week, month, year
- `weekday`: monday through sunday
- `cardinal`: two through nine (word form)
- `next_option`: weekday | "week" | "month"

**Parser Behavior:**
- Case insensitive (input is lowercased before parsing)
- Automatically rolls to next year when the specified date is in the past
- "Next [same weekday]" adds 7 days (not 0)
- Returns `SimpleDate` result

**Error Types:**
- `FailedToParseInput` -- input doesn't match grammar
- `UnknownRuleEncountered` -- internal parser error
- `InvalidDateError` -- resulting date is invalid
- `InvalidDurationSpecified` -- unknown duration string

---

## 5. Module 2: Core Library (`hkb_core`)

### Purpose
Shared library containing database access, DTOs, algorithms, data structures, image decoding, and logging. Conditionally compiled database feature (`sqlite-database` or `mysql-database`).

### 5.1 Database Layer

#### 5.1.1 Connection Management (database/mod.rs)

**Global Connection Pattern:**
- Single global `Mutex<Option<DatabaseConnection>>` (using `parking_lot`)
- `init_database(url, migrations)` -- establishes connection, runs all provided migrations in sequence, stores connection globally
- `within_database(callback)` -- acquires lock, passes mutable connection reference to callback, returns result
- Thread-safe via mutex; callers never hold the connection directly

**Database Backend Abstraction:**
- Compile-time feature flags: `sqlite-database` or `mysql-database`
- Uses `cfg_if!` macro to select between `SqliteConnection` and `MysqlConnection`
- Current usage: all binaries use `sqlite-database`

**Error Types:**
- `DatabaseNotInitialized` -- attempting DB operations before init
- `FailedToRunMigrations`
- `FailedToEstablishConnection`
- `FailedToFetchResult`

#### 5.1.2 Schema (database/schema.rs)

Single table:

```sql
CREATE TABLE reminders (
    id INTEGER PRIMARY KEY,
    note TEXT NOT NULL,
    remind_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL
)
```

**Column Types:**
- `id`: Auto-incrementing 64-bit integer
- `note`: Variable-length string
- `remind_at`: Stored as RFC 3339 date string
- `created_at`: Stored as RFC 3339 date string

#### 5.1.3 Models (database/models/reminders.rs)

Three model structs for ORM mapping:

1. **`Reminder`** (read model): `id: i64`, `note: String`, `remind_at: String`, `created_at: String`
2. **`CreateReminder`** (insert model): `note: String`, `remind_at: String`, `created_at: String`
3. **`UpdateReminder`** (update model): `note: Option<String>`, `remind_at: Option<String>`

Note: Dates are stored as strings in the DB and parsed to/from `SimpleDate` at the service layer boundary.

#### 5.1.4 Reminder Service (database/services/reminders.rs)

Full CRUD + query filtering:

**Functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `fetch_reminders` | `(options: Option<Vec<ReminderQueryOptions>>) -> Vec<ReminderData>` | List reminders with optional filters |
| `fetch_reminder` | `(id: i64) -> ReminderData` | Get single reminder by ID |
| `create_reminder` | `(data: CreateReminderData) -> ReminderData` | Create and return new reminder |
| `update_reminder` | `(data: UpdateReminderData) -> ReminderData` | Update fields and return updated |
| `delete_reminder` | `(id: i64) -> ()` | Delete single reminder |
| `delete_reminders` | `(option: ReminderQueryOptions) -> ()` | Bulk delete by query option |

**Query Options (`ReminderQueryOptions` enum):**

| Variant | Fields | SQL Equivalent |
|---------|--------|----------------|
| `RemindAtGe` | `date: SimpleDate` | `WHERE remind_at >= date` |
| `RemindAtLe` | `date: SimpleDate` | `WHERE remind_at <= date` |
| `RemindAtBetween` | `start_date, end_date: SimpleDate` | `WHERE remind_at BETWEEN start AND end` |
| `WithIds` | `ids: &Vec<i64>` | `WHERE id IN (ids...)` |
| `WithoutIds` | `ids: &Vec<i64>` | `WHERE id NOT IN (ids...)` |

Multiple query options can be combined (they are ANDed together). Results are ordered by `id ASC`.

### 5.2 DTOs (dtos/reminders.rs)

Data Transfer Objects used across module boundaries:

1. **`ReminderData`**: `id: i64`, `note: String`, `remind_at: SimpleDate`, `created_at: SimpleDate`
2. **`CreateReminderData`**: `note: String`, `remind_at: SimpleDate`
3. **`UpdateReminderData`**: `id: i64`, `note: Option<String>`, `remind_at: Option<SimpleDate>`

All implement `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`.

A `fakes` module provides test factory: `fakes::create_reminder()` returns a `ReminderData` with ID 1, note "Testing", and current local time.

### 5.3 Logger (logger.rs)

Structured logging with file and stdout appenders.

**Appender Types:**
- `File` -- writes to `__log/main.log` (debug) or `~/.cache/hkb/main.log` (release)
- `Stdout` -- writes to console

**Configuration:**
- Log pattern (debug): `{datetime} | {level} | {file}:{line} -- [{target}]: {message}`
- Log pattern (release): `{datetime} | {level} | [{target}]: {message}`
- Default level: `Debug` in dev, `Info` in release
- Override via `HKB_LOG_LEVEL` environment variable

**Log Targets Used Throughout:**
- `CLIENT`, `CLIENT_TERMINAL`, `CLIENT_MAIN`, `CLIENT_REMINDERS`, `CLIENT_REMINDERS_CREATE`, `CLIENT_REMINDERS_LIST`, `CLIENT_COMPONENT_IMAGE`
- `DAEMON`, `DAEMON_AUDIO`
- `CORE_DATABASE`, `CORE_REMINDERS_SERVICE`
- `DAEMON_CORE_CLIENT`

### 5.4 Algorithms (algorithms/)

#### Huffman Encoding (algorithms/huffman/)

A complete Huffman coding implementation (educational/exploratory feature).

**Components:**

1. **`HuffmanValue`** struct: `char: Option<char>`, `occurrence: u64`
   - Ordered by occurrence count (for min-heap priority)

2. **`HuffmanBinaryValue`** struct: `val: u32`, `max_bits: u32`
   - Represents a variable-width binary code
   - Supports `from_string("0110")`, `to_string_packed()`, bit shift/add operators

3. **`HuffmanBinary`** struct: `binary: Vec<HuffmanBinaryValue>`
   - The encoded output -- a sequence of variable-width binary values

4. **`Huffman`** struct (stateless):
   - `encode(text) -> (HuffmanBinary, HuffmanNode)` -- builds frequency table, constructs Huffman tree via min-heap, generates binary codes
   - `decode(binary, node) -> String` -- reverses the encoding

5. **`HuffmanNodeTraverser`** trait:
   - `to_binary(char) -> Option<HuffmanBinaryValue>` -- BFS traversal to find char's code
   - `from_binary(binary) -> Option<char>` -- BFS traversal to decode

### 5.5 Data Structures (data_structures/)

#### Binary Tree (data_structures/binary_tree.rs)

Generic binary tree node with reference counting.

**`Node<T>`** where `T: PartialEq + Eq + PartialOrd + Ord`:
- `val: T` -- stored value
- `left: Option<Rc<RefCell<Node<T>>>>` -- left child (reference counted, interior mutable)
- `right: Option<Rc<RefCell<Node<T>>>>` -- right child

**Methods:**
- `with_value(val)` -- leaf node
- `new(val, left_val, right_val)` -- node with optional child values
- `with_nodes(val, left_node, right_node)` -- node with pre-built children
- `get_left()`, `get_right()` -- clone of `Option<NodeRef<T>>`
- `height()` -- recursive height calculation

Implements `PartialEq`, `Eq`, `Ord`, `PartialOrd` based on `val`.

#### Min Heap (data_structures/min_heap.rs)

Generic min-heap (priority queue).

**`MinHeap<T>`** where `T: PartialEq + Eq + PartialOrd + Ord`:
- `insert(value)` -- add value, bubble up
- `pop() -> Option<T>` -- remove minimum, heapify down
- `has_element() -> bool`
- `size() -> usize`

Uses a `Vec<T>` internally with standard array-based heap indexing (parent = (i-1)/2, children = 2i+1, 2i+2).

### 5.6 JPEG Decoder (decoders/jpeg_decoder.rs)

A **work-in-progress** custom JPEG decoder (educational/exploratory feature).

**Current capabilities:**
- Reads JPEG file markers (Start, Header/JFIF, Quantization Table, Frame Start, Huffman Table, End)
- Parses JFIF header (identifier, version, units, density, thumbnail)
- Reads raw bytes for quantization/huffman/frame tables
- Does NOT produce decoded pixel data yet (prints raw table bytes)

**Marker Constants:**
- Start: `[0xFF, 0xD8]`
- Header (JFIF): `[0xFF, 0xE0]`
- Quantization: `[0xFF, 0xDB]`
- Frame Start: `[0xFF, 0xC0]`
- Huffman: `[0xFF, 0xC4]`
- End: `[0xFF, 0xD9]`

---

## 6. Module 3: IPC Transport Layer (`hkb_daemon_core`)

### Purpose
Provides the communication layer between client and daemon over Unix domain sockets, including connection management, event serialization, and a binary framing protocol.

### 6.1 Server (server.rs)

**Socket Location:** `~/.local/share/hkb/hkb.sock` (platform-dependent via `dirs::data_dir()`)

**`Server` struct:**
- `bind()` -- creates data directory if missing, removes stale socket file, binds UnixListener
- `accept()` -- async, returns `(UnixStream, SocketAddr)`
- `get_addr()` -- returns socket file path

**Lifecycle:**
1. On startup, remove existing socket file (prevents "address in use" errors)
2. Bind to socket
3. Accept connections in a loop, spawning async tasks per connection

### 6.2 Client (client.rs)

**`Client` struct:**
- `connect()` -- async, connects to daemon socket; returns error if daemon is not running
- `from_stream(UnixStream)` -- wrap an already-established stream (used server-side)
- `read_event()` -- async, reads a single frame from socket and deserializes to Event
- `send_event(event)` -- async, serializes event into frame sequence, writes all frames
- `queue_event(event)` -- adds event to internal deque for batched sending
- `flush()` -- async, sends one queued event per call
- Internal `event_queue: VecDeque<Event>` for batched sending

**Reconnection Logic (in client binary):**
- If connection fails, retry every 3 seconds
- If connection drops during operation, drop client and spawn new connection attempt
- Flush queued events every 500ms

### 6.3 Frame Protocol (frame.rs)

**Constants:**
- `FRAME_SIZE`: 16,384 bytes (16 KB) -- total frame size including metadata
- `FRAME_METADATA_SIZE`: 4 bytes
- `FRAME_DATA_SIZE`: 16,380 bytes

**Frame Layout:**
```
┌────────────┬──────────────┬────────────────┬──────────────────────┐
│ size (u16) │ frame_no (u8)│ related_frames │    data (16380 B)    │
│  2 bytes   │   1 byte     │    (u8) 1 byte │                      │
└────────────┴──────────────┴────────────────┴──────────────────────┘
```

- **size**: Number of valid data bytes in this frame (0-16380)
- **frame_number**: 1-indexed sequence number within a multi-frame message
- **related_frames**: Total number of frames in the sequence
- **data**: Payload bytes, zero-padded to 16380

**Serialization:**
- Events are serialized to JSON via serde_json
- JSON string is split into 16380-byte chunks
- Each chunk becomes one frame
- Maximum theoretical message size: 255 frames x 16380 bytes = ~4 MB

**Events Enum:**
- `ReminderCreated(ReminderData)` -- sent when client creates a reminder
- `ReminderDeleted(i64)` -- sent when client deletes a reminder (carries reminder ID)

**Frame construction:**
- `Frame::from_event(event)` -> `Vec<Frame>` (a "FrameSequence")
- `create_frame_from_bytes(buffer)` -- unsafe transmute from raw bytes to Frame struct
- `Frame::convert_to_bytes()` -- unsafe cast from Frame struct to byte slice

**WARNING:** The current implementation uses `unsafe` memory transmutation. A reimplementation should use proper serialization/deserialization.

---

## 7. Module 4: Background Daemon (`hkb_daemon`)

### Purpose
Long-running background process that provides reminder notifications, cleanup, and acts as the server endpoint for IPC.

### 7.1 Startup Sequence

1. Resolve database path: `~/.local/share/hkb/db` (platform-dependent)
2. Initialize database with core migrations
3. Initialize logger (both file and stdout appenders)
4. Bind Unix socket server
5. Spawn audio subsystem (async task)
6. Spawn reminder handler (async task)
7. Enter accept loop (main task)

### 7.2 Connection Handling

Each accepted connection is spawned as an async task:
- Reads events from client (currently logs them, no action taken)
- Flushes queued events to client every 500ms
- Detects client disconnect and cleans up

### 7.3 Reminder Notification System

**Polling Interval:** Every 10 seconds

**Notification Intervals:**
The daemon checks for upcoming reminders at four time windows relative to the current time:

| Start Offset | End Offset | Notification Text |
|-------------|-----------|------------------|
| 0 minutes | 1 minute | "1 minute" |
| 0 minutes | 5 minutes | "5 minutes" |
| 6 minutes | 15 minutes | "15 minutes" |
| 16 minutes | 30 minutes | "30 minutes" |

**Deduplication:**
- A `HashMap<String, Vec<i64>>` tracks which reminders have been notified for each interval
- Key: duration string (e.g., "Minute-5"), Value: list of already-reminded IDs
- Reminders are fetched with `WithoutIds` filter to exclude already-notified ones

**Notification Delivery:**
- Uses OS-native notifications (via `notify-rust`)
- Summary: "You have a reminder in: {timeframe}"
- Body: reminder note text
- Timeout: 3000ms
- Audio: plays `notification.wav` after any batch of notifications

### 7.4 Reminder Cleanup

**Cleanup Interval:** Every 5 minutes

**Logic:** Delete all reminders where `remind_at <= (now - 1 day)`

This means reminders are kept for up to 24 hours past their due time, then automatically purged.

### 7.5 Audio Subsystem (audio.rs)

**Architecture:** Message-passing via async mpsc channel

**Init:**
1. Create mpsc channel (capacity: 32)
2. Store sender in global mutex
3. Receiver loop: wait for path messages, play audio

**Playing:**
- Audio files resolved relative to `~/.local/share/hkb/sound/` (platform-dependent)
- Uses rodio library: opens default output device, creates sink, decodes file, plays to completion
- Blocking playback (sink.sleep_until_end)

**Sound Files:**
- `notification.wav` -- must be synced to data directory via `make sync_files_with_data_dir`

---

## 8. Module 5: TUI Client (`hkb_client`)

### Purpose
Terminal User Interface application for interacting with reminders. Built with ratatui (Rust TUI framework) and crossterm (terminal manipulation).

### 8.1 Startup Sequence

1. Initialize logger (file appender only)
2. Resolve and initialize database with both core and client migrations
3. Create async mpsc channel for daemon communication
4. Store sender in global singleton
5. Spawn daemon connection task (auto-reconnecting)
6. Initialize terminal (enter alternate screen, enable raw mode)
7. Create app instances and navigation
8. Enter render loop (~60 FPS)

### 8.2 Main Render Loop

```
while !should_quit:
    1. Poll all available terminal events (non-blocking)
    2. Handle global keys:
       - Ctrl+C → quit
       - Esc → exit edit mode
    3. Push events to global event handler
    4. Draw frame:
       - Navigation bar (top, 1 line)
       - Active view (middle, fills remaining)
       - Status bar (bottom, 1 line: "Insert mode" or "Normal Mode")
    5. Clear event buffer
    6. Sleep 16ms (~60 FPS)
```

### 8.3 Application State (app_state.rs)

**Global mutable state** stored in `Mutex<Option<AppState>>`:

| Field | Type | Purpose |
|-------|------|---------|
| `view` | `AppView` enum | Current active view (Main or Reminders) |
| `editing` | `bool` | Whether the app is in insert/edit mode |
| `ignore_navigation_events` | `bool` | Prevents tab-switching during form input |

**AppView Enum:**
- `Main` -- home screen (currently shows an image)
- `Reminders` -- reminder management

### 8.4 Event System (events.rs)

**Global Event Handler** with the following capabilities:

- `push(event)` -- add crossterm event to buffer
- `consume_if(predicate)` -- remove and return all matching events
- `consume(index)` -- remove specific event by index
- `clear()` -- clear all events
- Double-press detection: tracks `previous_key`, `times_pressed`, `key_release_delay` (300ms threshold)
- `is_pressed_at_least(char, times)` -- detect repeated key presses (used for "dd" to delete)
- `reset_key_press()` -- reset the double-press counter

**Macros:**
- `consume_key_events!(pattern)` -- filter and consume key events matching a pattern
- `consume_key_event!(pattern => expr, ...)` -- consume and dispatch key events
- `has_key_event!(pattern)` -- check if a matching event exists (consumes it)

### 8.5 Focus Management (focus.rs)

**`Focusable` trait:**
- `focus(&mut self)` -- mark as focused
- `blur(&mut self)` -- mark as unfocused

Implemented by: `InputState`, `ButtonState`

### 8.6 Views

#### 8.6.1 Main View (apps/main.rs)

**Current behavior:** Renders a JPEG image (`images/image.jpeg`) as ASCII art in the terminal.
- Resizes image to fit terminal area on first render
- Uses the Image component (ASCII art rendering)

#### 8.6.2 Reminders View (apps/reminders.rs)

**Architecture:** Internal router with sub-views and message passing.

**Sub-views:**
- `RemindersList` (default) -- shows today's and upcoming reminders
- `RemindersCreate` -- form for creating new reminders

**Message Enum:**
- `ChangeView(View)` -- switch between List and Create
- `CreateReminder(CreateReminderData)` -- trigger reminder creation
- `DeleteReminder(i64)` -- trigger reminder deletion

**Flow:**
1. Sub-view's `update()` returns an `Option<Message>`
2. Parent processes message: creates/deletes reminder in DB, sends IPC event to daemon, reinitializes view

#### 8.6.3 Reminders List (apps/reminders/reminders_list.rs)

**Layout:**
- Top 30%: "Today Reminders" list
- Bottom 70%: "Upcoming Reminders" list

**Display Format:** `{note} - {humanized_duration} ({date DD-MM-YYYY})`

**Today Reminders Query:** `WHERE remind_at BETWEEN start_of_today AND end_of_today`
**Upcoming Reminders Query:** `WHERE remind_at >= end_of_today`

**Keyboard Controls:**
| Key | Action |
|-----|--------|
| `j` | Move selection down |
| `k` | Move selection up |
| `a` or `A` | Switch to Create view |
| `dd` (double-press) or `Backspace` + `d` pressed twice | Delete selected reminder |

**Selection:** Uses `BoundedValue` to clamp selection index within valid range across both lists.

#### 8.6.4 Reminders Create (apps/reminders/reminders_create.rs)

**Layout:** Centered form with title input, date input, optional error message, and submit button.

**Form Fields:**
1. Title (text input) -- required
2. Reminder Date (text input, parsed by HumanDateParser) -- required
3. Submit button

**Focus Cycling:** Tab / Shift+Tab cycles through inputs and button.

**Validation:**
- Title must not be empty
- Reminder Date must not be empty
- Reminder Date must be parseable by `HumanDateParser`

**On Submit:**
- Parses date, creates `CreateReminderData`, returns `Message::CreateReminder`
- Parent creates reminder in DB and sends `ReminderCreated` event to daemon

**Mode Handling:**
- Entering create view: sets editing mode (insert mode), disables navigation events
- Returning to list view: clears editing mode, re-enables navigation events

### 8.7 UI Components

#### 8.7.1 Component Traits

```
trait Component {
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

trait StatefulComponent {
    type State;
    fn render(&mut self, frame: &mut Frame, state: &mut Self::State, area: Rect);
}
```

#### 8.7.2 Navigation (components/navigation.rs)

**Layout:** Black background bar with app title (left 30%) and tab bar (right 70%).

**Behavior:**
- Tab key cycles through views
- Respects `should_ignore_navigation_events()` flag
- Selected tab highlighted in white, others gray
- Updates global `AppView` state on tab change

#### 8.7.3 Input (components/input.rs)

A full-featured text input with vi-like keybindings.

**State (`InputState`):**
- `buffer: String` -- the input text
- `focused: bool`
- `cursor_offset: BoundedValue` -- cursor position within visible area
- `visible_buffer_offset: usize` -- scroll offset for long text
- `last_render_width: u16` -- tracks visible width for overflow

**Normal Mode Keybindings:**
| Key | Action |
|-----|--------|
| `h` | Move cursor left |
| `l` | Move cursor right |
| `^` | Go to start of input |
| `$` | Go to end of input |
| `e` | Jump to end of word |
| `b` | Jump to beginning of word |
| `i` | Enter insert mode at cursor |
| `I` | Enter insert mode at start |
| `a` | Enter insert mode after cursor |
| `A` | Enter insert mode at end |

**Insert Mode Keybindings:**
| Key | Action |
|-----|--------|
| Any char | Insert character at cursor |
| Left arrow | Move cursor left |
| Right arrow | Move cursor right |
| Backspace | Delete character before cursor |
| Esc | Return to normal mode (global handler) |

**Word Navigation:**
- Classifies characters into: whitespace (0), punctuation (1), alphanumeric (2)
- `e` jumps forward to end of current/next word class boundary
- `b` jumps backward to beginning of current/previous word class boundary

**Scrolling:**
- When text exceeds visible width, the input scrolls horizontally
- `visible_buffer_offset` tracks how many characters are scrolled past the left edge

#### 8.7.4 Button (components/button.rs)

**State (`ButtonState`):** `clicked: bool`, `focused: bool`

**Rendering:** Displays `<label>` text; inverted colors when focused.

**Behavior:** Enter key while focused triggers click.

#### 8.7.5 Image (components/image.rs)

**ASCII Art Renderer:**
- Converts pixel luminance to ASCII characters
- Character set (darkest to lightest): `Ñ @ # W $ 9 8 7 6 5 4 3 2 1 0 ? ! a b c ; : + = - , . _ [space]` (29 characters)
- Luminance formula: `0.2126*R + 0.7152*G + 0.0722*B`
- Maps luminance (0-255) to character index

### 8.8 Utilities

#### BoundedValue (utils/bounded_value.rs)

Numeric value clamped between min and max bounds.

**`BoundedValue` struct:** `val: usize`, `min_val: usize`, `max_val: usize`

**Methods:**
- `set_val(val)` -- clamp to max
- `add_val(val)` -- saturating add, clamp to max
- `sub_val(val)` -- saturating sub, clamp to min
- `set_max(max)` -- update max, clamp current val if needed
- `set_to_min()`, `set_to_max()`
- `get_val()`, `get_max_val()`

Implements `Add`, `Sub`, `AddAssign`, `SubAssign` for `BoundedValue`, `usize`, and `u16`.
Implements `PartialEq`, `Eq`, `Ord`, `PartialOrd` (for both `BoundedValue` and `usize`).

#### Centered Layout (utils.rs)

`centered_layout(area, percent_x, percent_y) -> Rect` -- returns a centered rectangle within the given area using percentage-based sizing.

### 8.9 Singleton (singleton.rs)

Global holder for the daemon message sender (`tokio::sync::mpsc::Sender<FrameEvent>`).

- `set_server_msg_sender(sender)` -- store during bootstrap
- `send_server_msg(event)` -- clone sender, spawn async send task on tokio runtime

### 8.10 Terminal (terminal.rs)

**Setup:**
- Enters alternate screen (preserves user's terminal content)
- Enables raw mode (character-by-character input)
- Sets panic hook to restore terminal on crash

**Cleanup:**
- Disables raw mode
- Leaves alternate screen

**Cursor Styles:**
- `set_cursor_steady_bar()` -- used in edit mode
- `set_cursor_to_default()` -- used in normal mode

---

## 9. Database Design

### Tables

**`reminders`:**

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| `id` | INTEGER | PRIMARY KEY, auto-increment | 64-bit |
| `note` | TEXT | NOT NULL | Reminder description |
| `remind_at` | TIMESTAMP (stored as TEXT) | NOT NULL | RFC 3339 string |
| `created_at` | TIMESTAMP (stored as TEXT) | NOT NULL | RFC 3339 string, set at creation |

**`test`** (client-only migration, appears unused):

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY |
| `gest` | VARCHAR | NOT NULL |
| `mest` | VARCHAR | NOT NULL |

### Migration Strategy

- Migrations are embedded at compile time (no runtime file access needed)
- Migrations are ordered and applied sequentially
- Both client and daemon run core migrations; client also runs its own migrations
- Down migrations exist for rollback (`DROP TABLE`)

### Database Location

- **Debug:** Current working directory (in-memory for tests)
- **Release:** `~/.local/share/hkb/db` (Linux) or `~/Library/Application Support/hkb/db` (macOS)

---

## 10. IPC Protocol Specification

### Transport
- Unix Domain Socket at `~/.local/share/hkb/hkb.sock`
- Async I/O (non-blocking reads/writes via tokio)

### Message Format

```
Event → JSON string → Frame Sequence → Raw bytes on socket
```

### Frame Binary Layout (16,384 bytes total)

```
Offset  Size  Field
0       2     data_size (u16, little-endian)
2       1     frame_number (u8, 1-indexed)
3       1     related_frames (u8, total frames in sequence)
4       16380 data (zero-padded payload)
```

### Event Types (JSON)

```json
// ReminderCreated
{"ReminderCreated": {"id": 1, "note": "...", "remind_at": "2024-...", "created_at": "2024-..."}}

// ReminderDeleted
{"ReminderDeleted": 42}
```

### Connection Lifecycle

1. Client connects to socket
2. Bidirectional event exchange
3. Client flushes queued events every 500ms
4. Disconnect detected via read returning 0 bytes or write error
5. Client auto-reconnects every 3 seconds on disconnect

---

## 11. Human Date Parsing Grammar

### PEG Grammar (Pest Format)

```
SENTENCE = IN | IN_ALT | AT | ON | NEXT | TOMORROW

IN       = "in " + digits + " " + duration + optional("s")
IN_ALT   = "in " + cardinal + " days" + optional(" " + AT)
AT       = "at " + HH + ":" + MM + optional(" " + ON)
ON       = "on " + optional("the ") + ordinal + " of " + month + optional(" " + AT)
NEXT     = "next " + next_option + optional(" " + AT)
TOMORROW = "tomorrow" + optional(" " + AT)
```

### Composability
- `AT` can contain an `ON` clause
- `ON` can contain an `AT` clause
- `IN_ALT` can contain an `AT` clause
- `NEXT` can contain an `AT` clause
- `TOMORROW` can contain an `AT` clause

This allows natural combinations like "At 13:00 on the 5th of May" or "Next Monday at 15:00".

---

## 12. Feature Catalog

### Production Features

| # | Feature | Status | Description |
|---|---------|--------|-------------|
| 1 | Reminder CRUD | Complete | Create, read, update, delete reminders via DB service |
| 2 | TUI Reminder List | Complete | View today's and upcoming reminders with selection |
| 3 | TUI Reminder Create | Complete | Form with title and natural language date input |
| 4 | TUI Reminder Delete | Complete | Double-press 'd' or Backspace to delete selected |
| 5 | Daemon Notifications | Complete | OS notifications at 1/5/15/30 minute intervals |
| 6 | Notification Audio | Complete | Plays WAV sound on reminder notification |
| 7 | Reminder Auto-Cleanup | Complete | Deletes reminders older than 1 day (every 5 min) |
| 8 | Client-Daemon IPC | Complete | Unix socket with binary framing protocol |
| 9 | Auto-Reconnect | Complete | Client reconnects to daemon every 3s on disconnect |
| 10 | Natural Language Dates | Complete | "In 5 minutes", "Next Monday at 13:00", etc. |
| 11 | Vi-like Input | Complete | Normal/insert modes, h/l/e/b/^/$ navigation |
| 12 | Tab Navigation | Complete | Tab key switches between Main and Reminders views |
| 13 | Structured Logging | Complete | File and stdout appenders with targets |

### Experimental/Learning Features

| # | Feature | Status | Description |
|---|---------|--------|-------------|
| 14 | Huffman Encoding | Complete | Encode/decode text with Huffman coding |
| 15 | Binary Tree | Complete | Generic binary tree with Rc<RefCell> nodes |
| 16 | Min Heap | Complete | Priority queue with property-based tests |
| 17 | JPEG Decoder | WIP | Reads JPEG markers/headers, no pixel output |
| 18 | ASCII Image Render | Complete | Renders image as ASCII art in terminal |

### Planned (from architecture diagram)

| # | Feature | Status | Description |
|---|---------|--------|-------------|
| 19 | Remote Server | Not started | Central server with DB + sync |
| 20 | CLI Client | Not started | Command-line interface alongside TUI |
| 21 | Client-Server Sync | Not started | Synchronize local DB with remote |

---

## 13. File System Layout & Conventions

### Runtime Directories

| Path | Purpose | Platform |
|------|---------|----------|
| `~/.local/share/hkb/db` | SQLite database | Linux |
| `~/Library/Application Support/hkb/db` | SQLite database | macOS |
| `~/.local/share/hkb/hkb.sock` | Unix domain socket | Linux |
| `~/Library/Application Support/hkb/hkb.sock` | Unix domain socket | macOS |
| `~/.local/share/hkb/sound/` | Audio files | Linux |
| `~/Library/Application Support/hkb/sound/` | Audio files | macOS |
| `~/.cache/hkb/main.log` | Log file (release) | Linux |
| `__log/main.log` | Log file (debug) | Both (CWD) |

### Project Structure Convention

```
project_root/
├── Cargo.toml                 # Workspace definition
├── Makefile                   # Build/run commands
├── module_name/
│   ├── Cargo.toml             # Module dependencies
│   ├── Makefile               # Module-specific commands (optional)
│   ├── migrations/            # SQL migrations (if DB-using)
│   │   └── YYYY-MM-DD-HHMMSS_name/
│   │       ├── up.sql
│   │       └── down.sql
│   ├── src/
│   │   ├── lib.rs or main.rs  # Entry point
│   │   ├── feature/
│   │   │   ├── mod.rs
│   │   │   └── sub_module.rs
│   │   └── ...
│   └── test-files/            # Test fixtures (optional)
└── docs/
    └── images/
```

---

## 14. Testing Strategy

### Unit Tests
- Database service tests use in-memory SQLite (`:memory:`)
- Tests run serially (`#[serial]`) due to global DB connection
- Table truncation between tests via raw SQL DELETE
- Constructor test (`#[ctor]`) initializes DB once before all tests

### Property-Based Tests
- Min heap uses `proptest` to verify heap invariant with random inputs
- Tests verify both insertion ordering and pop ordering

### Snapshot Tests
- Huffman encoding output verified via `insta` snapshot testing
- Binary-to-char conversion verified via snapshots

### Integration Tests
- Frame protocol tested with fixture files (`big_file.txt` for multi-frame scenarios)
- JPEG decoder tests marked `#[ignore]` (require manual run with test files)

### CI
- GitHub Actions on PR (opened, reopened, synchronize)
- Steps: lint (clippy), test, build
- Ubuntu 22.04 runner
- Installs `libasound2-dev` for audio support

---

## 15. CI/CD Pipeline

```yaml
Trigger: Pull Request (opened, reopened, synchronize)
Runner: ubuntu-22.04
Dependencies: libasound2-dev (ALSA audio)
Cache: ~/.cargo + target/

Steps:
1. Checkout
2. Restore cargo cache
3. Install system deps
4. Lint: cargo clippy -- -D warnings (with specific allowed lints)
5. Test: cargo test
6. Build: cargo build
```

---

## 16. Reimplementation Checklist

### Phase 1: Foundation Libraries

- [ ] **Date/Time Library**
  - [ ] SimpleDate struct with UTC/Local timezone support
  - [ ] Duration enum (Minute, Hour, Day, Week, Month, Year)
  - [ ] Calendar-aware month/year arithmetic (handle rollover)
  - [ ] Date addition and subtraction
  - [ ] RFC 3339 serialization/deserialization
  - [ ] Start/end of day helpers
  - [ ] Duration humanization ("2 days 3 hours 15 minutes")
  - [ ] PEG grammar for natural language date parsing
  - [ ] Six sentence patterns: IN, IN_ALT, AT, ON, NEXT, TOMORROW
  - [ ] Case-insensitive parsing
  - [ ] Auto year-rollover for past dates

- [ ] **Core Library**
  - [ ] Database abstraction with connection pooling/global connection
  - [ ] Migration runner (embedded migrations)
  - [ ] SQLite support (MySQL optional)
  - [ ] Reminder CRUD service with query options
  - [ ] DTO types with serialization (ReminderData, CreateReminderData, UpdateReminderData)
  - [ ] Query filter: by date range, by ID inclusion/exclusion
  - [ ] Structured logger with file and stdout outputs, configurable levels

### Phase 2: IPC Layer

- [ ] **Transport Library**
  - [ ] Unix domain socket server (bind, accept)
  - [ ] Unix domain socket client (connect, read, write)
  - [ ] Binary framing protocol (16 KB frames, metadata header)
  - [ ] Frame sequence support for large messages
  - [ ] JSON event serialization within frames
  - [ ] Event types: ReminderCreated, ReminderDeleted
  - [ ] Connection error handling and reporting

### Phase 3: Daemon

- [ ] **Background Daemon**
  - [ ] Unix socket server accepting multiple clients
  - [ ] Reminder polling (every 10 seconds)
  - [ ] 4-tier notification intervals (1m, 5m, 15m, 30m)
  - [ ] Deduplication of notifications per interval
  - [ ] OS-native notification delivery
  - [ ] Audio playback on notification (WAV file)
  - [ ] Automatic reminder cleanup (older than 1 day, every 5 minutes)
  - [ ] Event receiving from clients (logging currently)
  - [ ] Graceful client disconnect handling

### Phase 4: TUI Client

- [ ] **Terminal Setup**
  - [ ] Alternate screen mode
  - [ ] Raw mode input
  - [ ] Panic recovery (restore terminal on crash)
  - [ ] Cursor style switching (bar for insert, default for normal)

- [ ] **Application Framework**
  - [ ] Global application state (current view, edit mode, navigation lock)
  - [ ] Event queue with consume-on-read semantics
  - [ ] Double-press key detection (300ms threshold)
  - [ ] Focus management (Focusable trait)
  - [ ] Component/StatefulComponent trait system
  - [ ] ~60 FPS render loop

- [ ] **Navigation**
  - [ ] Tab bar with view switching
  - [ ] Navigation event suppression during form input
  - [ ] Status bar showing current mode (Normal/Insert)

- [ ] **Reminders List View**
  - [ ] Split layout: today (30%) and upcoming (70%)
  - [ ] Humanized duration display
  - [ ] j/k selection navigation
  - [ ] dd double-press to delete
  - [ ] 'a' to switch to create view

- [ ] **Reminders Create View**
  - [ ] Title input field
  - [ ] Date input field with natural language parsing
  - [ ] Tab/Shift+Tab focus cycling
  - [ ] Validation with error messages
  - [ ] Submit button
  - [ ] Auto-enter insert mode, disable navigation

- [ ] **Input Component**
  - [ ] Vi-like normal mode (h, l, e, b, ^, $, i, I, a, A)
  - [ ] Insert mode (character input, backspace, arrow keys)
  - [ ] Horizontal scrolling for overflow text
  - [ ] Word-class-aware navigation (whitespace, punctuation, alphanumeric)
  - [ ] Cursor position tracking within visible area

- [ ] **Daemon Connection**
  - [ ] Async connection to daemon socket
  - [ ] Auto-reconnect every 3 seconds
  - [ ] Event queuing and periodic flushing (500ms)
  - [ ] Send ReminderCreated/ReminderDeleted on CRUD operations

### Phase 5: Optional/Educational Features

- [ ] Huffman encoding/decoding
- [ ] Generic binary tree
- [ ] Min heap (priority queue)
- [ ] JPEG decoder
- [ ] ASCII art image rendering

### Phase 6: Future Features (from architecture diagram)

- [ ] CLI client (command-line interface)
- [ ] Remote server with its own database
- [ ] Client-to-server synchronization
- [ ] Multi-device support

---

## Appendix A: Key Technical Decisions

| Decision | Rationale |
|----------|-----------|
| SQLite for local storage | Single-file, zero-config, embedded -- ideal for desktop app |
| Unix domain sockets for IPC | Low latency, no network stack overhead, OS-level access control |
| 16 KB frame size | Balances overhead vs. typical message size; most events fit in one frame |
| JSON for event serialization | Human-readable, easy to debug, serde ecosystem |
| Global mutex for DB connection | Simple single-writer model; acceptable for single-user app |
| PEG grammar for date parsing | Declarative, maintainable, maps naturally to sentence structures |
| ~60 FPS render loop | Responsive feel without excessive CPU usage |
| Vi-like keybindings | Developer-friendly, consistent with terminal culture |
| RFC 3339 date storage | Unambiguous, sortable as strings, timezone-aware |
| Embedded migrations | No external migration files needed at runtime |

## Appendix B: External Dependencies Summary

| Dependency | Purpose | Module |
|-----------|---------|--------|
| chrono | Date/time operations | hkb_date |
| pest/pest_derive | PEG parser generator | hkb_date |
| diesel | ORM / database queries | hkb_core |
| diesel_migrations | Compile-time migration embedding | hkb_core, hkb_client, hkb_daemon |
| log4rs | Structured logging framework | hkb_core |
| parking_lot | Fast mutex implementation | All modules |
| serde / serde_json | Serialization framework | All modules |
| tokio | Async runtime | hkb_client, hkb_daemon, hkb_daemon_core |
| ratatui | TUI rendering framework | hkb_client |
| crossterm | Terminal manipulation | hkb_client |
| image | Image loading/resizing | hkb_client |
| notify-rust | OS notification API | hkb_daemon |
| rodio | Audio playback | hkb_daemon |
| dirs | Platform-specific directory resolution | All modules |
| thiserror | Ergonomic error types | All modules |
| proptest | Property-based testing | hkb_core |
| insta | Snapshot testing | hkb_core |

## Appendix C: Equivalent Library Suggestions by Language

| Concern | Rust (Current) | Go | Python | TypeScript/Node |
|---------|---------------|-----|--------|----------------|
| Date/Time | chrono | time (stdlib) | datetime / pendulum | date-fns / luxon |
| PEG Parsing | pest | pigeon / participle | parsimonious / lark | pegjs / ohm |
| ORM | diesel | GORM / sqlx | SQLAlchemy / peewee | Prisma / Knex |
| TUI | ratatui + crossterm | bubbletea / tview | textual / rich | blessed / ink |
| Async Runtime | tokio | goroutines (stdlib) | asyncio | Node event loop |
| Notifications | notify-rust | beeep / go-toast | plyer / notify-py | node-notifier |
| Audio | rodio | beep / oto | playsound / pygame | play-sound |
| Logging | log4rs | zerolog / zap | loguru / structlog | winston / pino |
| Serialization | serde | encoding/json | json (stdlib) | JSON (native) |
| IPC Socket | tokio UnixStream | net.Dial("unix") | socket (stdlib) | net.createConnection |
