package dateutil

import (
	"testing"
	"time"
)

func TestHumanizeDuration(t *testing.T) {
	cases := []struct {
		seconds  int
		expected string
	}{
		{86400, "1 day"},
		{86400 + 60, "1 day 1 minute"},
		{86400 * 2, "2 days"},
		{(86400 * 2) + (3600 * 2) + 125, "2 days 2 hours 2 minutes"},
		{3600, "1 hour"},
		{3600 + 60, "1 hour 1 minute"},
		{3600 * 2, "2 hours"},
		{3600*2 + 60*2, "2 hours 2 minutes"},
		{60, "1 minute"},
		{60 * 2, "2 minutes"},
		{60 * 59, "59 minutes"},
		{59, ""},
	}

	for _, tc := range cases {
		d := time.Duration(tc.seconds) * time.Second
		got := HumanizeDuration(d)
		if got != tc.expected {
			t.Errorf("HumanizeDuration(%ds) = %q, want %q", tc.seconds, got, tc.expected)
		}
	}
}

func TestDurationString(t *testing.T) {
	cases := []struct {
		d        Duration
		expected string
	}{
		{Minute(5), "Minute-5"},
		{Hour(1), "Hour-1"},
		{Day(3), "Day-3"},
		{Week(2), "Week-2"},
		{Month(6), "Month-6"},
		{Year(10), "Year-10"},
	}

	for _, tc := range cases {
		got := tc.d.String()
		if got != tc.expected {
			t.Errorf("Duration.String() = %q, want %q", got, tc.expected)
		}
	}
}

func TestDurationFromString(t *testing.T) {
	validCases := []struct {
		name  string
		value uint32
		typ   DurationType
	}{
		{"minute", 5, DurationMinute},
		{"hour", 1, DurationHour},
		{"day", 3, DurationDay},
		{"week", 2, DurationWeek},
		{"month", 6, DurationMonth},
		{"year", 10, DurationYear},
	}

	for _, tc := range validCases {
		d, err := DurationFromString(tc.name, tc.value)
		if err != nil {
			t.Errorf("DurationFromString(%q, %d) unexpected error: %v", tc.name, tc.value, err)
			continue
		}
		if d.Type != tc.typ || d.Value != tc.value {
			t.Errorf("DurationFromString(%q, %d) = {%d, %d}, want {%d, %d}", tc.name, tc.value, d.Type, d.Value, tc.typ, tc.value)
		}
	}

	_, err := DurationFromString("invalid", 1)
	if err == nil {
		t.Error("DurationFromString(\"invalid\", 1) expected error, got nil")
	}
}
