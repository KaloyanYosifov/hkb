package memory

import (
	"fmt"
	"sync"
	"time"

	"github.com/kaloyan-yosifov/hkb-go/internal/dateutil"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage"
)

type memoryRepo struct {
	mu        sync.Mutex
	reminders []storage.ReminderData
	nextID    int64
}

func New() storage.Repository {
	return &memoryRepo{nextID: 1}
}

func (r *memoryRepo) Close() error {
	return nil
}

func (r *memoryRepo) FetchReminders(opts ...storage.QueryOption) ([]storage.ReminderData, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	q := storage.BuildQuery(opts...)
	var result []storage.ReminderData

	for _, rem := range r.reminders {
		if matchesQuery(rem, q) {
			result = append(result, rem)
		}
	}

	return result, nil
}

func (r *memoryRepo) FetchReminder(id int64) (storage.ReminderData, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	for _, rem := range r.reminders {
		if rem.ID == id {
			return rem, nil
		}
	}
	return storage.ReminderData{}, fmt.Errorf("reminder %d not found", id)
}

func (r *memoryRepo) CreateReminder(data storage.CreateReminderData) (storage.ReminderData, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	rem := storage.ReminderData{
		ID:        r.nextID,
		Note:      data.Note,
		RemindAt:  data.RemindAt,
		CreatedAt: dateutil.FromTime(time.Now().UTC(), dateutil.TimezoneUTC),
	}
	r.nextID++
	r.reminders = append(r.reminders, rem)

	return rem, nil
}

func (r *memoryRepo) UpdateReminder(data storage.UpdateReminderData) (storage.ReminderData, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	for i, rem := range r.reminders {
		if rem.ID == data.ID {
			if data.Note != nil {
				r.reminders[i].Note = *data.Note
			}
			if data.RemindAt != nil {
				r.reminders[i].RemindAt = *data.RemindAt
			}
			return r.reminders[i], nil
		}
	}
	return storage.ReminderData{}, fmt.Errorf("reminder %d not found", data.ID)
}

func (r *memoryRepo) DeleteReminder(id int64) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	for i, rem := range r.reminders {
		if rem.ID == id {
			r.reminders = append(r.reminders[:i], r.reminders[i+1:]...)
			return nil
		}
	}
	return fmt.Errorf("reminder %d not found", id)
}

func (r *memoryRepo) DeleteReminders(opts ...storage.QueryOption) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	q := storage.BuildQuery(opts...)
	var remaining []storage.ReminderData
	for _, rem := range r.reminders {
		if !matchesQuery(rem, q) {
			remaining = append(remaining, rem)
		}
	}
	r.reminders = remaining
	return nil
}

func matchesQuery(rem storage.ReminderData, q storage.Query) bool {
	remindAtStr := rem.RemindAt.String()

	if q.RemindAtBetween != nil {
		start := q.RemindAtBetween[0].String()
		end := q.RemindAtBetween[1].String()
		if remindAtStr < start || remindAtStr > end {
			return false
		}
	}
	if q.RemindAtGe != nil {
		if remindAtStr < q.RemindAtGe.String() {
			return false
		}
	}
	if q.RemindAtLe != nil {
		if remindAtStr > q.RemindAtLe.String() {
			return false
		}
	}
	if len(q.WithIDs) > 0 {
		found := false
		for _, id := range q.WithIDs {
			if rem.ID == id {
				found = true
				break
			}
		}
		if !found {
			return false
		}
	}
	if len(q.WithoutIDs) > 0 {
		for _, id := range q.WithoutIDs {
			if rem.ID == id {
				return false
			}
		}
	}
	return true
}
