use chrono::{
    DateTime, Datelike, Local, NaiveDate, NaiveTime, ParseError, TimeDelta, Timelike, Utc,
};
use thiserror::Error as ThisError;

const MONTHS_IN_A_YEAR: u32 = 12;

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

impl<T: chrono::TimeZone> std::ops::Add<Duration> for DateTime<T> {
    type Output = DateTime<T>;

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
                let new_date = {
                    if let Some(date) = self.with_year(self.year() + years_to_add) {
                        date
                    } else {
                        self.with_year(0).unwrap()
                    }
                };

                new_month = new_month % MONTHS_IN_A_YEAR;

                if new_month == 0 {
                    new_month = MONTHS_IN_A_YEAR;
                }

                if let Some(date) = new_date.with_month(new_month) {
                    date
                } else {
                    new_date.with_month(0).unwrap()
                }
            }
            Duration::Year(v) => {
                let mut year = (self.year() as u32) + v;

                if year > (i32::MAX as u32) {
                    year = 0;
                }

                if let Some(date) = self.with_year(year as i32) {
                    date
                } else {
                    self.with_year(0).unwrap()
                }
            }
        }
    }
}

pub type DateResult = Result<(), DateError>;

type DateUnit = u32;

pub trait Date: ToString {
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

    #[cfg(feature = "chrono")]
    fn to_chrono_date(&self) -> Self::DateTime;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SimpleUtcDate {
    date: DateTime<Utc>,
}

impl SimpleUtcDate {
    pub fn now() -> Self {
        Self { date: Utc::now() }
    }

    pub fn parse_from_rfc3339(date: impl AsRef<str>) -> Result<Self, DateError> {
        let date = chrono::DateTime::parse_from_rfc3339(date.as_ref())?.to_utc();

        Ok(Self { date })
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

    #[cfg(feature = "chrono")]
    fn to_chrono_date(&self) -> Self::DateTime {
        self.date.clone()
    }
}

impl ToString for SimpleUtcDate {
    fn to_string(&self) -> String {
        self.date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SimpleLocalDate {
    date: DateTime<Local>,
}

impl SimpleLocalDate {
    pub fn now() -> Self {
        Self { date: Local::now() }
    }

    pub fn parse_from_rfc3339(date: impl AsRef<str>) -> Result<Self, DateError> {
        let date = chrono::DateTime::parse_from_rfc3339(date.as_ref())?
            .naive_local()
            .and_local_timezone(Local)
            .unwrap();

        Ok(Self { date })
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

    #[cfg(feature = "chrono")]
    fn to_chrono_date(&self) -> Self::DateTime {
        self.date.clone()
    }
}

impl ToString for SimpleLocalDate {
    fn to_string(&self) -> String {
        self.date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    macro_rules! assert_correct_date_time_from_duration {
        ($expected_date:literal, $duration:expr) => {
            let date = NaiveDateTime::parse_from_str("2024-04-14 08:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap()
                .and_utc();

            assert_eq!($expected_date, (date + $duration).to_string());
        };
    }

    // Sanity checks below (even though chrono has test cases for these below we do a sanity check)
    // to verify we are passing correct duration

    #[test]
    fn minute_duration_can_be_added_to_date_time() {
        assert_correct_date_time_from_duration!("2024-04-14 08:01:00 UTC", Duration::Minute(1));
        assert_correct_date_time_from_duration!("2024-04-14 08:05:00 UTC", Duration::Minute(5));
        assert_correct_date_time_from_duration!("2024-04-14 08:10:00 UTC", Duration::Minute(10));
        assert_correct_date_time_from_duration!("2024-04-14 08:15:00 UTC", Duration::Minute(15));
        assert_correct_date_time_from_duration!("2024-04-14 08:20:00 UTC", Duration::Minute(20));
        assert_correct_date_time_from_duration!("2024-04-14 08:25:00 UTC", Duration::Minute(25));
        assert_correct_date_time_from_duration!("2024-04-14 08:30:00 UTC", Duration::Minute(30));
        assert_correct_date_time_from_duration!("2024-04-14 09:00:00 UTC", Duration::Minute(60));
        assert_correct_date_time_from_duration!("2024-04-14 09:05:00 UTC", Duration::Minute(65));
        assert_correct_date_time_from_duration!("2024-04-14 09:10:00 UTC", Duration::Minute(70));
        assert_correct_date_time_from_duration!("2024-04-14 09:30:00 UTC", Duration::Minute(90));
        assert_correct_date_time_from_duration!("2024-04-15 08:00:00 UTC", Duration::Minute(1440));
    }

    #[test]
    fn hour_duration_can_be_added_to_date_time() {
        assert_correct_date_time_from_duration!("2024-04-14 09:00:00 UTC", Duration::Hour(1));
        assert_correct_date_time_from_duration!("2024-04-14 13:00:00 UTC", Duration::Hour(5));
        assert_correct_date_time_from_duration!("2024-04-14 18:00:00 UTC", Duration::Hour(10));
        assert_correct_date_time_from_duration!("2024-04-14 23:00:00 UTC", Duration::Hour(15));
        assert_correct_date_time_from_duration!("2024-04-15 00:00:00 UTC", Duration::Hour(16));
        assert_correct_date_time_from_duration!("2024-04-15 01:00:00 UTC", Duration::Hour(17));
        assert_correct_date_time_from_duration!("2024-04-15 08:00:00 UTC", Duration::Hour(24));
        assert_correct_date_time_from_duration!("2024-04-16 08:00:00 UTC", Duration::Hour(48));
        assert_correct_date_time_from_duration!("2024-04-21 08:00:00 UTC", Duration::Hour(168));
    }

    #[test]
    fn week_duration_can_be_added_to_date_time() {
        assert_correct_date_time_from_duration!("2024-04-21 08:00:00 UTC", Duration::Week(1));
        assert_correct_date_time_from_duration!("2024-05-19 08:00:00 UTC", Duration::Week(5));
        assert_correct_date_time_from_duration!("2024-06-23 08:00:00 UTC", Duration::Week(10));
        assert_correct_date_time_from_duration!("2024-10-13 08:00:00 UTC", Duration::Week(26));
        assert_correct_date_time_from_duration!("2025-04-13 08:00:00 UTC", Duration::Week(52));
    }

    #[test]
    fn month_duration_can_be_added_to_date_time() {
        assert_correct_date_time_from_duration!("2024-05-14 08:00:00 UTC", Duration::Month(1));
        assert_correct_date_time_from_duration!("2024-06-14 08:00:00 UTC", Duration::Month(2));
        assert_correct_date_time_from_duration!("2024-07-14 08:00:00 UTC", Duration::Month(3));
        assert_correct_date_time_from_duration!("2024-08-14 08:00:00 UTC", Duration::Month(4));
        assert_correct_date_time_from_duration!("2024-09-14 08:00:00 UTC", Duration::Month(5));
        assert_correct_date_time_from_duration!("2024-10-14 08:00:00 UTC", Duration::Month(6));
        assert_correct_date_time_from_duration!("2024-11-14 08:00:00 UTC", Duration::Month(7));
        assert_correct_date_time_from_duration!("2024-12-14 08:00:00 UTC", Duration::Month(8));
        assert_correct_date_time_from_duration!("2025-01-14 08:00:00 UTC", Duration::Month(9));
        assert_correct_date_time_from_duration!("2025-02-14 08:00:00 UTC", Duration::Month(10));
        assert_correct_date_time_from_duration!("2025-03-14 08:00:00 UTC", Duration::Month(11));
        assert_correct_date_time_from_duration!("2025-04-14 08:00:00 UTC", Duration::Month(12));
        assert_correct_date_time_from_duration!("2025-05-14 08:00:00 UTC", Duration::Month(13));
    }

    #[test]
    fn year_duration_can_be_added_to_date_time() {
        assert_correct_date_time_from_duration!("2025-04-14 08:00:00 UTC", Duration::Year(1));
        assert_correct_date_time_from_duration!("2029-04-14 08:00:00 UTC", Duration::Year(5));
        assert_correct_date_time_from_duration!("2034-04-14 08:00:00 UTC", Duration::Year(10));
        assert_correct_date_time_from_duration!("2039-04-14 08:00:00 UTC", Duration::Year(15));
        assert_correct_date_time_from_duration!("2044-04-14 08:00:00 UTC", Duration::Year(20));
        assert_correct_date_time_from_duration!("2049-04-14 08:00:00 UTC", Duration::Year(25));
        assert_correct_date_time_from_duration!("2054-04-14 08:00:00 UTC", Duration::Year(30));
        assert_correct_date_time_from_duration!(
            "0000-04-14 08:00:00 UTC",
            Duration::Year((i32::MAX - 2025) as u32)
        );
    }
}
