use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error as ThisError;

use crate::date::{Date, DateError, Duration, DurationError};

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

type DateParsingResult<T> = Result<T, DateParsingError>;

fn ctoi(char: char) -> u8 {
    // 48 is the ascii code of 0
    return (char as u8) - 48;
}

#[derive(Parser)]
#[grammar = "../grammar/human_date.pest"]
struct PestHumanDateParser;

pub struct HumanDateParser<T: Date> {
    start_date: T,
}

impl<T: Date + Clone> HumanDateParser<T> {
    pub fn new(start_date: T) -> Self {
        Self { start_date }
    }
}

impl<T: Date + Clone> HumanDateParser<T> {
    fn parse_in_sentence(&self, sentence: Pair<Rule>) -> DateParsingResult<T> {
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
        let mut date = self.start_date.clone();

        date.add_duration(duration)?;

        Ok(date)
    }

    fn parse_on_sentence(&self, sentence: Pair<Rule>) -> DateParsingResult<T> {
        // We are unwrapping because we are sure we have these in the
        // data structure
        let (day, month) = {
            let mut inner = sentence.into_inner();
            let day = inner.next().unwrap().as_str();
            let day = (&day[0..day.len() - 2]).parse::<u32>().unwrap();
            let month = inner.next().unwrap().as_str();
            let month = (MONTHS.iter().position(|&m| m == month).unwrap() + 1) as u32;

            (day, month)
        };
        let mut date = self.start_date.clone();

        date.set_ymd(date.year(), month, day)?;

        Ok(date)
    }

    fn parse_at_sentence(&self, sentence: Pair<Rule>) -> DateParsingResult<T> {
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

    /// Parse a human date string into a date
    ///
    /// Example
    /// ```rust
    /// use hkb_date::{HumanDateParser};
    /// use hkb_date::date::*;
    /// let date_parser = HumanDateParser::new(SimpleUtcDate::now());
    /// let input = "In 5 minutes";
    /// println!("{}", date_parser.parse(input).unwrap().to_string());
    ///
    /// ```
    pub fn parse(&self, input: impl AsRef<str>) -> DateParsingResult<T> {
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
            _ => Err(DateParsingError::UnknownRuleEncountered()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::{Date, SimpleUtcDate};

    macro_rules! assert_date_parsing {
        ($input:literal, $expected: literal) => {
            let date =
                SimpleUtcDate::parse_from_str("2024-04-14 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
            let date_parser = HumanDateParser::new(date);
            let date = date_parser
                .parse($input)
                .expect("We should have been able to parse!");

            assert_eq!($expected, date.to_string());
        };
    }

    #[test]
    fn it_can_parse_in_sentence() {
        assert_date_parsing!("In 10 minutes", "2024-04-14 08:10:00 UTC");
        assert_date_parsing!("In 5 days", "2024-04-19 08:00:00 UTC");
        assert_date_parsing!("In 3 months", "2024-07-14 08:00:00 UTC");
    }

    #[test]
    fn it_can_parse_at_sentence() {
        assert_date_parsing!("At 05:00", "2024-04-14 05:00:00 UTC");
        assert_date_parsing!("At 15:00 on the 31st of January", "2024-01-31 15:00:00 UTC");
        assert_date_parsing!("At 22:30 on the 30th of March", "2024-03-30 22:30:00 UTC");
        assert_date_parsing!(
            "At 13:00 on the 11th of December",
            "2024-12-11 13:00:00 UTC"
        );
    }

    #[test]
    fn it_can_parse_on_sentence() {
        assert_date_parsing!("On 5th of May", "2024-05-05 08:00:00 UTC");
        assert_date_parsing!("On the 5th of May", "2024-05-05 08:00:00 UTC");
        assert_date_parsing!("On the 1st of January", "2024-01-01 08:00:00 UTC");
    }
}
