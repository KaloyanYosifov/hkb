# HKB Go Rewrite -- Execution Plan

> Structured prompt sequence for an AI agent to rebuild HKB (Holy Kern Base) in Go from scratch.
> Each prompt is self-contained with enough context to execute independently.
> Read `PROJECT_MINDMAP.md` for the full original specification.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│  ┌──────────┐    ┌──────────┐    ┌──────────────────────────────────┐   │
│  │   CLI    │    │   TUI    │    │         hkb daemon               │   │
│  │  client  │    │  client  │    │                                  │   │
│  │ cmd/cli/ │    │ cmd/tui/ │    │  - Unix socket server            │   │
│  └────┬─────┘    └────┬─────┘    │  - Reminder notifications        │   │
│       │               │          │  - Cleanup scheduler             │   │
│       │               │          │  - Audio alerts                  │   │
│       └───────┬───────┘          │  - (future) Sync client          │   │
│               │                  └──────────┬───────────────────────┘   │
│               │                             │                           │
│          uses │                        uses │                           │
│               ▼                             ▼                           │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    internal packages                            │    │
│  │                                                                 │    │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐               │    │
│  │  │  storage   │  │    ipc     │  │  dateutil   │               │    │
│  │  │            │  │            │  │             │               │    │
│  │  │ Repository │  │ Server     │  │ SimpleDate  │               │    │
│  │  │ interface  │  │ Client     │  │ Duration    │               │    │
│  │  │            │  │ Frame      │  │ HumanParse  │               │    │
│  │  │ SQLite     │  │ Protocol   │  │             │               │    │
│  │  │ impl       │  │ Events     │  └────────────┘               │    │
│  │  │            │  └────────────┘                                │    │
│  │  │ (future)   │                  ┌────────────┐               │    │
│  │  │ Postgres   │                  │   notify   │               │    │
│  │  │ impl       │                  │            │               │    │
│  │  │            │                  │ OS notifs   │               │    │
│  │  │ (future)   │                  │ Audio play  │               │    │
│  │  │ Remote API │                  └────────────┘               │    │
│  │  │ impl       │                                                │    │
│  │  └────────────┘  ┌────────────┐  ┌────────────┐               │    │
│  │                   │   logger   │  │   config   │               │    │
│  │                   │            │  │            │               │    │
│  │                   │ Structured │  │ Paths      │               │    │
│  │                   │ File+Stdout│  │ Platform   │               │    │
│  │                   └────────────┘  │ Env vars   │               │    │
│  │                                   └────────────┘               │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                  Future: Remote Sync Server                     │    │
│  │  cmd/server/ -- HTTP/gRPC API, Postgres, auth, sync protocol   │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Key Design Principles

1. **Interface-driven storage.** All data access goes through a `Repository` interface. The first implementation is SQLite. Future implementations (Postgres, remote API) slot in without touching business logic. This is the primary extensibility mechanism for online sync.

2. **Separate binaries, shared packages.** Three `cmd/` entry points (`tui`, `daemon`, `cli`) import from `internal/` packages. No circular dependencies.

3. **IPC is a notification channel, not the source of truth.** The database is the source of truth. IPC events (`ReminderCreated`, `ReminderDeleted`) are fire-and-forget signals so the daemon can react immediately instead of waiting for its next poll cycle.

4. **Concurrency via goroutines and channels.** No global mutexes. The daemon uses goroutine-per-connection and ticker-based schedulers. The TUI uses a single-goroutine event loop with channel-based communication to the daemon connector.

5. **Extensible for remote sync.** The `Repository` interface and the event system are designed so that a future `RemoteRepository` or sync daemon can be added without restructuring.

---

## Go Project Layout

```
hkb-go/
├── go.mod
├── go.sum
├── Makefile
├── README.md
├── cmd/
│   ├── tui/
│   │   └── main.go              # TUI client entry point
│   ├── daemon/
│   │   └── main.go              # Daemon entry point
│   └── cli/                     # (Phase 7) CLI client
│       └── main.go
├── internal/
│   ├── config/
│   │   └── config.go            # Platform paths, env vars, app config
│   ├── dateutil/
│   │   ├── date.go              # SimpleDate type
│   │   ├── date_test.go
│   │   ├── duration.go          # Duration type + arithmetic
│   │   ├── duration_test.go
│   │   ├── humanparse.go        # Natural language date parser
│   │   ├── humanparse_test.go
│   │   └── grammar.go           # Parser rules (or use a PEG lib)
│   ├── storage/
│   │   ├── repository.go        # Repository interface
│   │   ├── models.go            # DTOs: ReminderData, CreateReminderData, etc.
│   │   ├── query.go             # QueryOption types
│   │   ├── sqlite/
│   │   │   ├── sqlite.go        # SQLite implementation
│   │   │   ├── sqlite_test.go
│   │   │   └── migrations/
│   │   │       └── 001_create_reminders.sql
│   │   └── memory/
│   │       └── memory.go        # In-memory impl for tests
│   ├── ipc/
│   │   ├── server.go            # Unix socket server
│   │   ├── client.go            # Unix socket client
│   │   ├── frame.go             # Binary framing protocol
│   │   ├── frame_test.go
│   │   └── events.go            # Event type definitions
│   ├── notify/
│   │   ├── notifier.go          # OS notification abstraction
│   │   └── audio.go             # Audio playback
│   ├── logger/
│   │   └── logger.go            # Structured logging setup
│   └── tui/
│       ├── app.go               # App state, render loop
│       ├── events.go            # Keyboard event system
│       ├── focus.go             # Focus management
│       ├── bounded.go           # BoundedValue utility
│       ├── views/
│       │   ├── main.go          # Main/home view
│       │   ├── reminders.go     # Reminders router
│       │   ├── reminders_list.go
│       │   └── reminders_create.go
│       └── components/
│           ├── navigation.go    # Tab navigation bar
│           ├── input.go         # Vi-like text input
│           └── button.go        # Clickable button
├── sounds/
│   └── notification.wav
├── migrations/
│   └── 001_create_reminders.sql # Also kept at top-level for tooling
└── .github/
    └── workflows/
        └── pr-checks.yml
```

---

## Prompt Sequence

Each numbered section below is a standalone prompt. Execute them in order. Earlier prompts produce code that later prompts depend on.

---

### Prompt 1 -- Project Scaffold & Configuration

```
Create a new Go project called "hkb-go".

Requirements:
- Initialize go.mod with module path "github.com/<owner>/hkb-go" (use a placeholder owner).
- Create the directory structure: cmd/tui/, cmd/daemon/, internal/config/, internal/logger/.
- Create a Makefile with targets: tui, daemon, lint, test, build.
  - tui: go run ./cmd/tui
  - daemon: go run ./cmd/daemon
  - lint: golangci-lint run ./...
  - test: go test ./...
  - build: go build ./cmd/tui && go build ./cmd/daemon
- Create internal/config/config.go:
  - Detect platform (Linux vs macOS) and resolve XDG-style paths:
    - DataDir: ~/.local/share/hkb (Linux) or ~/Library/Application Support/hkb (macOS)
    - CacheDir: ~/.cache/hkb (Linux) or ~/Library/Caches/hkb (macOS)
  - Expose functions: DataDir(), CacheDir(), DBPath(), SocketPath(), SoundDir().
    - DBPath = DataDir/db.sqlite
    - SocketPath = DataDir/hkb.sock
    - SoundDir = DataDir/sound
  - EnsureDataDir() creates the data directory if it doesn't exist.
- Create internal/logger/logger.go:
  - Use the slog package from the standard library.
  - Init(appenders ...AppenderType) where AppenderType is File or Stdout.
  - File appender writes to CacheDir/main.log in debug, DataDir/main.log in release.
  - Respect HKB_LOG_LEVEL env var (debug, info, warn, error).
  - Provide a package-level Logger() function returning the configured *slog.Logger.
- Create stub main.go files in cmd/tui/ and cmd/daemon/ that just call config.EnsureDataDir() and log a startup message.
- Create a .gitignore ignoring: binaries, __log/, *.sqlite, *.sock.
- Write tests for config path resolution.
```

---

### Prompt 2 -- Date/Time Library

```
Create the package internal/dateutil.

Reference: PROJECT_MINDMAP.md sections 4.1, 4.2, 4.3, and section 11.

Requirements:

1. date.go -- SimpleDate type:
   - Wraps time.Time and a Timezone enum (UTC, Local).
   - Constructors: Now() (UTC), Local(), ParseRFC3339(string), ParseFormat(string, layout).
   - Mutation: SetYear, SetMonth, SetDay, SetHour, SetMinute, SetSecond, SetYMD, SetHMS, SetStartOfDay, SetEndOfDay.
   - Arithmetic: AddDuration(Duration), SubDuration(Duration).
   - Subtraction of two SimpleDates returns time.Duration.
   - Getters: Year, Month, Day, Hour, Minute, Second, Timezone.
   - String() outputs RFC 3339 with seconds precision.
   - Format(layout) for custom formatting.
   - Implements json.Marshaler and json.Unmarshaler using RFC 3339.

2. duration.go -- Duration type:
   - An enum-style type with variants: Minute, Hour, Day, Week, Month, Year, each carrying a uint32 value.
   - FromString(name string, value uint32) factory.
   - Value() returns the inner uint32.
   - String() for display.
   - Calendar-aware Add and Sub on time.Time:
     - Month addition handles year rollover (e.g., Nov + 3 months = Feb next year).
     - Year addition handles overflow.
   - HumanizeStdDuration(d time.Duration) string:
     - Converts to "X days Y hours Z minutes".
     - Singular/plural. Empty string if < 1 minute.

3. humanparse.go -- HumanDateParser:
   - Takes a start date (SimpleDate) at construction.
   - Parse(input string) (SimpleDate, error).
   - Case insensitive.
   - Supports these sentence patterns (see PROJECT_MINDMAP.md section 11 for the full grammar):
     - IN: "In 5 minutes", "In 3 months"
     - IN_ALT: "In two days", "In three days at 5:00"
     - AT: "At 05:00", "At 13:00 on the 11th of December"
     - ON: "On 5th of May", "On the 1st of August at 13:45"
     - NEXT: "Next Monday", "Next Week at 16:00", "Next Month"
     - TOMORROW: "Tomorrow", "Tomorrow at 15:35"
   - Composability: AT can nest ON, ON can nest AT, IN_ALT/NEXT/TOMORROW can nest AT.
   - Auto-rolls to next year when the specified date is in the past relative to start date.
   - "Next [same weekday as today]" adds 7 days, not 0.
   - Implementation approach is up to you: regex, hand-written recursive descent, or a PEG library. Choose whichever produces the cleanest, most maintainable code.

4. Comprehensive tests for all three files. Port the test cases from the original Rust implementation (see PROJECT_MINDMAP.md sections 4.1, 4.2, 4.3 for exact expected values). Test edge cases: year boundaries, month rollover, same-weekday next.
```

---

### Prompt 3 -- Storage Layer (Repository Interface + SQLite)

```
Create the packages internal/storage and internal/storage/sqlite.

Reference: PROJECT_MINDMAP.md sections 5.1, 5.2, 9.

Requirements:

1. internal/storage/models.go -- DTOs:
   - ReminderData: ID int64, Note string, RemindAt dateutil.SimpleDate, CreatedAt dateutil.SimpleDate.
   - CreateReminderData: Note string, RemindAt dateutil.SimpleDate.
   - UpdateReminderData: ID int64, Note *string (nil = no change), RemindAt *dateutil.SimpleDate (nil = no change).
   - All must be JSON-serializable.

2. internal/storage/query.go -- Query options:
   - QueryOption interface or functional option pattern.
   - Options: RemindAtGe(date), RemindAtLe(date), RemindAtBetween(start, end), WithIDs(ids), WithoutIDs(ids).
   - Multiple options are ANDed together.

3. internal/storage/repository.go -- Repository interface:
   - FetchReminders(opts ...QueryOption) ([]ReminderData, error)
   - FetchReminder(id int64) (ReminderData, error)
   - CreateReminder(data CreateReminderData) (ReminderData, error)
   - UpdateReminder(data UpdateReminderData) (ReminderData, error)
   - DeleteReminder(id int64) error
   - DeleteReminders(opts ...QueryOption) error
   - Close() error

   This interface is the extensibility point. Future implementations (Postgres, remote API client) will satisfy it without changing any consumer code.

4. internal/storage/sqlite/sqlite.go -- SQLite implementation:
   - Use modernc.org/sqlite (pure Go, no CGO) or mattn/go-sqlite3.
   - Constructor: New(dbPath string) (Repository, error) -- opens DB, runs migrations, returns.
   - Migrations: create the reminders table (see PROJECT_MINDMAP.md section 9 for schema).
   - Embed migration SQL files using go:embed.
   - Dates stored as RFC 3339 strings, parsed at the service boundary.
   - FetchReminders: build query dynamically from QueryOptions, ORDER BY id ASC.
   - All methods match the Repository interface.

5. internal/storage/memory/memory.go -- In-memory implementation:
   - For testing. Stores reminders in a slice. Auto-increment ID.
   - Implements the full Repository interface.

6. Tests:
   - Write a test suite that takes a Repository and runs the same tests against it.
   - Run the suite against both the SQLite (with :memory:) and in-memory implementations.
   - Test: create, fetch single, fetch all, fetch with filters (date range, ID inclusion/exclusion), update, delete single, delete bulk.
```

---

### Prompt 4 -- IPC Transport Layer

```
Create the package internal/ipc.

Reference: PROJECT_MINDMAP.md sections 6 and 10.

Requirements:

1. events.go -- Event types:
   - Event is an interface or a tagged union (struct with a Type field + payload).
   - ReminderCreated: carries a storage.ReminderData.
   - ReminderDeleted: carries an int64 (reminder ID).
   - Must be JSON-serializable and deserializable.
   - Design the event system to be extensible -- adding new event types should only require adding a new constant/struct, not modifying existing code.

2. frame.go -- Binary framing protocol:
   - Constants: FrameSize = 16384, MetadataSize = 4, DataSize = 16380.
   - Frame struct: Size uint16, FrameNumber uint8, RelatedFrames uint8, Data [16380]byte.
   - FramesFromEvent(event Event) ([]Frame, error) -- serialize event to JSON, split into frames.
   - FrameFromBytes(buf [16384]byte) Frame -- deserialize bytes to Frame (use encoding/binary, NOT unsafe).
   - Frame.ToBytes() [16384]byte -- serialize Frame to bytes.
   - Frame.DataSlice() []byte -- returns only the valid data bytes (0..Size).
   - Frame.GetEvent() (Event, error) -- deserialize the data payload back to an Event.
   - Use little-endian for the size field.

3. server.go -- Unix socket server:
   - Server struct wrapping net.UnixListener.
   - Bind() (Server, error):
     - Resolve socket path from config.SocketPath().
     - Create data dir if missing.
     - Remove stale socket file.
     - Bind and listen.
   - Accept() (net.Conn, error).
   - Addr() string.
   - Close() error.

4. client.go -- Unix socket client:
   - Client struct wrapping net.UnixConn.
   - Connect() (Client, error) -- connect to config.SocketPath().
   - FromConn(net.Conn) Client -- wrap existing connection (for server-side use).
   - ReadEvent() (Event, error) -- read one frame, deserialize.
   - SendEvent(event Event) error -- serialize to frames, write all.
   - QueueEvent(event Event) -- add to internal queue.
   - Flush() error -- send one queued event.
   - Close() error.

5. Tests:
   - Frame round-trip: create event -> frames -> bytes -> frames -> event, verify equality.
   - Multi-frame: create a large payload that spans multiple frames, verify reassembly.
   - Client-server integration: start server in goroutine, connect client, send event, read event, verify.
```

---

### Prompt 5 -- Background Daemon

```
Create cmd/daemon/main.go and supporting packages.

Reference: PROJECT_MINDMAP.md section 7.

Requirements:

1. Startup sequence:
   - config.EnsureDataDir()
   - logger.Init(File, Stdout)
   - Open SQLite database via storage/sqlite.New(config.DBPath())
   - Bind IPC server via ipc.Bind()
   - Start reminder scheduler (goroutine)
   - Start cleanup scheduler (goroutine)
   - Start audio subsystem (goroutine)
   - Accept connections in main loop, spawn goroutine per connection.

2. Connection handling (per goroutine):
   - Wrap accepted conn as ipc.Client via FromConn.
   - Read events in a loop, log them.
   - Flush queued events every 500ms using a time.Ticker.
   - On disconnect (read returns error / EOF), log and exit goroutine.

3. Reminder notification scheduler:
   - Runs every 10 seconds (time.Ticker).
   - Checks 4 intervals relative to now:
     | Start     | End       | Label       |
     |-----------|-----------|-------------|
     | 0 min     | 1 min     | "1 minute"  |
     | 0 min     | 5 min     | "5 minutes" |
     | 6 min     | 15 min    | "15 minutes"|
     | 16 min    | 30 min    | "30 minutes"|
   - For each interval, fetch reminders where remind_at is BETWEEN (now+start, now+end) AND id NOT IN already-reminded set.
   - Send OS notification per reminder: summary "You have a reminder in: {label}", body = reminder note, timeout 3s.
   - Track reminded IDs per interval in a map[string][]int64.
   - After any notifications, trigger audio playback.

4. Reminder cleanup scheduler:
   - Runs every 5 minutes (time.Ticker).
   - Deletes reminders where remind_at <= (now - 24 hours).

5. internal/notify/notifier.go -- OS notifications:
   - Use beeep or go-toast or exec("notify-send") depending on platform.
   - Notify(summary, body string, timeout time.Duration) error.

6. internal/notify/audio.go -- Audio playback:
   - Channel-based architecture: a buffered channel receives file paths.
   - A goroutine listens and plays each file.
   - Play(path string) sends path to channel.
   - Init() starts the listener goroutine.
   - Resolve relative paths against config.SoundDir().
   - Use oto or beep library for audio, or exec an external player (aplay/afplay) -- choose the simplest reliable approach.

7. Graceful shutdown: listen for SIGINT/SIGTERM, close server socket, close DB, exit cleanly.
```

---

### Prompt 6 -- TUI Client

```
Create cmd/tui/main.go and the internal/tui package.

Reference: PROJECT_MINDMAP.md section 8.

Use the bubbletea library (github.com/charmbracelet/bubbletea) for the TUI framework, with lipgloss for styling and bubbles for common components. These are the idiomatic Go TUI libraries.

Requirements:

1. cmd/tui/main.go -- Bootstrap:
   - config.EnsureDataDir()
   - logger.Init(File)
   - Open SQLite database via storage/sqlite.New(config.DBPath())
   - Start daemon connection goroutine (see step 7)
   - Create and run the bubbletea program.

2. internal/tui/app.go -- Root model:
   - Implements bubbletea.Model (Init, Update, View).
   - State: currentView (Main or Reminders), editing bool, ignoreNavigation bool.
   - Delegates Update and View to the active view's sub-model.
   - Global key handling: Ctrl+C quits, Esc exits edit mode.
   - Layout: navigation bar (top), active view (middle), status bar (bottom: "Normal Mode" / "Insert Mode").

3. internal/tui/views/main.go -- Main view:
   - Simple welcome/dashboard view (placeholder content is fine; the Rust version shows an ASCII image, but just showing a welcome message is acceptable).

4. internal/tui/views/reminders.go -- Reminders router:
   - Sub-views: List (default), Create.
   - Routes messages: ChangeView, CreateReminder, DeleteReminder.
   - On CreateReminder: create in DB via Repository, send ReminderCreated event to daemon.
   - On DeleteReminder: delete in DB via Repository, send ReminderDeleted event to daemon.

5. internal/tui/views/reminders_list.go -- List view:
   - Fetches today's reminders (remind_at BETWEEN start_of_today AND end_of_today) and upcoming (remind_at >= end_of_today).
   - Displays two sections: "Today Reminders" (top 30%) and "Upcoming Reminders" (bottom 70%).
   - Format: "{note} - {humanized_duration} ({date DD-MM-YYYY})"
   - Keyboard: j/k to navigate, 'a' to switch to Create, 'dd' (double press d within 300ms) to delete selected.
   - Selection state: clamped index across both lists.

6. internal/tui/views/reminders_create.go -- Create view:
   - Form with two text inputs: Title, Reminder Date.
   - Submit button.
   - Tab/Shift+Tab cycles focus between fields and button.
   - On submit: validate inputs (non-empty, date parseable by dateutil.HumanDateParser), show error if invalid.
   - On successful submit: return CreateReminder message.
   - The input component should support vi-like keybindings:
     - Normal mode: h/l (move), ^/$ (start/end), e/b (word nav), i/I/a/A (enter insert).
     - Insert mode: type characters, backspace, arrow keys, Esc to return to normal.
   - Entering this view sets editing=true and ignoreNavigation=true.
   - Leaving resets both.

7. Daemon connection:
   - A goroutine that connects to the daemon via ipc.Connect().
   - Auto-reconnect every 3 seconds on failure.
   - Exposes a channel or function to send events (QueueEvent + Flush every 500ms).
   - The TUI sends events through this channel when creating/deleting reminders.

8. internal/tui/components/navigation.go:
   - Tab bar across top. Tab key cycles views.
   - Respects ignoreNavigation flag.
   - App title on left, tabs on right.

9. Tests: at minimum, test the dateutil integration (parse user input -> correct date) within the create view logic.
```

---

### Prompt 7 -- CLI Client (Future-Ready Extensibility)

```
Create cmd/cli/main.go.

This is a lightweight command-line client that operates on the same database and communicates with the same daemon. It demonstrates that the Repository interface and IPC layer are reusable across different frontends.

Requirements:

1. Commands (use cobra or the stdlib flag package):
   - hkb list              -- list all reminders (today first, then upcoming)
   - hkb create <note> <date>  -- create a reminder (date is natural language, parsed by dateutil)
   - hkb delete <id>       -- delete a reminder by ID
   - hkb version           -- print version

2. Each command:
   - Opens the SQLite database.
   - Performs the operation via the Repository interface.
   - On create/delete: attempts to notify daemon via IPC (non-fatal if daemon is not running).
   - Outputs results to stdout in a clean, readable format.

3. Tests: test that create + list round-trips correctly using the in-memory repository.
```

---

### Prompt 8 -- CI/CD Pipeline

```
Create .github/workflows/pr-checks.yml.

Requirements:
- Trigger on pull_request (opened, reopened, synchronize).
- Ubuntu latest runner.
- Steps:
  1. Checkout.
  2. Setup Go (latest stable).
  3. Cache go modules and build cache.
  4. Install golangci-lint.
  5. Run: make lint
  6. Run: make test
  7. Run: make build
- Install any system dependencies needed for audio (libasound2-dev on Linux).
```

---

### Prompt 9 -- Sync Extensibility Scaffolding (Design Only)

```
Create a design document at docs/SYNC_DESIGN.md.

This is NOT implementation -- it's the architecture plan for future online sync capability. The goal is to validate that the current Repository interface and event system can support it.

Cover:

1. Remote server architecture:
   - A separate Go binary (cmd/server/) exposing an HTTP or gRPC API.
   - Backed by PostgreSQL (or any remote DB).
   - Endpoints: CRUD for reminders, sync endpoint.

2. Sync strategy:
   - Option A: Last-write-wins with timestamps (simpler).
   - Option B: CRDT-based merge (more robust for offline-first).
   - Recommend one and explain why.

3. Repository adapter:
   - A new implementation of the Repository interface that wraps HTTP/gRPC calls to the remote server.
   - OR a SyncRepository that wraps both a local SQLite repo and a remote repo, handling conflict resolution.

4. Daemon's role in sync:
   - The daemon gains a sync scheduler (periodic push/pull).
   - New IPC event: SyncRequested, SyncCompleted.
   - Daemon can be the only component that talks to the remote server.

5. Schema changes needed:
   - Add updated_at and deleted_at (soft delete) columns.
   - Add a sync_version or vector clock column.
   - Migration path from current schema.

6. What needs NO changes:
   - The TUI client.
   - The CLI client.
   - The IPC protocol (just add new event types).
   - The dateutil package.

This document should confirm that the architecture built in Prompts 1-8 is sync-ready without structural rewrites.
```

---

## Execution Notes

- **Order matters for Prompts 1-6.** Each builds on the previous. Prompts 7-9 can be done in any order after 6.
- **The agent should write tests alongside implementation**, not as an afterthought. Each prompt specifies what to test.
- **Library choices are suggestions, not mandates.** The agent should use whatever produces the cleanest idiomatic Go. The architecture (interfaces, package boundaries, data flow) is what matters.
- **Do not port the experimental features** (Huffman, binary tree, min heap, JPEG decoder). Those were learning exercises in the Rust version.
- **The Repository interface is the most important design decision.** Everything else follows from it. If the agent disagrees with a detail, that's fine -- but the Repository abstraction must remain so that sync can be added later without rewriting consumers.
