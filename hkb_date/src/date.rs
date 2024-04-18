use chrono::{DateTime, Datelike, Local, ParseError, Timelike, Utc};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum DateError {
    #[error("Failed to parse date string")]
    FailedToParseDateString(#[from] ParseError),

    #[error("Failed to set {0} with {1}")]
    FailedToSetTimeUnit(String, u32),
}

pub enum Timezone {
    UTC,
    Local,
}

pub enum Duration {
    Minute(u32),
    Hour(u32),
    Day(u32),
    Week(u32),
    Month(u32),
    Year(u32),
}

pub type DateResult = Result<(), DateError>;

pub trait Date {
    type DateTime;

    fn set_year(&mut self, year: u32) -> DateResult;
    fn set_month(&mut self, month: u32) -> DateResult;
    fn set_day(&mut self, day: u32) -> DateResult;
    fn set_hour(&mut self, hour: u32) -> DateResult;
    fn set_minute(&mut self, minute: u32) -> DateResult;
    fn set_second(&mut self, second: u32) -> DateResult;

    fn get_timezone(&self) -> Timezone;

    fn to_string(&self) -> String;

    #[cfg(feature = "chrono")]
    fn to_chrono_date(&self) -> Self::DateTime;
}

fn set_year<Tz: chrono::TimeZone>(
    date: DateTime<Tz>,
    year: u32,
) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_year(year as i32) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("year".to_string(), year))
    }
}

fn set_month<Tz: chrono::TimeZone>(
    date: DateTime<Tz>,
    month: u32,
) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_month(month) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("month".to_string(), month))
    }
}

fn set_day<Tz: chrono::TimeZone>(date: DateTime<Tz>, day: u32) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_day(day) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("day".to_string(), day))
    }
}

fn set_hour<Tz: chrono::TimeZone>(
    date: DateTime<Tz>,
    hour: u32,
) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_hour(hour) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("hour".to_string(), hour))
    }
}

fn set_minute<Tz: chrono::TimeZone>(
    date: DateTime<Tz>,
    minute: u32,
) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_minute(minute) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("minute".to_string(), minute))
    }
}

fn set_second<Tz: chrono::TimeZone>(
    date: DateTime<Tz>,
    second: u32,
) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_second(second) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("second".to_string(), second))
    }
}

pub struct SimpleUtcDate {
    date: DateTime<Utc>,
}

impl SimpleUtcDate {
    pub fn now() -> Self {
        Self { date: Utc::now() }
    }

    pub fn parse_from_str(
        date: impl AsRef<str>,
        format: impl AsRef<str>,
    ) -> Result<Self, DateError> {
        let date = chrono::NaiveDateTime::parse_from_str(date.as_ref(), format.as_ref())?.and_utc();

        Ok(Self { date })
    }
}

impl Date for SimpleUtcDate {
    type DateTime = DateTime<Utc>;

    fn set_year(&mut self, year: u32) -> DateResult {
        self.date = set_year(self.date, year)?;

        Ok(())
    }

    fn set_month(&mut self, month: u32) -> DateResult {
        self.date = set_month(self.date, month)?;

        Ok(())
    }

    fn set_day(&mut self, day: u32) -> DateResult {
        self.date = set_day(self.date, day)?;

        Ok(())
    }

    fn set_hour(&mut self, hour: u32) -> DateResult {
        self.date = set_hour(self.date, hour)?;

        Ok(())
    }

    fn set_minute(&mut self, minute: u32) -> DateResult {
        self.date = set_minute(self.date, minute)?;

        Ok(())
    }

    fn set_second(&mut self, second: u32) -> DateResult {
        self.date = set_second(self.date, second)?;

        Ok(())
    }

    fn get_timezone(&self) -> Timezone {
        Timezone::UTC
    }

    fn to_string(&self) -> String {
        self.date.to_string()
    }

    #[cfg(feature = "chrono")]
    fn to_chrono_date(&self) -> Self::DateTime {}
}

pub struct SimpleLocalDate {
    date: DateTime<Local>,
}

impl SimpleLocalDate {
    pub fn now() -> Self {
        Self { date: Local::now() }
    }

    pub fn parse_from_str(
        date: impl AsRef<str>,
        format: impl AsRef<str>,
    ) -> Result<Self, DateError> {
        let date = chrono::NaiveDateTime::parse_from_str(date.as_ref(), format.as_ref())?
            .and_local_timezone(Local)
            .unwrap();

        Ok(Self { date })
    }
}

impl Date for SimpleLocalDate {
    type DateTime = DateTime<Utc>;

    fn set_year(&mut self, year: u32) -> DateResult {
        self.date = set_year(self.date, year)?;

        Ok(())
    }

    fn set_month(&mut self, month: u32) -> DateResult {
        self.date = set_month(self.date, month)?;

        Ok(())
    }

    fn set_day(&mut self, day: u32) -> DateResult {
        self.date = set_day(self.date, day)?;

        Ok(())
    }

    fn set_hour(&mut self, hour: u32) -> DateResult {
        self.date = set_hour(self.date, hour)?;

        Ok(())
    }

    fn set_minute(&mut self, minute: u32) -> DateResult {
        self.date = set_minute(self.date, minute)?;

        Ok(())
    }

    fn set_second(&mut self, second: u32) -> DateResult {
        self.date = set_second(self.date, second)?;

        Ok(())
    }

    fn get_timezone(&self) -> Timezone {
        Timezone::Local
    }

    fn to_string(&self) -> String {
        self.date.to_string()
    }

    #[cfg(feature = "chrono")]
    fn to_chrono_date(&self) -> Self::DateTime {}
}
