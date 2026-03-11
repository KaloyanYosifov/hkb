package dateutil

import (
	"fmt"
	"strings"
	"time"
)

type DurationType int

const (
	DurationMinute DurationType = iota
	DurationHour
	DurationDay
	DurationWeek
	DurationMonth
	DurationYear
)

type Duration struct {
	Type  DurationType
	Value uint32
}

func Minute(v uint32) Duration  { return Duration{Type: DurationMinute, Value: v} }
func Hour(v uint32) Duration    { return Duration{Type: DurationHour, Value: v} }
func Day(v uint32) Duration     { return Duration{Type: DurationDay, Value: v} }
func Week(v uint32) Duration    { return Duration{Type: DurationWeek, Value: v} }
func Month(v uint32) Duration   { return Duration{Type: DurationMonth, Value: v} }
func Year(v uint32) Duration    { return Duration{Type: DurationYear, Value: v} }

func DurationFromString(name string, value uint32) (Duration, error) {
	switch strings.ToLower(name) {
	case "second":
		return Duration{Type: DurationMinute, Value: 0}, fmt.Errorf("second duration not supported")
	case "minute":
		return Minute(value), nil
	case "hour":
		return Hour(value), nil
	case "day":
		return Day(value), nil
	case "week":
		return Week(value), nil
	case "month":
		return Month(value), nil
	case "year":
		return Year(value), nil
	default:
		return Duration{}, fmt.Errorf("invalid duration type: %s", name)
	}
}

func (d Duration) String() string {
	var name string
	switch d.Type {
	case DurationMinute:
		name = "Minute"
	case DurationHour:
		name = "Hour"
	case DurationDay:
		name = "Day"
	case DurationWeek:
		name = "Week"
	case DurationMonth:
		name = "Month"
	case DurationYear:
		name = "Year"
	}
	return fmt.Sprintf("%s-%d", name, d.Value)
}

func (d Duration) AddTo(t time.Time) time.Time {
	v := int(d.Value)
	switch d.Type {
	case DurationMinute:
		return t.Add(time.Duration(v) * time.Minute)
	case DurationHour:
		return t.Add(time.Duration(v) * time.Hour)
	case DurationDay:
		return t.AddDate(0, 0, v)
	case DurationWeek:
		return t.AddDate(0, 0, v*7)
	case DurationMonth:
		return t.AddDate(0, v, 0)
	case DurationYear:
		return t.AddDate(v, 0, 0)
	}
	return t
}

func (d Duration) SubFrom(t time.Time) time.Time {
	v := int(d.Value)
	switch d.Type {
	case DurationMinute:
		return t.Add(-time.Duration(v) * time.Minute)
	case DurationHour:
		return t.Add(-time.Duration(v) * time.Hour)
	case DurationDay:
		return t.AddDate(0, 0, -v)
	case DurationWeek:
		return t.AddDate(0, 0, -v*7)
	case DurationMonth:
		return t.AddDate(0, -v, 0)
	case DurationYear:
		return t.AddDate(-v, 0, 0)
	}
	return t
}

const (
	secondsPerMinute = 60
	secondsPerHour   = 60 * secondsPerMinute
	secondsPerDay    = 24 * secondsPerHour
)

func HumanizeDuration(d time.Duration) string {
	totalSeconds := int(d.Seconds())
	if totalSeconds < secondsPerMinute {
		return ""
	}

	units := []struct {
		divisor int
		single  string
		plural  string
	}{
		{secondsPerDay, "day", "days"},
		{secondsPerHour, "hour", "hours"},
		{secondsPerMinute, "minute", "minutes"},
	}

	var parts []string
	remaining := totalSeconds
	for _, u := range units {
		count := remaining / u.divisor
		if count > 0 {
			remaining %= u.divisor
			name := u.single
			if count >= 2 {
				name = u.plural
			}
			parts = append(parts, fmt.Sprintf("%d %s", count, name))
		}
	}

	return strings.Join(parts, " ")
}
