//! Small dependency-free local calendar date used by persistence boundaries.

use std::{fmt, str::FromStr};

/// Valid Gregorian calendar date in the inclusive year range 1–9999.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LocalDate {
    year: i32,
    month: u8,
    day: u8,
}

impl LocalDate {
    /// Creates a validated date.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, DateError> {
        if !(1..=9_999).contains(&year) || !(1..=12).contains(&month) {
            return Err(DateError::InvalidDate);
        }
        let maximum_day = days_in_month(year, month);
        if day == 0 || day > maximum_day {
            return Err(DateError::InvalidDate);
        }
        Ok(Self { year, month, day })
    }

    /// Calendar year.
    #[must_use]
    pub const fn year(self) -> i32 {
        self.year
    }

    /// Calendar month from 1 through 12.
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }

    /// Day of month from 1 through 31.
    #[must_use]
    pub const fn day(self) -> u8 {
        self.day
    }

    /// Returns the preceding calendar day, or `None` at the supported boundary.
    #[must_use]
    pub fn previous_day(self) -> Option<Self> {
        if self.day > 1 {
            return Self::new(self.year, self.month, self.day - 1).ok();
        }
        if self.month > 1 {
            let month = self.month - 1;
            return Self::new(self.year, month, days_in_month(self.year, month)).ok();
        }
        let year = self.year.checked_sub(1)?;
        Self::new(year, 12, 31).ok()
    }

    /// Returns the following calendar day, or `None` at the supported boundary.
    #[must_use]
    pub fn next_day(self) -> Option<Self> {
        if self.day < days_in_month(self.year, self.month) {
            return Self::new(self.year, self.month, self.day + 1).ok();
        }
        if self.month < 12 {
            return Self::new(self.year, self.month + 1, 1).ok();
        }
        let year = self.year.checked_add(1)?;
        Self::new(year, 1, 1).ok()
    }
}

impl fmt::Display for LocalDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

impl FromStr for LocalDate {
    type Err = DateError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split('-');
        let year = parts
            .next()
            .ok_or(DateError::InvalidFormat)?
            .parse()
            .map_err(|_| DateError::InvalidFormat)?;
        let month = parts
            .next()
            .ok_or(DateError::InvalidFormat)?
            .parse()
            .map_err(|_| DateError::InvalidFormat)?;
        let day = parts
            .next()
            .ok_or(DateError::InvalidFormat)?
            .parse()
            .map_err(|_| DateError::InvalidFormat)?;
        if parts.next().is_some() || value.len() != 10 {
            return Err(DateError::InvalidFormat);
        }
        Self::new(year, month, day)
    }
}

const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Invalid calendar date or textual representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DateError {
    /// Components do not form a supported Gregorian date.
    InvalidDate,
    /// Text is not exactly `YYYY-MM-DD`.
    InvalidFormat,
}

impl fmt::Display for DateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDate => write!(formatter, "invalid local calendar date"),
            Self::InvalidFormat => write!(formatter, "date must use YYYY-MM-DD"),
        }
    }
}

impl std::error::Error for DateError {}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{DateError, LocalDate};

    #[test]
    fn parses_formats_and_rolls_over_boundaries() {
        let leap = LocalDate::from_str("2024-02-29").unwrap();
        assert_eq!(leap.to_string(), "2024-02-29");
        assert_eq!(leap.next_day().unwrap().to_string(), "2024-03-01");
        assert_eq!(leap.previous_day().unwrap().to_string(), "2024-02-28");
        assert_eq!(
            LocalDate::new(2026, 1, 1)
                .unwrap()
                .previous_day()
                .unwrap()
                .to_string(),
            "2025-12-31"
        );
    }

    #[test]
    fn rejects_invalid_dates_and_formats() {
        assert_eq!(LocalDate::new(2025, 2, 29), Err(DateError::InvalidDate));
        assert_eq!(
            LocalDate::from_str("2025-2-01"),
            Err(DateError::InvalidFormat)
        );
    }
}
