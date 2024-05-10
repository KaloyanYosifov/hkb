use chrono::{Datelike, NaiveDateTime, TimeDelta};
use std::time::Duration as STDDuration;
use thiserror::Error as ThisError;

static SECONDS_PER_MINUTE: u64 = 60;
static SECONDS_PER_HOUR: u64 = SECONDS_PER_MINUTE * SECONDS_PER_MINUTE;
static SECONDS_PER_DAY: u64 = SECONDS_PER_HOUR * 24;

const MONTHS_IN_A_YEAR: u32 = 12;

type DateUnit = u32;

#[derive(ThisError, Debug)]
pub enum DurationError {
    #[error("Invalid duration specified: {0}")]
    InvalidDurationSpecified(String),
}

// TODO: do we need negative values here?
// For me a duration is forwards in time
pub enum Duration {
    Minute(DateUnit),
    Hour(DateUnit),
    Day(DateUnit),
    Week(DateUnit),
    Month(DateUnit),
    Year(DateUnit),
}

impl Duration {
    pub fn from_string(duration: &str, value: DateUnit) -> Result<Self, DurationError> {
        let duration = match duration {
            "minute" => Self::Minute(value),
            "hour" => Self::Hour(value),
            "day" => Self::Day(value),
            "week" => Self::Week(value),
            "month" => Self::Month(value),
            "year" => Self::Year(value),
            _ => return Err(DurationError::InvalidDurationSpecified(duration.into())),
        };

        Ok(duration)
    }

    pub fn get_value(&self) -> DateUnit {
        match self {
            Duration::Minute(v) => v,
            Duration::Hour(v) => v,
            Duration::Day(v) => v,
            Duration::Week(v) => v,
            Duration::Month(v) => v,
            Duration::Year(v) => v,
        }
        .clone()
    }
}

impl std::ops::Add<Duration> for NaiveDateTime {
    type Output = NaiveDateTime;

    fn add(self, rhs: Duration) -> Self::Output {
        match rhs {
            Duration::Minute(v) => self + (TimeDelta::minutes(v as i64)),
            Duration::Hour(v) => self + (TimeDelta::hours(v as i64)),
            Duration::Day(v) => self + (TimeDelta::days(v as i64)),
            Duration::Week(v) => self + (TimeDelta::weeks(v as i64)),
            Duration::Month(v) => {
                let mut new_month = self.month() + v;

                if new_month <= MONTHS_IN_A_YEAR {
                    return self.with_month(new_month).unwrap();
                }

                let years_to_add = (new_month / MONTHS_IN_A_YEAR) as i32;
                let new_date = self
                    .with_year(self.year() + years_to_add)
                    .unwrap_or_else(|| self.with_year(0).unwrap());

                new_month = new_month % MONTHS_IN_A_YEAR;

                if new_month == 0 {
                    new_month = MONTHS_IN_A_YEAR;
                }

                new_date
                    .with_month(new_month)
                    .unwrap_or_else(|| new_date.with_month(1).unwrap())
            }
            Duration::Year(v) => {
                let mut year = (self.year() as u32) + v;

                if year > (i32::MAX as u32) {
                    year = 0;
                }

                self.with_year(year as i32)
                    .unwrap_or_else(|| self.with_year(0).unwrap())
            }
        }
    }
}

impl std::ops::Sub<Duration> for NaiveDateTime {
    type Output = NaiveDateTime;

    fn sub(self, rhs: Duration) -> Self::Output {
        match rhs {
            Duration::Minute(v) => self - (TimeDelta::minutes(v as i64)),
            Duration::Hour(v) => self - (TimeDelta::hours(v as i64)),
            Duration::Day(v) => self - (TimeDelta::days(v as i64)),
            Duration::Week(v) => self - (TimeDelta::weeks(v as i64)),
            Duration::Month(v) => {
                if self.month() > v {
                    let mut month = self.month() - v;

                    if month == 0 {
                        month = 1;
                    }

                    return self.with_month(month).unwrap();
                }

                let mut new_month = MONTHS_IN_A_YEAR - (v - self.month());
                let mut years_to_subtract = 1;

                while new_month > MONTHS_IN_A_YEAR {
                    new_month = new_month.checked_sub(MONTHS_IN_A_YEAR).unwrap_or(0);
                    years_to_subtract += 1;
                }

                let new_date = {
                    let new_year = self.year().checked_sub(years_to_subtract).unwrap_or(0);

                    self.with_year(new_year)
                        .unwrap_or_else(|| self.with_year(0).unwrap())
                };

                new_date
                    .with_month(new_month)
                    .unwrap_or_else(|| new_date.with_month(1).unwrap())
            }
            Duration::Year(v) => {
                let year = (self.year() as u32).checked_sub(v).unwrap_or(0);

                self.with_year(year as i32)
                    .unwrap_or_else(|| self.with_year(0).unwrap())
            }
        }
    }
}

pub trait HumanizedDuration {
    fn to_human_string(&self) -> String;
}

impl HumanizedDuration for STDDuration {
    fn to_human_string(&self) -> String {
        let mut seconds = self.as_secs();
        let unit_suffix = vec!["days", "hours", "minutes"];

        vec![SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE]
            .into_iter()
            .enumerate()
            // TODO: if performance degrades due to format!'s new string allocation
            // we could go for an iterative approach with preallocated string
            .map(|(i, x)| {
                let dur = seconds / x;

                if dur <= 0 {
                    return "".to_string();
                }

                seconds %= x;

                let unit_suffix = unit_suffix[i];
                let suffix = {
                    if dur >= 2 {
                        &unit_suffix
                    } else {
                        &unit_suffix[0..unit_suffix.len() - 1]
                    }
                };

                format!("{} {}", dur, suffix)
            })
            .filter(|x| x != "")
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_duration_in_human_time() {
        assert_eq!("1 day", STDDuration::new(86400, 0).to_human_string());
        assert_eq!(
            "1 day 1 minute",
            STDDuration::new(86400 + 60, 0).to_human_string()
        );
        assert_eq!("2 days", STDDuration::new(86400 * 2, 0).to_human_string());
        assert_eq!(
            "2 days 2 hours 2 minutes",
            STDDuration::new((86400 * 2) + (3600 * 2) + 125, 0).to_human_string()
        );

        assert_eq!("1 hour", STDDuration::new(3600, 0).to_human_string());
        assert_eq!(
            "1 hour 1 minute",
            STDDuration::new(3600 + 60, 0).to_human_string()
        );
        assert_eq!("2 hours", STDDuration::new(3600 * 2, 0).to_human_string());
        assert_eq!(
            "2 hours 2 minutes",
            STDDuration::new(3600 * 2 + 60 * 2, 0).to_human_string()
        );

        assert_eq!("1 minute", STDDuration::new(60, 0).to_human_string());
        assert_eq!("2 minutes", STDDuration::new(60 * 2, 0).to_human_string());
        assert_eq!("59 minutes", STDDuration::new(60 * 59, 0).to_human_string());
        assert_eq!("", STDDuration::new(59, 0).to_human_string());
    }
}
