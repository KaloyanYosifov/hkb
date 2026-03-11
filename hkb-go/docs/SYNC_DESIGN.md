# HKB Sync Design Document

> Architecture plan for adding online synchronization to HKB. This document validates that the current codebase can support sync without structural rewrites.

---

## 1. Remote Server Architecture

A new binary `cmd/server/main.go` exposes an HTTP API backed by PostgreSQL.

```
hkb-go/
├── cmd/server/
│   └── main.go
├── internal/
│   ├── storage/
│   │   └── postgres/
│   │       └── postgres.go    # implements storage.Repository
│   └── api/
│       ├── server.go          # HTTP server, routes, middleware
│       ├── handlers.go        # CRUD handlers
│       └── auth.go            # Token-based auth
```

**Endpoints:**

| Method | Path | Description |
|--------|------|-------------|
| GET | /api/reminders | List reminders (with query params for filters) |
| GET | /api/reminders/:id | Get single reminder |
| POST | /api/reminders | Create reminder |
| PUT | /api/reminders/:id | Update reminder |
| DELETE | /api/reminders/:id | Delete reminder |
| POST | /api/sync | Sync endpoint (push local changes, pull remote changes) |

The server implements the same `storage.Repository` interface against PostgreSQL. All existing query options (`RemindAtGe`, `RemindAtLe`, `WithIDs`, etc.) translate directly to SQL.

---

## 2. Sync Strategy

### Recommended: Last-Write-Wins with Timestamps (Option A)

**Why this over CRDTs:**
- HKB reminders are simple CRUD records, not collaborative documents
- A single user across devices is the primary use case
- Conflict resolution semantics are straightforward (latest edit wins)
- Significantly simpler to implement and debug
- CRDTs add complexity that is not justified for this data model

**How it works:**
1. Every record gets an `updated_at` timestamp and a monotonic `sync_version` counter
2. On sync, client sends all records with `sync_version > last_synced_version`
3. Server compares `updated_at` -- the newer write wins
4. Server returns all records updated since the client's last sync
5. Client applies remote changes (skipping any where local `updated_at` is newer)

**Conflict edge case:** If the same reminder is edited on two devices within the same second, the server's copy wins. This is acceptable for a personal productivity tool.

---

## 3. Repository Adapter Options

### Option A: Remote Repository (thin HTTP client)

```go
// internal/storage/remote/remote.go
type remoteRepo struct {
    baseURL string
    token   string
    client  *http.Client
}
```

This implements `storage.Repository` by translating each method into an HTTP call. It can be used directly by the daemon or CLI if they should operate against the remote server only.

### Option B: Sync Repository (recommended)

```go
// internal/storage/sync/sync.go
type syncRepo struct {
    local  storage.Repository   // SQLite
    remote storage.Repository   // remote HTTP
}
```

This wraps both local and remote repositories:
- Reads always hit local (for offline-first, low-latency reads)
- Writes go to local first, then are queued for remote sync
- A background sync loop periodically reconciles local and remote

**This is the recommended approach** because it preserves offline-first behavior while enabling sync.

---

## 4. Daemon's Role in Sync

The daemon is the **only component** that communicates with the remote server. This avoids auth token management in the TUI/CLI.

**New scheduler:** `syncScheduler` runs every 60 seconds (configurable):
1. Fetch local changes since last sync (`WHERE sync_version > last_synced`)
2. Push to remote via `/api/sync`
3. Apply remote changes to local DB
4. Update `last_synced_version`

**New IPC events:**

| Event | Direction | Purpose |
|-------|-----------|---------|
| `SyncRequested` | Client → Daemon | Trigger immediate sync |
| `SyncCompleted` | Daemon → Client | Notify client to refresh data |
| `SyncFailed` | Daemon → Client | Notify client of sync error |

Adding these events requires only new constants in `internal/ipc/events.go` -- no structural changes to the frame protocol or client/server.

---

## 5. Schema Changes

### New columns on `reminders`:

```sql
ALTER TABLE reminders ADD COLUMN updated_at TEXT NOT NULL DEFAULT '';
ALTER TABLE reminders ADD COLUMN deleted_at TEXT;
ALTER TABLE reminders ADD COLUMN sync_version INTEGER NOT NULL DEFAULT 0;
```

| Column | Purpose |
|--------|---------|
| `updated_at` | Timestamp of last modification (for conflict resolution) |
| `deleted_at` | Soft delete timestamp (NULL = active, non-NULL = deleted) |
| `sync_version` | Monotonically increasing counter, bumped on every local write |

### New metadata table:

```sql
CREATE TABLE sync_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

Stores `last_synced_version` and `last_sync_time`.

### Migration path:
- New migration file `002_add_sync_columns.sql` in the existing embedded migrations directory
- Existing data gets `updated_at = created_at`, `sync_version = 0`, `deleted_at = NULL`
- No data loss, fully backward compatible

### Impact on `storage.Repository`:
- `FetchReminders` adds implicit `WHERE deleted_at IS NULL` filter
- `DeleteReminder` becomes a soft delete (sets `deleted_at`)
- `CreateReminder` and `UpdateReminder` bump `sync_version` and set `updated_at`
- A new `PurgeDeletedReminders()` method for post-sync cleanup

---

## 6. What Needs NO Changes

| Component | Why it's unaffected |
|-----------|-------------------|
| **TUI client** (`internal/tui/`) | Talks to `storage.Repository` -- doesn't know or care if it's local SQLite or sync-enabled |
| **CLI client** (`cmd/cli/`) | Same -- uses `storage.Repository` interface |
| **IPC protocol** (`internal/ipc/`) | Frame format is unchanged; new events are just new constants |
| **Date/time library** (`internal/dateutil/`) | Pure utility, no storage dependency |
| **Logger** (`internal/logger/`) | No changes needed |
| **Config** (`internal/config/`) | May add `SyncServerURL()` and `SyncToken()` accessors, but existing functions are untouched |

**This confirms the architecture is sync-ready.** The `Repository` interface is the key abstraction that makes this possible -- all consumers depend on the interface, not on SQLite directly.

---

## 7. Implementation Order (when the time comes)

1. Add schema migration for sync columns
2. Update SQLite repo to handle `updated_at`, `deleted_at`, `sync_version`
3. Build `internal/storage/remote/` (HTTP client implementing `Repository`)
4. Build `internal/storage/sync/` (wrapping local + remote)
5. Build `cmd/server/` with PostgreSQL backend
6. Add sync scheduler to daemon
7. Add new IPC events
8. Add `hkb sync` CLI command for manual trigger
