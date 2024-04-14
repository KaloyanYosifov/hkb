use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../grammar/human_date.pest"]
struct HumanDateParser;

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

    println!("{:?}", sentence);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_parse_in_sentence() {
        parse("In 10 minutes");

        assert!(false)
    }
}
