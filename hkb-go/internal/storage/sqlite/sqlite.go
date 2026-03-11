package sqlite

import (
	"database/sql"
	"embed"
	"fmt"
	"strings"
	"time"

	"github.com/kaloyan-yosifov/hkb-go/internal/dateutil"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage"

	_ "modernc.org/sqlite"
)

//go:embed migrations/*.sql
var migrationFS embed.FS

type sqliteRepo struct {
	db *sql.DB
}

func New(dbPath string) (storage.Repository, error) {
	db, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return nil, fmt.Errorf("open sqlite: %w", err)
	}

	if err := runMigrations(db); err != nil {
		db.Close()
		return nil, fmt.Errorf("run migrations: %w", err)
	}

	return &sqliteRepo{db: db}, nil
}

func runMigrations(db *sql.DB) error {
	entries, err := migrationFS.ReadDir("migrations")
	if err != nil {
		return fmt.Errorf("read migration dir: %w", err)
	}

	for _, entry := range entries {
		data, err := migrationFS.ReadFile("migrations/" + entry.Name())
		if err != nil {
			return fmt.Errorf("read migration %s: %w", entry.Name(), err)
		}
		if _, err := db.Exec(string(data)); err != nil {
			return fmt.Errorf("exec migration %s: %w", entry.Name(), err)
		}
	}
	return nil
}

func (r *sqliteRepo) Close() error {
	return r.db.Close()
}

func (r *sqliteRepo) FetchReminders(opts ...storage.QueryOption) ([]storage.ReminderData, error) {
	q := storage.BuildQuery(opts...)
	query := "SELECT id, note, remind_at, created_at FROM reminders"
	var conditions []string
	var args []any

	if q.RemindAtBetween != nil {
		conditions = append(conditions, "remind_at BETWEEN ? AND ?")
		args = append(args, q.RemindAtBetween[0].String(), q.RemindAtBetween[1].String())
	}
	if q.RemindAtGe != nil {
		conditions = append(conditions, "remind_at >= ?")
		args = append(args, q.RemindAtGe.String())
	}
	if q.RemindAtLe != nil {
		conditions = append(conditions, "remind_at <= ?")
		args = append(args, q.RemindAtLe.String())
	}
	if len(q.WithIDs) > 0 {
		placeholders := makePlaceholders(len(q.WithIDs))
		conditions = append(conditions, "id IN ("+placeholders+")")
		for _, id := range q.WithIDs {
			args = append(args, id)
		}
	}
	if len(q.WithoutIDs) > 0 {
		placeholders := makePlaceholders(len(q.WithoutIDs))
		conditions = append(conditions, "id NOT IN ("+placeholders+")")
		for _, id := range q.WithoutIDs {
			args = append(args, id)
		}
	}

	if len(conditions) > 0 {
		query += " WHERE " + strings.Join(conditions, " AND ")
	}
	query += " ORDER BY id ASC"

	rows, err := r.db.Query(query, args...)
	if err != nil {
		return nil, fmt.Errorf("fetch reminders: %w", err)
	}
	defer rows.Close()

	var result []storage.ReminderData
	for rows.Next() {
		var id int64
		var note, remindAtStr, createdAtStr string
		if err := rows.Scan(&id, &note, &remindAtStr, &createdAtStr); err != nil {
			return nil, fmt.Errorf("scan reminder: %w", err)
		}
		remindAt, err := dateutil.ParseRFC3339(remindAtStr)
		if err != nil {
			return nil, fmt.Errorf("parse remind_at: %w", err)
		}
		createdAt, err := dateutil.ParseRFC3339(createdAtStr)
		if err != nil {
			return nil, fmt.Errorf("parse created_at: %w", err)
		}
		result = append(result, storage.ReminderData{
			ID:        id,
			Note:      note,
			RemindAt:  remindAt,
			CreatedAt: createdAt,
		})
	}
	return result, rows.Err()
}

func (r *sqliteRepo) FetchReminder(id int64) (storage.ReminderData, error) {
	var note, remindAtStr, createdAtStr string
	err := r.db.QueryRow(
		"SELECT id, note, remind_at, created_at FROM reminders WHERE id = ?", id,
	).Scan(&id, &note, &remindAtStr, &createdAtStr)
	if err != nil {
		return storage.ReminderData{}, fmt.Errorf("fetch reminder %d: %w", id, err)
	}

	remindAt, err := dateutil.ParseRFC3339(remindAtStr)
	if err != nil {
		return storage.ReminderData{}, fmt.Errorf("parse remind_at: %w", err)
	}
	createdAt, err := dateutil.ParseRFC3339(createdAtStr)
	if err != nil {
		return storage.ReminderData{}, fmt.Errorf("parse created_at: %w", err)
	}

	return storage.ReminderData{
		ID:        id,
		Note:      note,
		RemindAt:  remindAt,
		CreatedAt: createdAt,
	}, nil
}

func (r *sqliteRepo) CreateReminder(data storage.CreateReminderData) (storage.ReminderData, error) {
	now := dateutil.FromTime(time.Now().UTC(), dateutil.TimezoneUTC)
	result, err := r.db.Exec(
		"INSERT INTO reminders (note, remind_at, created_at) VALUES (?, ?, ?)",
		data.Note, data.RemindAt.String(), now.String(),
	)
	if err != nil {
		return storage.ReminderData{}, fmt.Errorf("create reminder: %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return storage.ReminderData{}, fmt.Errorf("get last insert id: %w", err)
	}

	return r.FetchReminder(id)
}

func (r *sqliteRepo) UpdateReminder(data storage.UpdateReminderData) (storage.ReminderData, error) {
	var sets []string
	var args []any

	if data.Note != nil {
		sets = append(sets, "note = ?")
		args = append(args, *data.Note)
	}
	if data.RemindAt != nil {
		sets = append(sets, "remind_at = ?")
		args = append(args, data.RemindAt.String())
	}

	if len(sets) == 0 {
		return r.FetchReminder(data.ID)
	}

	args = append(args, data.ID)
	query := "UPDATE reminders SET " + strings.Join(sets, ", ") + " WHERE id = ?"
	if _, err := r.db.Exec(query, args...); err != nil {
		return storage.ReminderData{}, fmt.Errorf("update reminder: %w", err)
	}

	return r.FetchReminder(data.ID)
}

func (r *sqliteRepo) DeleteReminder(id int64) error {
	_, err := r.db.Exec("DELETE FROM reminders WHERE id = ?", id)
	if err != nil {
		return fmt.Errorf("delete reminder %d: %w", id, err)
	}
	return nil
}

func (r *sqliteRepo) DeleteReminders(opts ...storage.QueryOption) error {
	q := storage.BuildQuery(opts...)
	query := "DELETE FROM reminders"
	var conditions []string
	var args []any

	if q.RemindAtBetween != nil {
		conditions = append(conditions, "remind_at BETWEEN ? AND ?")
		args = append(args, q.RemindAtBetween[0].String(), q.RemindAtBetween[1].String())
	}
	if q.RemindAtGe != nil {
		conditions = append(conditions, "remind_at >= ?")
		args = append(args, q.RemindAtGe.String())
	}
	if q.RemindAtLe != nil {
		conditions = append(conditions, "remind_at <= ?")
		args = append(args, q.RemindAtLe.String())
	}
	if len(q.WithIDs) > 0 {
		placeholders := makePlaceholders(len(q.WithIDs))
		conditions = append(conditions, "id IN ("+placeholders+")")
		for _, id := range q.WithIDs {
			args = append(args, id)
		}
	}
	if len(q.WithoutIDs) > 0 {
		placeholders := makePlaceholders(len(q.WithoutIDs))
		conditions = append(conditions, "id NOT IN ("+placeholders+")")
		for _, id := range q.WithoutIDs {
			args = append(args, id)
		}
	}

	if len(conditions) > 0 {
		query += " WHERE " + strings.Join(conditions, " AND ")
	}

	_, err := r.db.Exec(query, args...)
	if err != nil {
		return fmt.Errorf("delete reminders: %w", err)
	}
	return nil
}

func makePlaceholders(n int) string {
	if n <= 0 {
		return ""
	}
	return strings.Repeat("?,", n-1) + "?"
}
