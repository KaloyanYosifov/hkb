package dateutil

import (
	"fmt"
	"strconv"
	"strings"
	"time"
)

var daysOfWeek = []string{
	"monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
}

var months = []string{
	"january", "february", "march", "april", "may", "june",
	"july", "august", "september", "october", "november", "december",
}

var cardinals = map[string]int{
	"two": 2, "three": 3, "four": 4, "five": 5,
	"six": 6, "seven": 7, "eight": 8, "nine": 9,
}

type HumanDateParser struct {
	start SimpleDate
}

func NewHumanDateParser(start SimpleDate) *HumanDateParser {
	return &HumanDateParser{start: start}
}

func (p *HumanDateParser) Parse(input string) (SimpleDate, error) {
	s := strings.TrimSpace(strings.ToLower(input))
	if s == "" {
		return SimpleDate{}, fmt.Errorf("empty input")
	}

	if strings.HasPrefix(s, "in ") {
		rest := s[3:]
		if d, ok := p.tryParseInAlt(rest); ok {
			return d, nil
		}
		return p.parseIn(rest)
	}
	if strings.HasPrefix(s, "at ") {
		return p.parseAt(s[3:])
	}
	if strings.HasPrefix(s, "on ") {
		return p.parseOn(s[3:])
	}
	if strings.HasPrefix(s, "next ") {
		return p.parseNext(s[5:])
	}
	if strings.HasPrefix(s, "tomorrow") {
		return p.parseTomorrow(s[8:])
	}

	return SimpleDate{}, fmt.Errorf("unable to parse: %q", input)
}

// "in 5 minutes", "in 3 months"
func (p *HumanDateParser) parseIn(rest string) (SimpleDate, error) {
	parts := strings.Fields(rest)
	if len(parts) < 2 {
		return SimpleDate{}, fmt.Errorf("invalid 'in' sentence: %q", rest)
	}

	value, err := strconv.ParseUint(parts[0], 10, 32)
	if err != nil {
		return SimpleDate{}, fmt.Errorf("invalid number in 'in' sentence: %q", parts[0])
	}

	durName := strings.TrimSuffix(parts[1], "s")
	dur, err := DurationFromString(durName, uint32(value))
	if err != nil {
		return SimpleDate{}, err
	}

	return p.start.AddDuration(dur), nil
}

// "two days", "three days at 5:00"
func (p *HumanDateParser) tryParseInAlt(rest string) (SimpleDate, bool) {
	parts := strings.Fields(rest)
	if len(parts) < 2 {
		return SimpleDate{}, false
	}

	days, ok := cardinals[parts[0]]
	if !ok {
		return SimpleDate{}, false
	}

	if parts[1] != "days" {
		return SimpleDate{}, false
	}

	date := p.start.AddDuration(Day(uint32(days)))

	if len(parts) >= 4 && parts[2] == "at" {
		atDate, err := p.parseAtTime(parts[3], date)
		if err != nil {
			return SimpleDate{}, false
		}
		return atDate, true
	}

	return date, true
}

// "05:00", "13:00 on the 11th of december"
func (p *HumanDateParser) parseAt(rest string) (SimpleDate, error) {
	parts := strings.Fields(rest)
	if len(parts) < 1 {
		return SimpleDate{}, fmt.Errorf("invalid 'at' sentence")
	}

	timeStr := parts[0]
	date := p.start

	if len(parts) > 1 {
		remaining := strings.Join(parts[1:], " ")
		if strings.HasPrefix(remaining, "on ") {
			onDate, err := p.parseOn(remaining[3:])
			if err != nil {
				return SimpleDate{}, err
			}
			date = onDate
		}
	}

	return p.parseAtTime(timeStr, date)
}

func (p *HumanDateParser) parseAtTime(timeStr string, base SimpleDate) (SimpleDate, error) {
	hour, minute, err := parseTimeHHMM(timeStr)
	if err != nil {
		return SimpleDate{}, err
	}
	return base.SetHMS(hour, minute, 0), nil
}

// "5th of may", "the 5th of may", "the 5th of may at 13:45"
func (p *HumanDateParser) parseOn(rest string) (SimpleDate, error) {
	s := rest
	s = strings.TrimPrefix(s, "the ")

	parts := strings.Fields(s)
	if len(parts) < 3 {
		return SimpleDate{}, fmt.Errorf("invalid 'on' sentence: %q", rest)
	}

	dayStr := parts[0]
	day, err := parseOrdinalDay(dayStr)
	if err != nil {
		return SimpleDate{}, err
	}

	if parts[1] != "of" {
		return SimpleDate{}, fmt.Errorf("expected 'of' in 'on' sentence, got %q", parts[1])
	}

	monthIdx := -1
	for i, m := range months {
		if parts[2] == m {
			monthIdx = i + 1
			break
		}
	}
	if monthIdx == -1 {
		return SimpleDate{}, fmt.Errorf("unknown month: %q", parts[2])
	}

	date := p.start
	year := date.Year()

	// Roll to next year if the specified date is in the past
	startMonth := int(date.Month())
	startDay := date.Day()
	if monthIdx < startMonth || (monthIdx == startMonth && day < startDay) {
		year++
	}

	date = date.SetYMD(year, monthIdx, day)

	if len(parts) >= 5 && parts[3] == "at" {
		atDate, err := p.parseAtTime(parts[4], date)
		if err != nil {
			return SimpleDate{}, err
		}
		return atDate, nil
	}

	return date, nil
}

// "monday", "week", "month", "monday at 13:00"
func (p *HumanDateParser) parseNext(rest string) (SimpleDate, error) {
	parts := strings.Fields(rest)
	if len(parts) < 1 {
		return SimpleDate{}, fmt.Errorf("invalid 'next' sentence")
	}

	option := parts[0]
	date := p.start

	switch {
	case isWeekday(option):
		targetWd := parseWeekday(option)
		currentWd := date.Weekday()
		daysUntil := (int(targetWd) - int(currentWd) + 7) % 7
		if daysUntil == 0 {
			daysUntil = 7
		}
		date = date.AddDuration(Day(uint32(daysUntil)))
	case option == "week":
		date = date.AddDuration(Week(1))
	case option == "month":
		date = date.AddDuration(Month(1))
	default:
		return SimpleDate{}, fmt.Errorf("unknown next option: %q", option)
	}

	if len(parts) >= 3 && parts[1] == "at" {
		atDate, err := p.parseAtTime(parts[2], date)
		if err != nil {
			return SimpleDate{}, err
		}
		return atDate, nil
	}

	return date, nil
}

// "tomorrow", "tomorrow at 15:35"
func (p *HumanDateParser) parseTomorrow(rest string) (SimpleDate, error) {
	date := p.start.AddDuration(Day(1))
	rest = strings.TrimSpace(rest)

	if rest == "" {
		return date, nil
	}

	if strings.HasPrefix(rest, "at ") {
		timeStr := strings.TrimPrefix(rest, "at ")
		timeStr = strings.TrimSpace(timeStr)
		return p.parseAtTime(timeStr, date)
	}

	return SimpleDate{}, fmt.Errorf("unexpected after 'tomorrow': %q", rest)
}

func parseTimeHHMM(s string) (int, int, error) {
	parts := strings.SplitN(s, ":", 2)
	if len(parts) != 2 {
		return 0, 0, fmt.Errorf("invalid time format: %q", s)
	}
	hour, err := strconv.Atoi(parts[0])
	if err != nil {
		return 0, 0, fmt.Errorf("invalid hour: %q", parts[0])
	}
	minute, err := strconv.Atoi(parts[1])
	if err != nil {
		return 0, 0, fmt.Errorf("invalid minute: %q", parts[1])
	}
	if hour < 0 || hour > 23 || minute < 0 || minute > 59 {
		return 0, 0, fmt.Errorf("time out of range: %02d:%02d", hour, minute)
	}
	return hour, minute, nil
}

func parseOrdinalDay(s string) (int, error) {
	s = strings.TrimRight(s, "stndrdth")
	day, err := strconv.Atoi(s)
	if err != nil {
		return 0, fmt.Errorf("invalid day ordinal: %q", s)
	}
	if day < 1 || day > 31 {
		return 0, fmt.Errorf("day out of range: %d", day)
	}
	return day, nil
}

func isWeekday(s string) bool {
	for _, wd := range daysOfWeek {
		if s == wd {
			return true
		}
	}
	return false
}

func parseWeekday(s string) time.Weekday {
	switch s {
	case "monday":
		return time.Monday
	case "tuesday":
		return time.Tuesday
	case "wednesday":
		return time.Wednesday
	case "thursday":
		return time.Thursday
	case "friday":
		return time.Friday
	case "saturday":
		return time.Saturday
	case "sunday":
		return time.Sunday
	}
	return time.Sunday
}
