package ipc

import (
	"strings"
	"testing"

	"github.com/kaloyan-yosifov/hkb-go/internal/dateutil"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage"
)

func mustDate(s string) dateutil.SimpleDate {
	d, err := dateutil.ParseFormat(s, "2006-01-02 15:04:05")
	if err != nil {
		panic(err)
	}
	return d
}

func TestFrameRoundTrip(t *testing.T) {
	reminder := storage.ReminderData{
		ID:        1,
		Note:      "Test reminder",
		RemindAt:  mustDate("2024-05-01 10:00:00"),
		CreatedAt: mustDate("2024-04-14 08:00:00"),
	}

	event, err := NewReminderCreatedEvent(reminder)
	if err != nil {
		t.Fatalf("create event: %v", err)
	}

	frames, err := FramesFromEvent(event)
	if err != nil {
		t.Fatalf("frames from event: %v", err)
	}

	if len(frames) != 1 {
		t.Fatalf("expected 1 frame, got %d", len(frames))
	}

	// Round-trip through bytes
	buf := frames[0].ToBytes()
	restored := FrameFromBytes(buf)

	if restored.Size != frames[0].Size {
		t.Errorf("size: got %d, want %d", restored.Size, frames[0].Size)
	}
	if restored.FrameNumber != frames[0].FrameNumber {
		t.Errorf("frame number: got %d, want %d", restored.FrameNumber, frames[0].FrameNumber)
	}
	if restored.RelatedFrames != frames[0].RelatedFrames {
		t.Errorf("related frames: got %d, want %d", restored.RelatedFrames, frames[0].RelatedFrames)
	}

	restoredEvent, err := AssembleFrames([]Frame{restored})
	if err != nil {
		t.Fatalf("assemble frames: %v", err)
	}

	restoredReminder, err := restoredEvent.ReminderData()
	if err != nil {
		t.Fatalf("reminder data: %v", err)
	}

	if restoredReminder.ID != reminder.ID {
		t.Errorf("id: got %d, want %d", restoredReminder.ID, reminder.ID)
	}
	if restoredReminder.Note != reminder.Note {
		t.Errorf("note: got %q, want %q", restoredReminder.Note, reminder.Note)
	}
}

func TestDeleteEventRoundTrip(t *testing.T) {
	event, err := NewReminderDeletedEvent(42)
	if err != nil {
		t.Fatalf("create event: %v", err)
	}

	frames, err := FramesFromEvent(event)
	if err != nil {
		t.Fatalf("frames from event: %v", err)
	}

	restored, err := AssembleFrames(frames)
	if err != nil {
		t.Fatalf("assemble: %v", err)
	}

	id, err := restored.ReminderID()
	if err != nil {
		t.Fatalf("reminder id: %v", err)
	}

	if id != 42 {
		t.Errorf("got %d, want 42", id)
	}
}

func TestMultiFrameEvent(t *testing.T) {
	longNote := strings.Repeat("A", FrameDataSize+100)
	reminder := storage.ReminderData{
		ID:        99,
		Note:      longNote,
		RemindAt:  mustDate("2024-05-01 10:00:00"),
		CreatedAt: mustDate("2024-04-14 08:00:00"),
	}

	event, err := NewReminderCreatedEvent(reminder)
	if err != nil {
		t.Fatalf("create event: %v", err)
	}

	frames, err := FramesFromEvent(event)
	if err != nil {
		t.Fatalf("frames from event: %v", err)
	}

	if len(frames) < 2 {
		t.Fatalf("expected multi-frame, got %d frames", len(frames))
	}

	for i, f := range frames {
		if f.FrameNumber != uint8(i+1) {
			t.Errorf("frame %d: number = %d, want %d", i, f.FrameNumber, i+1)
		}
		if f.RelatedFrames != uint8(len(frames)) {
			t.Errorf("frame %d: related = %d, want %d", i, f.RelatedFrames, len(frames))
		}
	}

	// Round-trip through bytes
	var restoredFrames []Frame
	for _, f := range frames {
		buf := f.ToBytes()
		restoredFrames = append(restoredFrames, FrameFromBytes(buf))
	}

	restoredEvent, err := AssembleFrames(restoredFrames)
	if err != nil {
		t.Fatalf("assemble: %v", err)
	}

	restoredReminder, err := restoredEvent.ReminderData()
	if err != nil {
		t.Fatalf("reminder data: %v", err)
	}

	if restoredReminder.Note != longNote {
		t.Errorf("note length: got %d, want %d", len(restoredReminder.Note), len(longNote))
	}
}

func TestFrameDataSlice(t *testing.T) {
	f := Frame{
		Size:          5,
		FrameNumber:   1,
		RelatedFrames: 1,
	}
	copy(f.Data[:], []byte("hello"))

	slice := f.DataSlice()
	if string(slice) != "hello" {
		t.Errorf("DataSlice() = %q, want %q", string(slice), "hello")
	}
	if len(slice) != 5 {
		t.Errorf("DataSlice() len = %d, want 5", len(slice))
	}
}
