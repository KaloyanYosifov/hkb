package storage_test

import (
	"testing"

	"github.com/kaloyan-yosifov/hkb-go/internal/dateutil"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage/memory"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage/sqlite"
)

type repoFactory struct {
	name string
	make func(t *testing.T) storage.Repository
}

func factories() []repoFactory {
	return []repoFactory{
		{
			name: "sqlite",
			make: func(t *testing.T) storage.Repository {
				t.Helper()
				repo, err := sqlite.New(":memory:")
				if err != nil {
					t.Fatalf("create sqlite repo: %v", err)
				}
				t.Cleanup(func() { repo.Close() })
				return repo
			},
		},
		{
			name: "memory",
			make: func(t *testing.T) storage.Repository {
				return memory.New()
			},
		},
	}
}

func date(s string) dateutil.SimpleDate {
	d, err := dateutil.ParseFormat(s, "2006-01-02 15:04:05")
	if err != nil {
		panic(err)
	}
	return d
}

func strPtr(s string) *string                     { return &s }
func datePtr(d dateutil.SimpleDate) *dateutil.SimpleDate { return &d }

func TestCreateAndFetch(t *testing.T) {
	for _, f := range factories() {
		t.Run(f.name, func(t *testing.T) {
			repo := f.make(t)

			created, err := repo.CreateReminder(storage.CreateReminderData{
				Note:     "Test reminder",
				RemindAt: date("2024-04-15 10:00:00"),
			})
			if err != nil {
				t.Fatalf("create: %v", err)
			}
			if created.Note != "Test reminder" {
				t.Errorf("note = %q, want %q", created.Note, "Test reminder")
			}
			if created.ID == 0 {
				t.Error("expected non-zero ID")
			}

			fetched, err := repo.FetchReminder(created.ID)
			if err != nil {
				t.Fatalf("fetch: %v", err)
			}
			if fetched.Note != "Test reminder" {
				t.Errorf("note = %q, want %q", fetched.Note, "Test reminder")
			}
		})
	}
}

func TestFetchAllReminders(t *testing.T) {
	for _, f := range factories() {
		t.Run(f.name, func(t *testing.T) {
			repo := f.make(t)

			for i := 0; i < 5; i++ {
				_, err := repo.CreateReminder(storage.CreateReminderData{
					Note:     "Reminder",
					RemindAt: date("2024-04-15 10:00:00"),
				})
				if err != nil {
					t.Fatalf("create %d: %v", i, err)
				}
			}

			all, err := repo.FetchReminders()
			if err != nil {
				t.Fatalf("fetch all: %v", err)
			}
			if len(all) != 5 {
				t.Errorf("got %d reminders, want 5", len(all))
			}
		})
	}
}

func TestUpdateReminder(t *testing.T) {
	for _, f := range factories() {
		t.Run(f.name, func(t *testing.T) {
			repo := f.make(t)

			created, err := repo.CreateReminder(storage.CreateReminderData{
				Note:     "Original",
				RemindAt: date("2024-04-15 10:00:00"),
			})
			if err != nil {
				t.Fatalf("create: %v", err)
			}

			newDate := date("2024-05-01 12:00:00")
			updated, err := repo.UpdateReminder(storage.UpdateReminderData{
				ID:       created.ID,
				Note:     strPtr("Updated"),
				RemindAt: datePtr(newDate),
			})
			if err != nil {
				t.Fatalf("update: %v", err)
			}
			if updated.Note != "Updated" {
				t.Errorf("note = %q, want %q", updated.Note, "Updated")
			}
			if updated.RemindAt.String() != newDate.String() {
				t.Errorf("remind_at = %q, want %q", updated.RemindAt.String(), newDate.String())
			}
		})
	}
}

func TestDeleteReminder(t *testing.T) {
	for _, f := range factories() {
		t.Run(f.name, func(t *testing.T) {
			repo := f.make(t)

			created, err := repo.CreateReminder(storage.CreateReminderData{
				Note:     "To delete",
				RemindAt: date("2024-04-15 10:00:00"),
			})
			if err != nil {
				t.Fatalf("create: %v", err)
			}

			if err := repo.DeleteReminder(created.ID); err != nil {
				t.Fatalf("delete: %v", err)
			}

			all, err := repo.FetchReminders()
			if err != nil {
				t.Fatalf("fetch all: %v", err)
			}
			if len(all) != 0 {
				t.Errorf("got %d reminders after delete, want 0", len(all))
			}
		})
	}
}

func TestFetchWithDateRange(t *testing.T) {
	for _, f := range factories() {
		t.Run(f.name, func(t *testing.T) {
			repo := f.make(t)

			dates := []string{
				"2024-04-10 10:00:00",
				"2024-04-15 10:00:00",
				"2024-04-20 10:00:00",
				"2024-04-25 10:00:00",
			}
			for _, d := range dates {
				_, err := repo.CreateReminder(storage.CreateReminderData{
					Note:     "r",
					RemindAt: date(d),
				})
				if err != nil {
					t.Fatalf("create: %v", err)
				}
			}

			start := date("2024-04-14 00:00:00")
			end := date("2024-04-21 00:00:00")

			results, err := repo.FetchReminders(storage.RemindAtBetween(start, end))
			if err != nil {
				t.Fatalf("fetch between: %v", err)
			}
			if len(results) != 2 {
				t.Errorf("between: got %d, want 2", len(results))
			}

			geResults, err := repo.FetchReminders(storage.RemindAtGe(date("2024-04-20 10:00:00")))
			if err != nil {
				t.Fatalf("fetch ge: %v", err)
			}
			if len(geResults) != 2 {
				t.Errorf("ge: got %d, want 2", len(geResults))
			}

			leResults, err := repo.FetchReminders(storage.RemindAtLe(date("2024-04-15 10:00:00")))
			if err != nil {
				t.Fatalf("fetch le: %v", err)
			}
			if len(leResults) != 2 {
				t.Errorf("le: got %d, want 2", len(leResults))
			}
		})
	}
}

func TestFetchWithIDFilters(t *testing.T) {
	for _, f := range factories() {
		t.Run(f.name, func(t *testing.T) {
			repo := f.make(t)

			var ids []int64
			for i := 0; i < 4; i++ {
				created, err := repo.CreateReminder(storage.CreateReminderData{
					Note:     "r",
					RemindAt: date("2024-04-15 10:00:00"),
				})
				if err != nil {
					t.Fatalf("create: %v", err)
				}
				ids = append(ids, created.ID)
			}

			withResults, err := repo.FetchReminders(storage.WithIDs(ids[0], ids[2]))
			if err != nil {
				t.Fatalf("fetch with IDs: %v", err)
			}
			if len(withResults) != 2 {
				t.Errorf("with IDs: got %d, want 2", len(withResults))
			}

			withoutResults, err := repo.FetchReminders(storage.WithoutIDs(ids[0], ids[2]))
			if err != nil {
				t.Fatalf("fetch without IDs: %v", err)
			}
			if len(withoutResults) != 2 {
				t.Errorf("without IDs: got %d, want 2", len(withoutResults))
			}
		})
	}
}

func TestBulkDelete(t *testing.T) {
	for _, f := range factories() {
		t.Run(f.name, func(t *testing.T) {
			repo := f.make(t)

			var ids []int64
			for i := 0; i < 4; i++ {
				created, err := repo.CreateReminder(storage.CreateReminderData{
					Note:     "r",
					RemindAt: date("2024-04-15 10:00:00"),
				})
				if err != nil {
					t.Fatalf("create: %v", err)
				}
				ids = append(ids, created.ID)
			}

			err := repo.DeleteReminders(storage.WithIDs(ids[0], ids[1]))
			if err != nil {
				t.Fatalf("bulk delete: %v", err)
			}

			all, err := repo.FetchReminders()
			if err != nil {
				t.Fatalf("fetch all: %v", err)
			}
			if len(all) != 2 {
				t.Errorf("after bulk delete: got %d, want 2", len(all))
			}
		})
	}
}

func TestDeleteRemindersWithDateRange(t *testing.T) {
	for _, f := range factories() {
		t.Run(f.name, func(t *testing.T) {
			repo := f.make(t)

			dates := []string{
				"2024-04-10 10:00:00",
				"2024-04-15 10:00:00",
				"2024-04-20 10:00:00",
			}
			for _, d := range dates {
				_, err := repo.CreateReminder(storage.CreateReminderData{
					Note:     "r",
					RemindAt: date(d),
				})
				if err != nil {
					t.Fatalf("create: %v", err)
				}
			}

			err := repo.DeleteReminders(storage.RemindAtLe(date("2024-04-15 10:00:00")))
			if err != nil {
				t.Fatalf("delete le: %v", err)
			}

			all, err := repo.FetchReminders()
			if err != nil {
				t.Fatalf("fetch all: %v", err)
			}
			if len(all) != 1 {
				t.Errorf("after date range delete: got %d, want 1", len(all))
			}
		})
	}
}
