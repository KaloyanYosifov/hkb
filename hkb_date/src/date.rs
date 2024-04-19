use chrono::{
    DateTime, Datelike, Local, NaiveDate, NaiveTime, ParseError, TimeDelta, Timelike, Utc,
};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum DateError {
    #[error("Failed to parse date string")]
    FailedToParseDateString(#[from] ParseError),

    #[error("Failed to set {0} with {1}")]
    FailedToSetTimeUnit(String, DateUnit),

    #[error("Failed to set year with {0}")]
    FailedToSetYear(i32),

    #[error("Failed to set time")]
    FailedToSetTime,
}

#[derive(ThisError, Debug)]
pub enum DurationError {
    #[error("Invalid duration specified: {0}")]
    InvalidDurationSpecified(String),
}

pub enum Timezone {
    UTC,
    Local,
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
}

impl Into<TimeDelta> for Duration {
    fn into(self) -> TimeDelta {
        match self {
            Self::Minute(v) => TimeDelta::minutes(v as i64),
            Self::Hour(v) => TimeDelta::hours(v as i64),
            Self::Day(v) => TimeDelta::days(v as i64),
            Self::Week(v) => TimeDelta::weeks(v as i64),
            Self::Month(v) => TimeDelta::weeks(4 * (v as i64)),
            Self::Year(v) => TimeDelta::weeks((12 * 4) * (v as i64)),
        }
    }
}

impl<T: chrono::TimeZone> std::ops::Add<Duration> for DateTime<T> {
    type Output = DateTime<T>;

    fn add(self, rhs: Duration) -> Self::Output {
        let delta: TimeDelta = rhs.into();

        self + delta
    }
}

pub type DateResult = Result<(), DateError>;

type DateUnit = u32;

pub trait Date {
    type DateTime;

    fn add_duration(&mut self, duration: Duration) -> DateResult;
    fn set_year(&mut self, year: i32) -> DateResult;
    fn set_month(&mut self, month: DateUnit) -> DateResult;
    fn set_day(&mut self, day: DateUnit) -> DateResult;
    fn set_hour(&mut self, hour: DateUnit) -> DateResult;
    fn set_minute(&mut self, minute: DateUnit) -> DateResult;
    fn set_second(&mut self, second: DateUnit) -> DateResult;

    fn set_ymd(&mut self, year: i32, month: DateUnit, date: DateUnit) -> DateResult;
    fn set_hms(&mut self, hour: DateUnit, minute: DateUnit, second: DateUnit) -> DateResult;
    fn set_ymdhms(
        &mut self,
        year: i32,
        month: DateUnit,
        date: DateUnit,
        hour: DateUnit,
        minute: DateUnit,
        second: DateUnit,
    ) -> DateResult;

    fn year(&self) -> i32;
    fn month(&self) -> DateUnit;
    fn day(&self) -> DateUnit;
    fn hour(&self) -> DateUnit;
    fn minute(&self) -> DateUnit;
    fn second(&self) -> DateUnit;

    fn get_timezone(&self) -> Timezone;

    fn to_string(&self) -> String;

    #[cfg(feature = "chrono")]
    fn to_chrono_date(&self) -> Self::DateTime;
}

#[derive(Clone, Copy)]
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

    fn add_duration(&mut self, duration: Duration) -> DateResult {
        self.date = self.date + duration;

        Ok(())
    }

    fn set_year(&mut self, year: i32) -> DateResult {
        self.date = set_year(self.date, year)?;

        Ok(())
    }

    fn set_month(&mut self, month: DateUnit) -> DateResult {
        self.date = set_month(self.date, month)?;

        Ok(())
    }

    fn set_day(&mut self, day: DateUnit) -> DateResult {
        self.date = set_day(self.date, day)?;

        Ok(())
    }

    fn set_hour(&mut self, hour: DateUnit) -> DateResult {
        self.date = set_hour(self.date, hour)?;

        Ok(())
    }

    fn set_minute(&mut self, minute: DateUnit) -> DateResult {
        self.date = set_minute(self.date, minute)?;

        Ok(())
    }

    fn set_second(&mut self, second: DateUnit) -> DateResult {
        self.date = set_second(self.date, second)?;

        Ok(())
    }

    fn set_ymd(&mut self, year: i32, month: DateUnit, date: DateUnit) -> DateResult {
        let time =
            NaiveDate::from_ymd_opt(year, month, date).ok_or_else(|| DateError::FailedToSetTime)?;

        self.date = NaiveDate::from_ymd_opt(year, month, date)
            .ok_or_else(|| DateError::FailedToSetTime)?
            .and_hms_opt(self.date.hour(), self.date.minute(), self.date.second())
            .ok_or_else(|| DateError::FailedToSetTime)?
            .and_utc();

        Ok(())
    }

    fn set_hms(&mut self, hour: DateUnit, minute: DateUnit, second: DateUnit) -> DateResult {
        let time = NaiveTime::from_hms_opt(hour, minute, second)
            .ok_or_else(|| DateError::FailedToSetTime)?;

        self.date = self.date.with_time(time).unwrap();

        Ok(())
    }

    fn set_ymdhms(
        &mut self,
        year: i32,
        month: DateUnit,
        date: DateUnit,
        hour: DateUnit,
        minute: DateUnit,
        second: DateUnit,
    ) -> DateResult {
        let time =
            NaiveDate::from_ymd_opt(year, month, date).ok_or_else(|| DateError::FailedToSetTime)?;

        self.date = NaiveDate::from_ymd_opt(year, month, date)
            .ok_or_else(|| DateError::FailedToSetTime)?
            .and_hms_opt(hour, minute, second)
            .ok_or_else(|| DateError::FailedToSetTime)?
            .and_utc();

        Ok(())
    }

    fn year(&self) -> i32 {
        self.date.year()
    }

    fn month(&self) -> DateUnit {
        self.date.month()
    }

    fn day(&self) -> DateUnit {
        self.date.day()
    }

    fn hour(&self) -> DateUnit {
        self.date.hour()
    }

    fn minute(&self) -> DateUnit {
        self.date.minute()
    }

    fn second(&self) -> DateUnit {
        self.date.second()
    }

    fn get_timezone(&self) -> Timezone {
        Timezone::UTC
    }

    fn to_string(&self) -> String {
        self.date.to_string()
    }

    #[cfg(feature = "chrono")]
    fn to_chrono_date(&self) -> Self::DateTime {
        self.date.clone()
    }
}

#[derive(Clone, Copy)]
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
    type DateTime = DateTime<Local>;

    fn add_duration(&mut self, duration: Duration) -> DateResult {
        self.date = self.date + duration;

        Ok(())
    }

    fn set_year(&mut self, year: i32) -> DateResult {
        self.date = set_year(self.date, year)?;

        Ok(())
    }

    fn set_month(&mut self, month: DateUnit) -> DateResult {
        self.date = set_month(self.date, month)?;

        Ok(())
    }

    fn set_day(&mut self, day: DateUnit) -> DateResult {
        self.date = set_day(self.date, day)?;

        Ok(())
    }

    fn set_hour(&mut self, hour: DateUnit) -> DateResult {
        self.date = set_hour(self.date, hour)?;

        Ok(())
    }

    fn set_minute(&mut self, minute: DateUnit) -> DateResult {
        self.date = set_minute(self.date, minute)?;

        Ok(())
    }

    fn set_second(&mut self, second: DateUnit) -> DateResult {
        self.date = set_second(self.date, second)?;

        Ok(())
    }

    fn set_ymd(&mut self, year: i32, month: DateUnit, date: DateUnit) -> DateResult {
        let time =
            NaiveDate::from_ymd_opt(year, month, date).ok_or_else(|| DateError::FailedToSetTime)?;

        // TODO: Fix unwrapping here
        self.date = NaiveDate::from_ymd_opt(year, month, date)
            .ok_or_else(|| DateError::FailedToSetTime)?
            .and_hms_opt(self.date.hour(), self.date.minute(), self.date.second())
            .ok_or_else(|| DateError::FailedToSetTime)?
            .and_local_timezone(Local)
            .unwrap();

        Ok(())
    }

    fn set_hms(&mut self, hour: DateUnit, minute: DateUnit, second: DateUnit) -> DateResult {
        let time = NaiveTime::from_hms_opt(hour, minute, second)
            .ok_or_else(|| DateError::FailedToSetTime)?;

        self.date = self.date.with_time(time).unwrap();

        Ok(())
    }

    fn set_ymdhms(
        &mut self,
        year: i32,
        month: DateUnit,
        date: DateUnit,
        hour: DateUnit,
        minute: DateUnit,
        second: DateUnit,
    ) -> DateResult {
        let time =
            NaiveDate::from_ymd_opt(year, month, date).ok_or_else(|| DateError::FailedToSetTime)?;

        // TODO: Fix unwrapping here
        self.date = NaiveDate::from_ymd_opt(year, month, date)
            .ok_or_else(|| DateError::FailedToSetTime)?
            .and_hms_opt(hour, minute, second)
            .ok_or_else(|| DateError::FailedToSetTime)?
            .and_local_timezone(Local)
            .unwrap();

        Ok(())
    }

    fn year(&self) -> i32 {
        self.date.year()
    }

    fn month(&self) -> DateUnit {
        self.date.month()
    }

    fn day(&self) -> DateUnit {
        self.date.day()
    }

    fn hour(&self) -> DateUnit {
        self.date.hour()
    }

    fn minute(&self) -> DateUnit {
        self.date.minute()
    }

    fn second(&self) -> DateUnit {
        self.date.second()
    }

    fn get_timezone(&self) -> Timezone {
        Timezone::Local
    }

    fn to_string(&self) -> String {
        self.date.to_string()
    }

    #[cfg(feature = "chrono")]
    fn to_chrono_date(&self) -> Self::DateTime {
        self.date.clone()
    }
}
fn set_year<Tz: chrono::TimeZone>(
    date: DateTime<Tz>,
    year: i32,
) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_year(year) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetYear(year))
    }
}

fn set_month<Tz: chrono::TimeZone>(
    date: DateTime<Tz>,
    month: DateUnit,
) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_month(month) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("month".to_string(), month))
    }
}

fn set_day<Tz: chrono::TimeZone>(
    date: DateTime<Tz>,
    day: DateUnit,
) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_day(day) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("day".to_string(), day))
    }
}

fn set_hour<Tz: chrono::TimeZone>(
    date: DateTime<Tz>,
    hour: DateUnit,
) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_hour(hour) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("hour".to_string(), hour))
    }
}

fn set_minute<Tz: chrono::TimeZone>(
    date: DateTime<Tz>,
    minute: DateUnit,
) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_minute(minute) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("minute".to_string(), minute))
    }
}

fn set_second<Tz: chrono::TimeZone>(
    date: DateTime<Tz>,
    second: DateUnit,
) -> Result<DateTime<Tz>, DateError> {
    if let Some(date) = date.with_second(second) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("second".to_string(), second))
    }
}
