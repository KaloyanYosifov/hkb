use crate::duration::*;
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, ParseError, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration as STDDuration;
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

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Timezone {
    UTC,
    Local,
}

pub type DateResult<T> = Result<T, DateError>;

type DateUnit = u32;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimpleDate {
    date: NaiveDateTime,
    timezone: Timezone,
}

impl SimpleDate {
    pub fn now() -> Self {
        Self {
            date: Utc::now().naive_utc(),
            timezone: Timezone::UTC,
        }
    }

    pub fn local() -> Self {
        Self {
            date: Local::now().naive_local(),
            timezone: Timezone::Local,
        }
    }

    pub fn parse_from_rfc3339(date: impl AsRef<str>) -> Result<Self, DateError> {
        let date = chrono::DateTime::parse_from_rfc3339(date.as_ref())?.to_utc();

        Ok(Self {
            date: date.naive_utc(),
            timezone: Timezone::UTC,
        })
    }

    pub fn parse_from_str(
        date: impl AsRef<str>,
        format: impl AsRef<str>,
    ) -> Result<Self, DateError> {
        let date = chrono::NaiveDateTime::parse_from_str(date.as_ref(), format.as_ref())?.and_utc();

        Ok(Self {
            date: date.naive_utc(),
            timezone: Timezone::UTC,
        })
    }

    pub fn add_duration(mut self, duration: impl AsRef<Duration>) -> DateResult<Self> {
        self.date = self.date + duration.as_ref();

        Ok(self)
    }

    pub fn sub_duration(mut self, duration: impl AsRef<Duration>) -> DateResult<Self> {
        self.date = self.date - duration.as_ref();

        Ok(self)
    }

    pub fn set_year(&mut self, year: i32) -> DateResult<()> {
        self.date = set_year(self.date, year)?;

        Ok(())
    }

    pub fn set_month(&mut self, month: DateUnit) -> DateResult<()> {
        self.date = set_month(self.date, month)?;

        Ok(())
    }

    pub fn set_day(&mut self, day: DateUnit) -> DateResult<()> {
        self.date = set_day(self.date, day)?;

        Ok(())
    }

    pub fn set_hour(&mut self, hour: DateUnit) -> DateResult<()> {
        self.date = set_hour(self.date, hour)?;

        Ok(())
    }

    pub fn set_minute(&mut self, minute: DateUnit) -> DateResult<()> {
        self.date = set_minute(self.date, minute)?;

        Ok(())
    }

    pub fn set_second(&mut self, second: DateUnit) -> DateResult<()> {
        self.date = set_second(self.date, second)?;

        Ok(())
    }

    pub fn set_ymd(&mut self, year: i32, month: DateUnit, date: DateUnit) -> DateResult<()> {
        self.date = NaiveDate::from_ymd_opt(year, month, date)
            .ok_or_else(|| DateError::FailedToSetTime)?
            .and_hms_opt(self.date.hour(), self.date.minute(), self.date.second())
            .ok_or_else(|| DateError::FailedToSetTime)?;

        Ok(())
    }

    pub fn set_hms(
        &mut self,
        hour: DateUnit,
        minute: DateUnit,
        second: DateUnit,
    ) -> DateResult<()> {
        let time = NaiveTime::from_hms_opt(hour, minute, second)
            .ok_or_else(|| DateError::FailedToSetTime)?;

        self.date = NaiveDateTime::new(self.date.date(), time);

        Ok(())
    }

    pub fn set_ymdhms(
        &mut self,
        year: i32,
        month: DateUnit,
        date: DateUnit,
        hour: DateUnit,
        minute: DateUnit,
        second: DateUnit,
    ) -> DateResult<()> {
        self.date = NaiveDate::from_ymd_opt(year, month, date)
            .ok_or_else(|| DateError::FailedToSetTime)?
            .and_hms_opt(hour, minute, second)
            .ok_or_else(|| DateError::FailedToSetTime)?;

        Ok(())
    }

    pub fn year(&self) -> i32 {
        self.date.year()
    }

    pub fn month(&self) -> DateUnit {
        self.date.month()
    }

    pub fn day(&self) -> DateUnit {
        self.date.day()
    }

    pub fn hour(&self) -> DateUnit {
        self.date.hour()
    }

    pub fn minute(&self) -> DateUnit {
        self.date.minute()
    }

    pub fn second(&self) -> DateUnit {
        self.date.second()
    }

    pub fn get_timezone(&self) -> Timezone {
        self.timezone
    }

    #[cfg(not(feature = "chrono"))]
    pub(crate) fn to_chrono_date(&self) -> chrono::NaiveDateTime {
        self.date.clone()
    }

    #[cfg(feature = "chrono")]
    pub fn to_chrono_date(&self) -> chrono::NaiveDateTime {
        self.date.clone()
    }
}

impl ToString for SimpleDate {
    fn to_string(&self) -> String {
        self.date
            .and_utc()
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }
}

impl std::ops::Sub<SimpleDate> for SimpleDate {
    type Output = STDDuration;

    fn sub(self, rhs: SimpleDate) -> Self::Output {
        (self.date - rhs.date)
            .to_std()
            .unwrap_or(STDDuration::new(0, 0))
    }
}

fn set_year(date: NaiveDateTime, year: i32) -> Result<NaiveDateTime, DateError> {
    if let Some(date) = date.with_year(year) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetYear(year))
    }
}

fn set_month(date: NaiveDateTime, month: DateUnit) -> Result<NaiveDateTime, DateError> {
    if let Some(date) = date.with_month(month) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("month".to_string(), month))
    }
}

fn set_day(date: NaiveDateTime, day: DateUnit) -> Result<NaiveDateTime, DateError> {
    if let Some(date) = date.with_day(day) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("day".to_string(), day))
    }
}

fn set_hour(date: NaiveDateTime, hour: DateUnit) -> Result<NaiveDateTime, DateError> {
    if let Some(date) = date.with_hour(hour) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("hour".to_string(), hour))
    }
}

fn set_minute(date: NaiveDateTime, minute: DateUnit) -> Result<NaiveDateTime, DateError> {
    if let Some(date) = date.with_minute(minute) {
        Ok(date)
    } else {
        Err(DateError::FailedToSetTimeUnit("minute".to_string(), minute))
    }
}

fn set_second(date: NaiveDateTime, second: DateUnit) -> Result<NaiveDateTime, DateError> {
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
            let date =
                NaiveDateTime::parse_from_str("2024-04-14 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

            assert_eq!($expected_date, (date + $duration).to_string());
        };
    }

    macro_rules! assert_correct_date_time_from_sub_duration {
        ($expected_date:literal, $duration:expr) => {
            let date =
                NaiveDateTime::parse_from_str("2024-04-14 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

            assert_eq!($expected_date, (date - $duration).to_string());
        };
    }

    // Sanity checks below (even though chrono has test cases for these below we do a sanity check)
    // to verify we are passing correct duration

    #[test]
    fn minute_duration_can_be_added_to_date_time() {
        assert_correct_date_time_from_duration!("2024-04-14 08:01:00", Duration::Minute(1));
        assert_correct_date_time_from_duration!("2024-04-14 08:05:00", Duration::Minute(5));
        assert_correct_date_time_from_duration!("2024-04-14 08:10:00", Duration::Minute(10));
        assert_correct_date_time_from_duration!("2024-04-14 08:15:00", Duration::Minute(15));
        assert_correct_date_time_from_duration!("2024-04-14 08:20:00", Duration::Minute(20));
        assert_correct_date_time_from_duration!("2024-04-14 08:25:00", Duration::Minute(25));
        assert_correct_date_time_from_duration!("2024-04-14 08:30:00", Duration::Minute(30));
        assert_correct_date_time_from_duration!("2024-04-14 09:00:00", Duration::Minute(60));
        assert_correct_date_time_from_duration!("2024-04-14 09:05:00", Duration::Minute(65));
        assert_correct_date_time_from_duration!("2024-04-14 09:10:00", Duration::Minute(70));
        assert_correct_date_time_from_duration!("2024-04-14 09:30:00", Duration::Minute(90));
        assert_correct_date_time_from_duration!("2024-04-15 08:00:00", Duration::Minute(1440));
    }

    #[test]
    fn hour_duration_can_be_added_to_date_time() {
        assert_correct_date_time_from_duration!("2024-04-14 09:00:00", Duration::Hour(1));
        assert_correct_date_time_from_duration!("2024-04-14 13:00:00", Duration::Hour(5));
        assert_correct_date_time_from_duration!("2024-04-14 18:00:00", Duration::Hour(10));
        assert_correct_date_time_from_duration!("2024-04-14 23:00:00", Duration::Hour(15));
        assert_correct_date_time_from_duration!("2024-04-15 00:00:00", Duration::Hour(16));
        assert_correct_date_time_from_duration!("2024-04-15 01:00:00", Duration::Hour(17));
        assert_correct_date_time_from_duration!("2024-04-15 08:00:00", Duration::Hour(24));
        assert_correct_date_time_from_duration!("2024-04-16 08:00:00", Duration::Hour(48));
        assert_correct_date_time_from_duration!("2024-04-21 08:00:00", Duration::Hour(168));
    }

    #[test]
    fn week_duration_can_be_added_to_date_time() {
        assert_correct_date_time_from_duration!("2024-04-21 08:00:00", Duration::Week(1));
        assert_correct_date_time_from_duration!("2024-05-19 08:00:00", Duration::Week(5));
        assert_correct_date_time_from_duration!("2024-06-23 08:00:00", Duration::Week(10));
        assert_correct_date_time_from_duration!("2024-10-13 08:00:00", Duration::Week(26));
        assert_correct_date_time_from_duration!("2025-04-13 08:00:00", Duration::Week(52));
    }

    #[test]
    fn month_duration_can_be_added_to_date_time() {
        assert_correct_date_time_from_duration!("2024-05-14 08:00:00", Duration::Month(1));
        assert_correct_date_time_from_duration!("2024-06-14 08:00:00", Duration::Month(2));
        assert_correct_date_time_from_duration!("2024-07-14 08:00:00", Duration::Month(3));
        assert_correct_date_time_from_duration!("2024-08-14 08:00:00", Duration::Month(4));
        assert_correct_date_time_from_duration!("2024-09-14 08:00:00", Duration::Month(5));
        assert_correct_date_time_from_duration!("2024-10-14 08:00:00", Duration::Month(6));
        assert_correct_date_time_from_duration!("2024-11-14 08:00:00", Duration::Month(7));
        assert_correct_date_time_from_duration!("2024-12-14 08:00:00", Duration::Month(8));
        assert_correct_date_time_from_duration!("2025-01-14 08:00:00", Duration::Month(9));
        assert_correct_date_time_from_duration!("2025-02-14 08:00:00", Duration::Month(10));
        assert_correct_date_time_from_duration!("2025-03-14 08:00:00", Duration::Month(11));
        assert_correct_date_time_from_duration!("2025-04-14 08:00:00", Duration::Month(12));
        assert_correct_date_time_from_duration!("2025-05-14 08:00:00", Duration::Month(13));
    }

    #[test]
    fn year_duration_can_be_added_to_date_time() {
        assert_correct_date_time_from_duration!("2025-04-14 08:00:00", Duration::Year(1));
        assert_correct_date_time_from_duration!("2029-04-14 08:00:00", Duration::Year(5));
        assert_correct_date_time_from_duration!("2034-04-14 08:00:00", Duration::Year(10));
        assert_correct_date_time_from_duration!("2039-04-14 08:00:00", Duration::Year(15));
        assert_correct_date_time_from_duration!("2044-04-14 08:00:00", Duration::Year(20));
        assert_correct_date_time_from_duration!("2049-04-14 08:00:00", Duration::Year(25));
        assert_correct_date_time_from_duration!("2054-04-14 08:00:00", Duration::Year(30));
        assert_correct_date_time_from_duration!(
            "0000-04-14 08:00:00",
            Duration::Year((i32::MAX - 2025) as u32)
        );
    }

    #[test]
    fn minute_duration_can_be_subtracted_from_date_time() {
        assert_correct_date_time_from_sub_duration!("2024-04-14 07:59:00", Duration::Minute(1));
        assert_correct_date_time_from_sub_duration!("2024-04-14 07:55:00", Duration::Minute(5));
        assert_correct_date_time_from_sub_duration!("2024-04-14 07:50:00", Duration::Minute(10));
        assert_correct_date_time_from_sub_duration!("2024-04-14 07:45:00", Duration::Minute(15));
        assert_correct_date_time_from_sub_duration!("2024-04-14 07:40:00", Duration::Minute(20));
        assert_correct_date_time_from_sub_duration!("2024-04-14 07:35:00", Duration::Minute(25));
        assert_correct_date_time_from_sub_duration!("2024-04-14 07:30:00", Duration::Minute(30));
        assert_correct_date_time_from_sub_duration!("2024-04-14 07:00:00", Duration::Minute(60));
        assert_correct_date_time_from_sub_duration!("2024-04-14 06:55:00", Duration::Minute(65));
        assert_correct_date_time_from_sub_duration!("2024-04-14 06:50:00", Duration::Minute(70));
        assert_correct_date_time_from_sub_duration!("2024-04-14 06:30:00", Duration::Minute(90));
        assert_correct_date_time_from_sub_duration!("2024-04-13 08:00:00", Duration::Minute(1440));
    }

    #[test]
    fn hour_duration_can_be_subtracted_from_date_time() {
        assert_correct_date_time_from_sub_duration!("2024-04-14 07:00:00", Duration::Hour(1));
        assert_correct_date_time_from_sub_duration!("2024-04-14 03:00:00", Duration::Hour(5));
        assert_correct_date_time_from_sub_duration!("2024-04-13 22:00:00", Duration::Hour(10));
        assert_correct_date_time_from_sub_duration!("2024-04-13 17:00:00", Duration::Hour(15));
        assert_correct_date_time_from_sub_duration!("2024-04-13 16:00:00", Duration::Hour(16));
        assert_correct_date_time_from_sub_duration!("2024-04-13 15:00:00", Duration::Hour(17));
        assert_correct_date_time_from_sub_duration!("2024-04-13 08:00:00", Duration::Hour(24));
        assert_correct_date_time_from_sub_duration!("2024-04-12 08:00:00", Duration::Hour(48));
        assert_correct_date_time_from_sub_duration!("2024-04-07 08:00:00", Duration::Hour(168));
    }

    #[test]
    fn week_duration_can_be_subtracted_from_date_time() {
        assert_correct_date_time_from_sub_duration!("2024-04-07 08:00:00", Duration::Week(1));
        assert_correct_date_time_from_sub_duration!("2024-03-10 08:00:00", Duration::Week(5));
        assert_correct_date_time_from_sub_duration!("2024-02-04 08:00:00", Duration::Week(10));
        assert_correct_date_time_from_sub_duration!("2023-10-15 08:00:00", Duration::Week(26));
        assert_correct_date_time_from_sub_duration!("2023-04-16 08:00:00", Duration::Week(52));
    }

    #[test]
    fn month_duration_can_be_subtracted_from_date_time() {
        assert_correct_date_time_from_sub_duration!("2024-03-14 08:00:00", Duration::Month(1));
        assert_correct_date_time_from_sub_duration!("2024-02-14 08:00:00", Duration::Month(2));
        assert_correct_date_time_from_sub_duration!("2024-01-14 08:00:00", Duration::Month(3));
        assert_correct_date_time_from_sub_duration!("2023-12-14 08:00:00", Duration::Month(4));
        assert_correct_date_time_from_sub_duration!("2023-11-14 08:00:00", Duration::Month(5));
        assert_correct_date_time_from_sub_duration!("2023-10-14 08:00:00", Duration::Month(6));
        assert_correct_date_time_from_sub_duration!("2023-09-14 08:00:00", Duration::Month(7));
        assert_correct_date_time_from_sub_duration!("2023-08-14 08:00:00", Duration::Month(8));
        assert_correct_date_time_from_sub_duration!("2023-07-14 08:00:00", Duration::Month(9));
        assert_correct_date_time_from_sub_duration!("2023-06-14 08:00:00", Duration::Month(10));
        assert_correct_date_time_from_sub_duration!("2023-05-14 08:00:00", Duration::Month(11));
        assert_correct_date_time_from_sub_duration!("2023-04-14 08:00:00", Duration::Month(12));
        assert_correct_date_time_from_sub_duration!("2023-03-14 08:00:00", Duration::Month(13));
    }

    #[test]
    fn year_duration_can_be_subtracted_from_date_time() {
        assert_correct_date_time_from_sub_duration!("2023-04-14 08:00:00", Duration::Year(1));
        assert_correct_date_time_from_sub_duration!("2019-04-14 08:00:00", Duration::Year(5));
        assert_correct_date_time_from_sub_duration!("2014-04-14 08:00:00", Duration::Year(10));
        assert_correct_date_time_from_sub_duration!("2009-04-14 08:00:00", Duration::Year(15));
        assert_correct_date_time_from_sub_duration!("2004-04-14 08:00:00", Duration::Year(20));
        assert_correct_date_time_from_sub_duration!("1999-04-14 08:00:00", Duration::Year(25));
        assert_correct_date_time_from_sub_duration!("1994-04-14 08:00:00", Duration::Year(30));
        assert_correct_date_time_from_sub_duration!(
            "0000-04-14 08:00:00",
            Duration::Year((i32::MAX - 2025) as u32)
        );
    }
}
