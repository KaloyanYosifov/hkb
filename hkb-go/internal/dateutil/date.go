package dateutil

import (
	"encoding/json"
	"fmt"
	"time"
)

type Timezone int

const (
	TimezoneUTC   Timezone = iota
	TimezoneLocal
)

type SimpleDate struct {
	t  time.Time
	tz Timezone
}

func Now() SimpleDate {
	return SimpleDate{t: time.Now().UTC(), tz: TimezoneUTC}
}

func Local() SimpleDate {
	return SimpleDate{t: time.Now(), tz: TimezoneLocal}
}

func FromTime(t time.Time, tz Timezone) SimpleDate {
	return SimpleDate{t: t, tz: tz}
}

func ParseRFC3339(s string) (SimpleDate, error) {
	t, err := time.Parse(time.RFC3339, s)
	if err != nil {
		return SimpleDate{}, fmt.Errorf("parse RFC3339: %w", err)
	}
	return SimpleDate{t: t.UTC(), tz: TimezoneUTC}, nil
}

func ParseFormat(s, layout string) (SimpleDate, error) {
	t, err := time.Parse(layout, s)
	if err != nil {
		return SimpleDate{}, fmt.Errorf("parse format: %w", err)
	}
	return SimpleDate{t: t.UTC(), tz: TimezoneUTC}, nil
}

func (d SimpleDate) Year() int          { return d.t.Year() }
func (d SimpleDate) Month() time.Month  { return d.t.Month() }
func (d SimpleDate) Day() int           { return d.t.Day() }
func (d SimpleDate) Hour() int          { return d.t.Hour() }
func (d SimpleDate) Minute() int        { return d.t.Minute() }
func (d SimpleDate) Second() int        { return d.t.Second() }
func (d SimpleDate) Timezone() Timezone { return d.tz }
func (d SimpleDate) Time() time.Time    { return d.t }

func (d SimpleDate) SetYear(year int) SimpleDate {
	d.t = time.Date(year, d.t.Month(), d.t.Day(), d.t.Hour(), d.t.Minute(), d.t.Second(), 0, d.t.Location())
	return d
}

func (d SimpleDate) SetMonth(month int) SimpleDate {
	d.t = time.Date(d.t.Year(), time.Month(month), d.t.Day(), d.t.Hour(), d.t.Minute(), d.t.Second(), 0, d.t.Location())
	return d
}

func (d SimpleDate) SetDay(day int) SimpleDate {
	d.t = time.Date(d.t.Year(), d.t.Month(), day, d.t.Hour(), d.t.Minute(), d.t.Second(), 0, d.t.Location())
	return d
}

func (d SimpleDate) SetHour(hour int) SimpleDate {
	d.t = time.Date(d.t.Year(), d.t.Month(), d.t.Day(), hour, d.t.Minute(), d.t.Second(), 0, d.t.Location())
	return d
}

func (d SimpleDate) SetMinute(minute int) SimpleDate {
	d.t = time.Date(d.t.Year(), d.t.Month(), d.t.Day(), d.t.Hour(), minute, d.t.Second(), 0, d.t.Location())
	return d
}

func (d SimpleDate) SetSecond(second int) SimpleDate {
	d.t = time.Date(d.t.Year(), d.t.Month(), d.t.Day(), d.t.Hour(), d.t.Minute(), second, 0, d.t.Location())
	return d
}

func (d SimpleDate) SetYMD(year, month, day int) SimpleDate {
	d.t = time.Date(year, time.Month(month), day, d.t.Hour(), d.t.Minute(), d.t.Second(), 0, d.t.Location())
	return d
}

func (d SimpleDate) SetHMS(hour, minute, second int) SimpleDate {
	d.t = time.Date(d.t.Year(), d.t.Month(), d.t.Day(), hour, minute, second, 0, d.t.Location())
	return d
}

func (d SimpleDate) StartOfDay() SimpleDate {
	return d.SetHMS(0, 0, 0)
}

func (d SimpleDate) EndOfDay() SimpleDate {
	return d.SetHMS(23, 59, 59)
}

func (d SimpleDate) AddDuration(dur Duration) SimpleDate {
	d.t = dur.AddTo(d.t)
	return d
}

func (d SimpleDate) SubDuration(dur Duration) SimpleDate {
	d.t = dur.SubFrom(d.t)
	return d
}

func (d SimpleDate) Sub(other SimpleDate) time.Duration {
	diff := d.t.Sub(other.t)
	if diff < 0 {
		return 0
	}
	return diff
}

func (d SimpleDate) Format(layout string) string {
	return d.t.Format(layout)
}

func (d SimpleDate) String() string {
	return d.t.UTC().Format(time.RFC3339)
}

func (d SimpleDate) Equal(other SimpleDate) bool {
	return d.t.Equal(other.t)
}

func (d SimpleDate) Before(other SimpleDate) bool {
	return d.t.Before(other.t)
}

func (d SimpleDate) After(other SimpleDate) bool {
	return d.t.After(other.t)
}

func (d SimpleDate) IsZero() bool {
	return d.t.IsZero()
}

func (d SimpleDate) Weekday() time.Weekday {
	return d.t.Weekday()
}

func (d SimpleDate) MarshalJSON() ([]byte, error) {
	return json.Marshal(d.String())
}

func (d *SimpleDate) UnmarshalJSON(data []byte) error {
	var s string
	if err := json.Unmarshal(data, &s); err != nil {
		return err
	}
	parsed, err := ParseRFC3339(s)
	if err != nil {
		return err
	}
	*d = parsed
	return nil
}
