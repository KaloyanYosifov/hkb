use chrono::{DateTime, TimeDelta, Utc};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[cfg(not(test))]
macro_rules! now {
    () => {
        chrono::prelude::Utc::now()
    };
}

#[cfg(test)]
macro_rules! now {
    () => {{
        let date =
            chrono::NaiveDateTime::parse_from_str("2024-04-14 08:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap();
        chrono::DateTime::from_naive_utc_and_offset(date, chrono::Utc)
            as chrono::DateTime<chrono::Utc>
    }};
}

#[derive(Parser)]
#[grammar = "../grammar/human_date.pest"]
struct HumanDateParser;

fn ctoi(char: char) -> u8 {
    // 48 is the ascii code of 0
    return (char as u8) - 48;
}

fn parse_in_sentence(sentence: Pair<Rule>) -> DateTime<Utc> {
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
    let duration = match duration {
        "minute" => TimeDelta::minutes(duration_value),
        "hour" => TimeDelta::hours(duration_value),
        "day" => TimeDelta::days(duration_value),
        "week" => TimeDelta::weeks(duration_value),
        "month" => TimeDelta::weeks(4 * duration_value),
        "year" => TimeDelta::weeks((12 * 4) * duration_value),
        _ => panic!("NOOOOOOO"),
    };
    let final_date = now!() + duration;

    final_date
}

fn parse_on_sentence(sentence: Pair<Rule>) -> DateTime<Utc> {
    todo!("Implement on sentence!");
}

fn parse_at_sentence(sentence: Pair<Rule>) -> DateTime<Utc> {
    todo!("Implement at sentence!");
}

/// Parse a human date string into a date
///
/// Example
/// ```rust
/// use hkb_date::parse_human_date;
/// let input = "In 5 minutes";
/// println!("{:?}}, parse_human_date(input));
///
/// ```
pub fn parse(input: impl AsRef<str>) {
    let lowercased = input.as_ref().to_lowercase();
    let mut result = match HumanDateParser::parse(Rule::SENTENCE, &lowercased) {
        Ok(result) => result,
        Err(e) => panic!("Could not parse {:?}", e),
    };
    let sentence = result.next().unwrap();

    match sentence.as_rule() {
        Rule::IN => parse_in_sentence(sentence),
        Rule::AT => parse_at_sentence(sentence),
        Rule::ON => parse_on_sentence(sentence),
        _ => panic!("Unknown rule!"),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_parse_in_sentence() {
        parse("In 10 minutes");

        // TODO: fix assert
        assert!(false)
    }

    #[test]
    fn it_can_parse_at_sentence() {
        parse("At 5:00");
        parse("At 5:00 on the 31st of January");
        parse("At 5:00 on the 30th of March");
        parse("At 5:00 on the 11th of aDecembere");

        // TODO: fix assert
        assert!(false)
    }
}
