package storage

import (
	"github.com/kaloyan-yosifov/hkb-go/internal/dateutil"
)

type ReminderData struct {
	ID        int64              `json:"id"`
	Note      string             `json:"note"`
	RemindAt  dateutil.SimpleDate `json:"remind_at"`
	CreatedAt dateutil.SimpleDate `json:"created_at"`
}

type CreateReminderData struct {
	Note     string             `json:"note"`
	RemindAt dateutil.SimpleDate `json:"remind_at"`
}

type UpdateReminderData struct {
	ID       int64               `json:"id"`
	Note     *string             `json:"note,omitempty"`
	RemindAt *dateutil.SimpleDate `json:"remind_at,omitempty"`
}
