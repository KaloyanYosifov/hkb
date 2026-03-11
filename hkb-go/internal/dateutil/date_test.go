package dateutil

import (
	"encoding/json"
	"testing"
	"time"
)

func mustParse(s string) SimpleDate {
	d, err := ParseFormat(s, "2006-01-02 15:04:05")
	if err != nil {
		panic(err)
	}
	return d
}

func TestStartOfDay(t *testing.T) {
	d := mustParse("2024-04-14 08:00:00")
	if d.Hour() != 8 {
		t.Fatalf("expected hour 8, got %d", d.Hour())
	}

	d = d.StartOfDay()
	if d.Hour() != 0 || d.Minute() != 0 || d.Second() != 0 {
		t.Fatalf("start of day: got %02d:%02d:%02d", d.Hour(), d.Minute(), d.Second())
	}

	d = d.EndOfDay()
	if d.Hour() != 23 || d.Minute() != 59 || d.Second() != 59 {
		t.Fatalf("end of day: got %02d:%02d:%02d", d.Hour(), d.Minute(), d.Second())
	}
}

func TestAddMinuteDuration(t *testing.T) {
	cases := []struct {
		minutes  uint32
		expected string
	}{
		{1, "2024-04-14 08:01:00"},
		{5, "2024-04-14 08:05:00"},
		{30, "2024-04-14 08:30:00"},
		{60, "2024-04-14 09:00:00"},
		{1440, "2024-04-15 08:00:00"},
	}

	for _, tc := range cases {
		d := mustParse("2024-04-14 08:00:00")
		result := d.AddDuration(Minute(tc.minutes))
		expected := mustParse(tc.expected)
		if !result.Time().Equal(expected.Time()) {
			t.Errorf("AddDuration(Minute(%d)): got %v, want %v", tc.minutes, result.Time(), expected.Time())
		}
	}
}

func TestAddHourDuration(t *testing.T) {
	cases := []struct {
		hours    uint32
		expected string
	}{
		{1, "2024-04-14 09:00:00"},
		{5, "2024-04-14 13:00:00"},
		{24, "2024-04-15 08:00:00"},
	}

	for _, tc := range cases {
		d := mustParse("2024-04-14 08:00:00")
		result := d.AddDuration(Hour(tc.hours))
		expected := mustParse(tc.expected)
		if !result.Time().Equal(expected.Time()) {
			t.Errorf("AddDuration(Hour(%d)): got %v, want %v", tc.hours, result.Time(), expected.Time())
		}
	}
}

func TestAddMonthDuration(t *testing.T) {
	cases := []struct {
		months   uint32
		expected string
	}{
		{1, "2024-05-14 08:00:00"},
		{3, "2024-07-14 08:00:00"},
		{9, "2025-01-14 08:00:00"},
		{12, "2025-04-14 08:00:00"},
	}

	for _, tc := range cases {
		d := mustParse("2024-04-14 08:00:00")
		result := d.AddDuration(Month(tc.months))
		expected := mustParse(tc.expected)
		if !result.Time().Equal(expected.Time()) {
			t.Errorf("AddDuration(Month(%d)): got %v, want %v", tc.months, result.Time(), expected.Time())
		}
	}
}

func TestSubMinuteDuration(t *testing.T) {
	cases := []struct {
		minutes  uint32
		expected string
	}{
		{1, "2024-04-14 07:59:00"},
		{60, "2024-04-14 07:00:00"},
		{1440, "2024-04-13 08:00:00"},
	}

	for _, tc := range cases {
		d := mustParse("2024-04-14 08:00:00")
		result := d.SubDuration(Minute(tc.minutes))
		expected := mustParse(tc.expected)
		if !result.Time().Equal(expected.Time()) {
			t.Errorf("SubDuration(Minute(%d)): got %v, want %v", tc.minutes, result.Time(), expected.Time())
		}
	}
}

func TestSubMonthDuration(t *testing.T) {
	cases := []struct {
		months   uint32
		expected string
	}{
		{1, "2024-03-14 08:00:00"},
		{4, "2023-12-14 08:00:00"},
		{12, "2023-04-14 08:00:00"},
	}

	for _, tc := range cases {
		d := mustParse("2024-04-14 08:00:00")
		result := d.SubDuration(Month(tc.months))
		expected := mustParse(tc.expected)
		if !result.Time().Equal(expected.Time()) {
			t.Errorf("SubDuration(Month(%d)): got %v, want %v", tc.months, result.Time(), expected.Time())
		}
	}
}

func TestFormat(t *testing.T) {
	d := mustParse("2024-04-14 08:00:00")
	got := d.Format("2006-01-02")
	if got != "2024-04-14" {
		t.Errorf("Format: got %q, want %q", got, "2024-04-14")
	}
}

func TestJSONRoundTrip(t *testing.T) {
	original := mustParse("2024-04-14 08:00:00")

	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}

	var restored SimpleDate
	if err := json.Unmarshal(data, &restored); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}

	if !original.Time().Equal(restored.Time()) {
		t.Errorf("round trip: got %v, want %v", restored.Time(), original.Time())
	}
}

func TestSubSimpleDates(t *testing.T) {
	a := mustParse("2024-04-14 09:00:00")
	b := mustParse("2024-04-14 08:00:00")
	diff := a.Sub(b)
	if diff != time.Hour {
		t.Errorf("Sub: got %v, want %v", diff, time.Hour)
	}

	reverseDiff := b.Sub(a)
	if reverseDiff != 0 {
		t.Errorf("reverse Sub: got %v, want 0", reverseDiff)
	}
}

func TestWeekDuration(t *testing.T) {
	d := mustParse("2024-04-14 08:00:00")
	result := d.AddDuration(Week(1))
	expected := mustParse("2024-04-21 08:00:00")
	if !result.Time().Equal(expected.Time()) {
		t.Errorf("AddDuration(Week(1)): got %v, want %v", result.Time(), expected.Time())
	}

	result = d.SubDuration(Week(1))
	expected = mustParse("2024-04-07 08:00:00")
	if !result.Time().Equal(expected.Time()) {
		t.Errorf("SubDuration(Week(1)): got %v, want %v", result.Time(), expected.Time())
	}
}

func TestYearDuration(t *testing.T) {
	d := mustParse("2024-04-14 08:00:00")
	result := d.AddDuration(Year(1))
	expected := mustParse("2025-04-14 08:00:00")
	if !result.Time().Equal(expected.Time()) {
		t.Errorf("AddDuration(Year(1)): got %v, want %v", result.Time(), expected.Time())
	}

	result = d.SubDuration(Year(1))
	expected = mustParse("2023-04-14 08:00:00")
	if !result.Time().Equal(expected.Time()) {
		t.Errorf("SubDuration(Year(1)): got %v, want %v", result.Time(), expected.Time())
	}
}
