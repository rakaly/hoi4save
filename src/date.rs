use std::cmp::Ordering;
use std::convert::TryFrom;

use jomini::Scalar;

pub type Hoi4Date = Date;
const DAYS_PER_MONTH: [u8; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Struct specialized to parsing, formatting, and manipulating dates in games
///
/// A game date does not follow any traditional calendar and instead views the
/// world on simpler terms: that every year should be treated as a non-leap year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    year: i16,
    month: u8,
    day: u8,
    hour: u8,
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Date) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Date) -> Ordering {
        self.year
            .cmp(&other.year)
            .then_with(|| self.month.cmp(&other.month))
            .then_with(|| self.day.cmp(&other.day))
            .then_with(|| self.hour.cmp(&other.hour))
    }
}

impl Date {
    pub fn new(year: i16, month: u8, day: u8, hour: u8) -> Option<Self> {
        if year != 0 && month != 0 && day != 0 && year > -100 && hour != 0 && hour < 25 {
            if let Some(&days) = DAYS_PER_MONTH.get(usize::from(month)) {
                if day <= days {
                    return Some(Date { year, month, day, hour });
                }
            }
        }

        None
    }

    /// Year of the date
    pub fn year(&self) -> i16 {
        self.year
    }

    /// Month of the date
    pub fn month(&self) -> u8 {
        self.month
    }

    /// Day of the date
    pub fn day(&self) -> u8 {
        self.day
    }

    /// Day of the date
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Parses a string and returns a new Date if valid.
    pub fn parse_from_str<T: AsRef<str>>(s: T) -> Option<Self> {
        let data = s.as_ref().as_bytes();
        let mut state = 0;
        let mut span1: &[u8] = &[];
        let mut span2: &[u8] = &[];
        let mut span3: &[u8] = &[];
        let mut start = 0;

        // micro-optimization: check the first byte to see if the first character (if available)
        // is outside our upper bound (ie: not a number). This micro optimization doesn't
        // harm the happy path (input is a date) by more than a few percent, but if the input
        // is not a date, this shaves off 20-25% in date parsing benchmarks.
        if data.get(0).map_or(true, |c| *c > b'9') {
            return None;
        }

        for (pos, &c) in data.iter().enumerate() {
            if c == b'.' {
                match state {
                    0 => {
                        span1 = &data[start..pos];
                        state = 1;
                    }
                    1 => {
                        span2 = &data[start..pos];
                        state = 2;
                    }
                    2 => {
                        span3 = &data[start..pos];
                        state = 3;
                    }
                    _ => return None,
                }
                start = pos + 1;
            }
        }

        let span4 = &data[start..];

        let year = Scalar::new(span1)
            .to_i64()
            .ok()
            .and_then(|x| i16::try_from(x).ok());

        let year = match year {
            Some(x) => x,
            None => return None,
        };

        let month = Scalar::new(span2)
            .to_u64()
            .ok()
            .and_then(|x| u8::try_from(x).ok());

        let month = match month {
            Some(x) => x,
            None => return None,
        };

        let day = Scalar::new(span3)
            .to_u64()
            .ok()
            .and_then(|x| u8::try_from(x).ok());

        let day = match day {
            Some(x) => x,
            None => return None,
        };

        let hour = Scalar::new(span4)
            .to_u64()
            .ok()
            .and_then(|x| u8::try_from(x).ok());

        let hour = match hour {
            Some(x) => x,
            None => return None,
        };

        Date::new(year, month, day, hour)
    }

    fn days(&self) -> i32 {
        let month_days = match self.month {
            1 => -1,
            2 => 30,
            3 => 58,
            4 => 89,
            5 => 119,
            6 => 150,
            7 => 180,
            8 => 211,
            9 => 242,
            10 => 272,
            11 => 303,
            12 => 333,
            _ => unreachable!(),
        };

        let year_day = i32::from(self.year) * 365;
        if year_day < 0 {
            year_day - month_days - i32::from(self.day)
        } else {
            year_day + month_days + i32::from(self.day)
        }
    }

    /// Returns the number of days between two dates
    pub fn days_until(&self, other: &Date) -> i32 {
        other.days() - self.days()
    }

    /// Return a new date that is the given number of days in the future
    /// from the current date
    ///
    /// Will panic on overflow or underflow.
    pub fn add_days(&self, days: i32) -> Date {
        let new_days = self
            .days()
            .checked_add(days)
            .expect("adding days overflowed");

        let days_since_jan1 = (new_days % 365).abs();
        let year = new_days / 365;
        let (month, day) = month_day_from_julian(days_since_jan1);

        let year = i16::try_from(year).expect("year to fit inside signed 32bits");
        Date { year, month, day, hour: self.hour, }
    }

    /// Decodes a date from a number that had been parsed from binary data
    pub fn from_binary(mut s: i32) -> Option<Self> {
        if s < 0 {
            return None
        }

        let hours = (s % 24) as u8 + 1;
        s /= 24;
        let days_since_jan1 = s % 365;
        s /= 365;
        let year = match s.checked_sub(5000).and_then(|x| i16::try_from(x).ok()) {
            Some(y) => y,
            None => return None,
        };

        let (month, day) = month_day_from_julian(days_since_jan1);
        Date::new(year, month, day, hours)
    }

    /// Formats a date in the ISO 8601 format: YYYY-MM-DD
    pub fn iso_8601(&self) -> String {
        format!("{:04}-{:02}-{:02}T{:02}", self.year, self.month, self.day, self.hour)
    }

    /// Formats a date in the game format: Y.M.D
    pub fn game_fmt(&self) -> String {
        format!("{}.{}.{}.{}", self.year, self.month, self.day, self.hour)
    }
}

fn month_day_from_julian(days_since_jan1: i32) -> (u8, u8) {
    // https://landweb.modaps.eosdis.nasa.gov/browse/calendar.html
    // except we start at 0 instead of 1
    let (month, day) = match days_since_jan1 {
        0..=30 => (1, days_since_jan1 + 1),
        31..=58 => (2, days_since_jan1 - 30),
        59..=89 => (3, days_since_jan1 - 58),
        90..=119 => (4, days_since_jan1 - 89),
        120..=150 => (5, days_since_jan1 - 119),
        151..=180 => (6, days_since_jan1 - 150),
        181..=211 => (7, days_since_jan1 - 180),
        212..=242 => (8, days_since_jan1 - 211),
        243..=272 => (9, days_since_jan1 - 242),
        273..=303 => (10, days_since_jan1 - 272),
        304..=333 => (11, days_since_jan1 - 303),
        334..=364 => (12, days_since_jan1 - 333),
        _ => unreachable!(),
    };

    debug_assert!(day < 255);
    (month, day as u8)
}

#[cfg(feature = "derive")]
mod datederive {
    use super::Date;
    use serde::{de, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt;

    impl Serialize for Date {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(self.iso_8601().as_str())
        }
    }

    struct DateVisitor;

    impl<'de> Visitor<'de> for DateVisitor {
        type Value = Date;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a date")
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Date::from_binary(v)
                .ok_or_else(|| de::Error::custom(format!("invalid binary date: {}", v)))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Date::parse_from_str(v).ok_or_else(|| de::Error::custom(format!("invalid date: {}", v)))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(v.as_str())
        }
    }

    impl<'de> Deserialize<'de> for Date {
        fn deserialize<D>(deserializer: D) -> Result<Date, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(DateVisitor)
        }
    }
}

#[cfg(not(feature = "derive"))]
mod datederive {}
