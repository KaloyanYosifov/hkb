package ipc

import (
	"encoding/json"
	"fmt"

	"github.com/kaloyan-yosifov/hkb-go/internal/storage"
)

type EventType string

const (
	EventReminderCreated EventType = "ReminderCreated"
	EventReminderDeleted EventType = "ReminderDeleted"
)

type Event struct {
	Type    EventType       `json:"type"`
	Payload json.RawMessage `json:"payload"`
}

func NewReminderCreatedEvent(r storage.ReminderData) (Event, error) {
	data, err := json.Marshal(r)
	if err != nil {
		return Event{}, fmt.Errorf("marshal reminder: %w", err)
	}
	return Event{Type: EventReminderCreated, Payload: data}, nil
}

func NewReminderDeletedEvent(id int64) (Event, error) {
	data, err := json.Marshal(id)
	if err != nil {
		return Event{}, fmt.Errorf("marshal id: %w", err)
	}
	return Event{Type: EventReminderDeleted, Payload: data}, nil
}

func (e Event) ReminderData() (storage.ReminderData, error) {
	if e.Type != EventReminderCreated {
		return storage.ReminderData{}, fmt.Errorf("event type is %s, not ReminderCreated", e.Type)
	}
	var r storage.ReminderData
	if err := json.Unmarshal(e.Payload, &r); err != nil {
		return storage.ReminderData{}, fmt.Errorf("unmarshal reminder: %w", err)
	}
	return r, nil
}

func (e Event) ReminderID() (int64, error) {
	if e.Type != EventReminderDeleted {
		return 0, fmt.Errorf("event type is %s, not ReminderDeleted", e.Type)
	}
	var id int64
	if err := json.Unmarshal(e.Payload, &id); err != nil {
		return 0, fmt.Errorf("unmarshal id: %w", err)
	}
	return id, nil
}

func MarshalEvent(e Event) ([]byte, error) {
	return json.Marshal(e)
}

func UnmarshalEvent(data []byte) (Event, error) {
	var e Event
	if err := json.Unmarshal(data, &e); err != nil {
		return Event{}, fmt.Errorf("unmarshal event: %w", err)
	}
	return e, nil
}
