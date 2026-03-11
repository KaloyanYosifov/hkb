package dateutil

import (
	"testing"
)

func assertDateParsing(t *testing.T, input, expected string, startDate ...string) {
	t.Helper()
	start := "2024-04-14 08:00:00"
	if len(startDate) > 0 {
		start = startDate[0]
	}
	sd := mustParse(start)
	parser := NewHumanDateParser(sd)
	result, err := parser.Parse(input)
	if err != nil {
		t.Fatalf("Parse(%q) error: %v", input, err)
	}
	got := result.String()
	if got != expected {
		t.Errorf("Parse(%q) = %q, want %q", input, got, expected)
	}
}

func TestParseInSentence(t *testing.T) {
	assertDateParsing(t, "In 10 minutes", "2024-04-14T08:10:00Z")
	assertDateParsing(t, "In 5 days", "2024-04-19T08:00:00Z")
	assertDateParsing(t, "In 3 months", "2024-07-14T08:00:00Z")
}

func TestParseInSentenceEdgeCases(t *testing.T) {
	assertDateParsing(t, "In 1 minute", "2025-01-01T00:00:00Z", "2024-12-31 23:59:00")
	assertDateParsing(t, "in 3 hours", "2025-01-01T01:00:00Z", "2024-12-31 22:00:00")
	assertDateParsing(t, "In 4 months", "2025-01-29T22:00:00Z", "2024-09-29 22:00:00")
}

func TestParseInAltSentence(t *testing.T) {
	assertDateParsing(t, "In two days", "2024-04-16T08:00:00Z")
	assertDateParsing(t, "In three days", "2024-04-17T08:00:00Z")
	assertDateParsing(t, "In four days", "2024-04-18T08:00:00Z")
	assertDateParsing(t, "In five days", "2024-04-19T08:00:00Z")
	assertDateParsing(t, "In six days", "2024-04-20T08:00:00Z")
	assertDateParsing(t, "In seven days", "2024-04-21T08:00:00Z")
	assertDateParsing(t, "In eight days", "2024-04-22T08:00:00Z")
	assertDateParsing(t, "In nine days", "2024-04-23T08:00:00Z")
}

func TestParseInAltWithTime(t *testing.T) {
	assertDateParsing(t, "In two days at 5:00", "2024-04-16T05:00:00Z")
	assertDateParsing(t, "In three days at 18:30", "2024-04-17T18:30:00Z")
	assertDateParsing(t, "In four days at 23:59", "2024-04-18T23:59:00Z")
	assertDateParsing(t, "In five days at 21:00", "2024-04-19T21:00:00Z")
}

func TestParseInAltEdgeCases(t *testing.T) {
	start := "2024-12-31 22:00:00"
	assertDateParsing(t, "In two days", "2025-01-02T22:00:00Z", start)
	assertDateParsing(t, "In three days", "2025-01-03T22:00:00Z", start)
	assertDateParsing(t, "In seven days", "2025-01-07T22:00:00Z", start)
}

func TestParseAtSentence(t *testing.T) {
	assertDateParsing(t, "At 05:00", "2024-04-14T05:00:00Z")
	assertDateParsing(t, "At 13:00 on the 11th of December", "2024-12-11T13:00:00Z")
}

func TestParseAtSentenceEdgeCases(t *testing.T) {
	assertDateParsing(t, "At 15:00 on the 31st of January", "2025-01-31T15:00:00Z")
	assertDateParsing(t, "At 22:30 on the 30th of March", "2025-03-30T22:30:00Z")
}

func TestParseOnSentence(t *testing.T) {
	assertDateParsing(t, "On 5th of May", "2024-05-05T08:00:00Z")
	assertDateParsing(t, "On the 5th of May", "2024-05-05T08:00:00Z")
	assertDateParsing(t, "On the 1st of August", "2024-08-01T08:00:00Z")
}

func TestParseOnSentenceEdgeCases(t *testing.T) {
	assertDateParsing(t, "On 3rd of February", "2025-02-03T08:00:00Z")
	assertDateParsing(t, "On the 5th of March", "2025-03-05T08:00:00Z")
	assertDateParsing(t, "On the 1st of January", "2025-01-01T08:00:00Z")
}

func TestParseOnWithTime(t *testing.T) {
	assertDateParsing(t, "On 5th of May at 5:00", "2024-05-05T05:00:00Z")
	assertDateParsing(t, "On the 5th of May at 23:59", "2024-05-05T23:59:00Z")
	assertDateParsing(t, "On the 1st of August at 13:45", "2024-08-01T13:45:00Z")
}

func TestParseOnWithTimeEdgeCases(t *testing.T) {
	assertDateParsing(t, "On 3rd of February at 13:10", "2025-02-03T13:10:00Z")
	assertDateParsing(t, "On the 5th of March at 18:05", "2025-03-05T18:05:00Z")
	assertDateParsing(t, "On the 1st of January at 21:33", "2025-01-01T21:33:00Z")
}

func TestParseNextSentence(t *testing.T) {
	// 2024-04-14 is a Sunday
	assertDateParsing(t, "Next Monday", "2024-04-15T08:00:00Z")
	assertDateParsing(t, "Next Tuesday", "2024-04-16T08:00:00Z")
	assertDateParsing(t, "Next Wednesday", "2024-04-17T08:00:00Z")
	assertDateParsing(t, "Next Thursday", "2024-04-18T08:00:00Z")
	assertDateParsing(t, "Next Friday", "2024-04-19T08:00:00Z")
	assertDateParsing(t, "Next Saturday", "2024-04-20T08:00:00Z")
	assertDateParsing(t, "Next Sunday", "2024-04-21T08:00:00Z")
	assertDateParsing(t, "Next Week", "2024-04-21T08:00:00Z")
	assertDateParsing(t, "Next Month", "2024-05-14T08:00:00Z")
}

func TestParseNextSentenceEdgeCases(t *testing.T) {
	// 2024-12-31 is a Tuesday
	start := "2024-12-31 22:00:00"
	assertDateParsing(t, "Next Monday", "2025-01-06T22:00:00Z", start)
	assertDateParsing(t, "Next Tuesday", "2025-01-07T22:00:00Z", start)
	assertDateParsing(t, "Next Wednesday", "2025-01-01T22:00:00Z", start)
	assertDateParsing(t, "Next Thursday", "2025-01-02T22:00:00Z", start)
	assertDateParsing(t, "Next Friday", "2025-01-03T22:00:00Z", start)
	assertDateParsing(t, "Next Saturday", "2025-01-04T22:00:00Z", start)
	assertDateParsing(t, "Next Sunday", "2025-01-05T22:00:00Z", start)
	assertDateParsing(t, "Next Week", "2025-01-07T22:00:00Z", start)
	assertDateParsing(t, "Next Month", "2025-01-31T22:00:00Z", start)
}

func TestParseNextWithTime(t *testing.T) {
	assertDateParsing(t, "Next Monday at 13:00", "2024-04-15T13:00:00Z")
	assertDateParsing(t, "Next Friday at 5:00", "2024-04-19T05:00:00Z")
	assertDateParsing(t, "Next Week at 16:00", "2024-04-21T16:00:00Z")
	assertDateParsing(t, "Next Month at 17:54", "2024-05-14T17:54:00Z")
}

func TestParseNextWithTimeEdgeCases(t *testing.T) {
	start := "2024-12-31 22:00:00"
	assertDateParsing(t, "Next Monday at 13:00", "2025-01-06T13:00:00Z", start)
	assertDateParsing(t, "Next Week at 16:00", "2025-01-07T16:00:00Z", start)
	assertDateParsing(t, "Next Month at 17:54", "2025-01-31T17:54:00Z", start)
}

func TestParseTomorrowSentence(t *testing.T) {
	assertDateParsing(t, "Tomorrow", "2024-04-15T08:00:00Z")
	assertDateParsing(t, "Tomorrow at 03:00", "2024-04-15T03:00:00Z")
	assertDateParsing(t, "Tomorrow at 15:35", "2024-04-15T15:35:00Z")
	assertDateParsing(t, "Tomorrow at 23:59", "2024-04-15T23:59:00Z")
}

func TestParseErrors(t *testing.T) {
	parser := NewHumanDateParser(mustParse("2024-04-14 08:00:00"))

	errorCases := []string{
		"",
		"hello",
		"In",
		"In abc minutes",
		"On foo of bar",
		"Next blah",
	}

	for _, input := range errorCases {
		_, err := parser.Parse(input)
		if err == nil {
			t.Errorf("Parse(%q) expected error, got nil", input)
		}
	}
}
