use chrono::Datelike;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error as ThisError;

use crate::date::{DateError, SimpleDate};
use crate::duration::{Duration, DurationError};

#[derive(ThisError, Debug)]
pub enum DateParsingError {
    #[error("Failed to parse input")]
    FailedToParseInput(String),
    #[error("Unknown rule encountered!")]
    UnknownRuleEncountered(),
    #[error(transparent)]
    InvalidDateError(#[from] DateError),
    #[error(transparent)]
    InvalidDurationSpecified(#[from] DurationError),
}

const DAYS_OF_WEEK: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

type DateParsingResult = Result<SimpleDate, DateParsingError>;

fn ctoi(char: char) -> u8 {
    // 48 is the ascii code of 0
    return (char as u8) - 48;
}

#[derive(Parser)]
#[grammar = "../grammar/human_date.pest"]
struct PestHumanDateParser;

pub struct HumanDateParser {
    start_date: SimpleDate,
}

impl HumanDateParser {
    pub fn new(start_date: SimpleDate) -> Self {
        Self { start_date }
    }
}

impl HumanDateParser {
    fn parse_in_sentence(&self, sentence: Pair<Rule>) -> DateParsingResult {
        let mut inner = sentence.into_inner();
        let mut pair = inner.next().unwrap();
        let mut duration_value: i64 = 0;

        while !matches!(pair.as_rule(), Rule::duration) {
            let number = ctoi(pair.as_span().as_str().chars().next().unwrap());

            duration_value *= 10;
            duration_value += number as i64;

            pair = inner.next().unwrap();
        }

        let duration = pair.as_span().as_str();
        let duration = Duration::from_string(duration, duration_value as u32)?;
        let date = self.start_date.clone().add_duration(duration)?;

        Ok(date)
    }

    fn parse_on_sentence(&self, sentence: Pair<Rule>) -> DateParsingResult {
        let mut inner = sentence.into_inner();
        // We are unwrapping because we are sure we have these in the
        // data structure
        let (day, month) = {
            let day = inner.next().unwrap().as_str();
            let day = (&day[0..day.len() - 2]).parse::<u32>().unwrap();
            let month = inner.next().unwrap().as_str();
            let month = (MONTHS.iter().position(|&m| m == month).unwrap() + 1) as u32;

            (day, month)
        };
        let mut date = {
            if let Some(at_sentence) = inner.next() {
                self.parse_at_sentence(at_sentence)?
            } else {
                self.start_date.clone()
            }
        };
        let mut year = date.year();

        match (date.month(), date.day()) {
            (m, _) if m > month => year += 1,
            (m, d) if m == month && d > day => year += 1,
            _ => {}
        }

        date.set_ymd(year, month, day)?;

        Ok(date)
    }

    fn parse_at_sentence(&self, sentence: Pair<Rule>) -> DateParsingResult {
        let mut inner = sentence.into_inner();
        let hour = inner.next().unwrap().as_str().parse::<u32>().unwrap();
        let minute = inner.next().unwrap().as_str().parse::<u32>().unwrap();

        let date = {
            let mut on_date = {
                if let Some(pair) = inner.next() {
                    self.parse_on_sentence(pair)?
                } else {
                    self.start_date.clone()
                }
            };

            on_date.set_hms(hour, minute, 0)?;

            on_date
        };

        Ok(date)
    }

    fn parse_next_sentence(&self, sentence: Pair<Rule>) -> DateParsingResult {
        let mut inner = sentence.into_inner();
        let pair = inner.next().unwrap();
        let option = pair.as_str();
        let start_date = {
            if let Some(at_sentence) = inner.next() {
                self.parse_at_sentence(at_sentence)?
            } else {
                self.start_date.clone()
            }
        };

        match option {
            day if DAYS_OF_WEEK.contains(&day) => {
                let weekday = day.parse::<chrono::Weekday>().unwrap();
                let current_weekday = start_date.to_chrono_date().weekday();
                let mut days_since_weekday = weekday.days_since(current_weekday);

                // we set it for the next same weekday
                // if the weekday we have specified is the same one as today
                if days_since_weekday == 0 {
                    days_since_weekday = 7;
                }

                Ok(start_date
                    .add_duration(Duration::Day(days_since_weekday))
                    .unwrap())
            }
            "week" => Ok(start_date.clone().add_duration(Duration::Week(1)).unwrap()),
            "month" => Ok(start_date.clone().add_duration(Duration::Month(1)).unwrap()),
            _ => Err(DateParsingError::FailedToParseInput(
                "Invalid day option!".to_string(),
            )),
        }
    }

    fn parse_tomorrow_sentence(&self, sentence: Pair<Rule>) -> DateParsingResult {
        let mut inner = sentence.into_inner();
        let start_date = {
            if let Some(at_sentence) = inner.next() {
                self.parse_at_sentence(at_sentence)?
            } else {
                self.start_date.clone()
            }
        };

        Ok(start_date.add_duration(Duration::Day(1)).unwrap())
    }

    /// Parse a human date string into a date
    ///
    /// Example
    /// ```rust
    /// use hkb_date::{HumanDateParser};
    /// use hkb_date::date::*;
    /// let date_parser = HumanDateParser::new(SimpleDate::local());
    /// let input = "In 5 minutes";
    /// println!("{}", date_parser.parse(input).unwrap().to_string());
    ///
    /// ```
    pub fn parse(&self, input: impl AsRef<str>) -> DateParsingResult {
        let lowercased = input.as_ref().to_lowercase();
        let mut result = match PestHumanDateParser::parse(Rule::SENTENCE, &lowercased) {
            Ok(result) => result,
            Err(_) => return Err(DateParsingError::FailedToParseInput(input.as_ref().into())),
        };
        let sentence = result.next().unwrap();

        match sentence.as_rule() {
            Rule::IN => self.parse_in_sentence(sentence),
            Rule::AT => self.parse_at_sentence(sentence),
            Rule::ON => self.parse_on_sentence(sentence),
            Rule::NEXT => self.parse_next_sentence(sentence),
            Rule::TOMORROW => self.parse_tomorrow_sentence(sentence),
            _ => Err(DateParsingError::UnknownRuleEncountered()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::SimpleDate;

    macro_rules! assert_date_parsing {
        ($input:literal, $expected:literal) => {
            assert_date_parsing!($input, $expected, "2024-04-14 08:00:00");
        };
        ($input:literal, $expected:literal, $start_date:expr) => {
            let date = SimpleDate::parse_from_str($start_date, "%Y-%m-%d %H:%M:%S").unwrap();
            let date_parser = HumanDateParser::new(date);
            let date = date_parser
                .parse($input)
                .expect("We should have been able to parse!");

            assert_eq!($expected, date.to_string());
        };
    }

    #[test]
    fn it_can_parse_in_sentence() {
        assert_date_parsing!("In 10 minutes", "2024-04-14T08:10:00Z");
        assert_date_parsing!("In 5 days", "2024-04-19T08:00:00Z");
        assert_date_parsing!("In 3 months", "2024-07-14T08:00:00Z");

        // edge cases and new years
        assert_date_parsing!("In 1 minute", "2025-01-01T00:00:00Z", "2024-12-31 23:59:00");
        assert_date_parsing!("in 3 hours", "2025-01-01T01:00:00Z", "2024-12-31 22:00:00");
        assert_date_parsing!("In 4 months", "2025-01-29T22:00:00Z", "2024-09-29 22:00:00");
    }

    #[test]
    fn it_can_parse_at_sentence() {
        assert_date_parsing!("At 05:00", "2024-04-14T05:00:00Z");
        assert_date_parsing!("At 13:00 on the 11th of December", "2024-12-11T13:00:00Z");

        // edge cases and new years
        assert_date_parsing!("At 15:00 on the 31st of January", "2025-01-31T15:00:00Z");
        assert_date_parsing!("At 22:30 on the 30th of March", "2025-03-30T22:30:00Z");
    }

    #[test]
    fn it_can_parse_on_sentence() {
        assert_date_parsing!("On 5th of May", "2024-05-05T08:00:00Z");
        assert_date_parsing!("On the 5th of May", "2024-05-05T08:00:00Z");
        assert_date_parsing!("On the 1st of August", "2024-08-01T08:00:00Z");

        // edge cases and new years
        assert_date_parsing!("On 3rd of February", "2025-02-03T08:00:00Z");
        assert_date_parsing!("On the 5th of March", "2025-03-05T08:00:00Z");
        assert_date_parsing!("On the 1st of January", "2025-01-01T08:00:00Z");
    }

    #[test]
    fn it_can_parse_on_sentence_with_specific_time() {
        assert_date_parsing!("On 5th of May at 5:00", "2024-05-05T05:00:00Z");
        assert_date_parsing!("On the 5th of May at 23:59", "2024-05-05T23:59:00Z");
        assert_date_parsing!("On the 1st of August at 13:45", "2024-08-01T13:45:00Z");

        // edge cases and new years
        assert_date_parsing!("On 3rd of February at 13:10", "2025-02-03T13:10:00Z");
        assert_date_parsing!("On the 5th of March at 18:05", "2025-03-05T18:05:00Z");
        assert_date_parsing!("On the 1st of January at 21:33", "2025-01-01T21:33:00Z");
    }

    #[test]
    fn it_can_parse_next_sentence() {
        assert_date_parsing!("Next Monday", "2024-04-15T08:00:00Z");
        assert_date_parsing!("Next Tuesday", "2024-04-16T08:00:00Z");
        assert_date_parsing!("Next Wednesday", "2024-04-17T08:00:00Z");
        assert_date_parsing!("Next Thursday", "2024-04-18T08:00:00Z");
        assert_date_parsing!("Next Friday", "2024-04-19T08:00:00Z");
        assert_date_parsing!("Next Saturday", "2024-04-20T08:00:00Z");
        assert_date_parsing!("Next Sunday", "2024-04-21T08:00:00Z");
        assert_date_parsing!("Next Week", "2024-04-21T08:00:00Z");
        assert_date_parsing!("Next Month", "2024-05-14T08:00:00Z");

        // edge cases and new years
        let start_date = "2024-12-31 22:00:00";
        assert_date_parsing!("Next Monday", "2025-01-06T22:00:00Z", start_date);
        assert_date_parsing!("Next Tuesday", "2025-01-07T22:00:00Z", start_date);
        assert_date_parsing!("Next Wednesday", "2025-01-01T22:00:00Z", start_date);
        assert_date_parsing!("Next Thursday", "2025-01-02T22:00:00Z", start_date);
        assert_date_parsing!("Next Friday", "2025-01-03T22:00:00Z", start_date);
        assert_date_parsing!("Next Saturday", "2025-01-04T22:00:00Z", start_date);
        assert_date_parsing!("Next Sunday", "2025-01-05T22:00:00Z", start_date);
        assert_date_parsing!("Next Week", "2025-01-07T22:00:00Z", start_date);
        assert_date_parsing!("Next Month", "2025-01-31T22:00:00Z", start_date);
    }

    #[test]
    fn it_can_parse_next_sentence_with_specified_time() {
        assert_date_parsing!("Next Monday at 13:00", "2024-04-15T13:00:00Z");
        assert_date_parsing!("Next Tuesday at 21:00", "2024-04-16T21:00:00Z");
        assert_date_parsing!("Next Wednesday at 23:59", "2024-04-17T23:59:00Z");
        assert_date_parsing!("Next Thursday at 03:00", "2024-04-18T03:00:00Z");
        assert_date_parsing!("Next Friday at 5:00", "2024-04-19T05:00:00Z");
        assert_date_parsing!("Next Saturday at 5:45", "2024-04-20T05:45:00Z");
        assert_date_parsing!("Next Sunday at 02:11", "2024-04-21T02:11:00Z");
        assert_date_parsing!("Next Week at 16:00", "2024-04-21T16:00:00Z");
        assert_date_parsing!("Next Month at 17:54", "2024-05-14T17:54:00Z");

        // edge cases and new years
        let start_date = "2024-12-31 22:00:00";
        assert_date_parsing!("Next Monday at 13:00", "2025-01-06T13:00:00Z", start_date);
        assert_date_parsing!("Next Tuesday at 21:00", "2025-01-07T21:00:00Z", start_date);
        assert_date_parsing!(
            "Next Wednesday at 23:59",
            "2025-01-01T23:59:00Z",
            start_date
        );
        assert_date_parsing!("Next Thursday at 03:00", "2025-01-02T03:00:00Z", start_date);
        assert_date_parsing!("Next Friday at 5:00", "2025-01-03T05:00:00Z", start_date);
        assert_date_parsing!("Next Saturday at 5:45", "2025-01-04T05:45:00Z", start_date);
        assert_date_parsing!("Next Sunday at 02:11", "2025-01-05T02:11:00Z", start_date);
        assert_date_parsing!("Next Week at 16:00", "2025-01-07T16:00:00Z", start_date);
        assert_date_parsing!("Next Month at 17:54", "2025-01-31T17:54:00Z", start_date);
    }

    #[test]
    fn it_can_parse_tomorrow_sentence() {
        assert_date_parsing!("Tomorrow", "2024-04-15T08:00:00Z");
        assert_date_parsing!("Tomorrow at 03:00", "2024-04-15T03:00:00Z");
        assert_date_parsing!("Tomorrow at 15:35", "2024-04-15T15:35:00Z");
        assert_date_parsing!("Tomorrow at 23:59", "2024-04-15T23:59:00Z");
    }
}
