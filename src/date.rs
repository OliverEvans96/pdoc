use std::fmt::Display;

use serde::{Deserialize, Serialize};
use time::{format_description::FormatItem, macros::format_description, Date};

/// A utility class for serializing / deserializing dates.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DateString(String);

const SERDE_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day]");

impl DateString {
    pub fn try_new(s: String) -> Result<Self, time::error::Parse> {
        // Make sure date string can be parsed before continuing.
        _ = Date::parse(&s, SERDE_FORMAT)?;

        Ok(Self(s))
    }

    pub fn to_beancount_owned(self) -> beancount_core::Date<'static> {
        beancount_core::Date::from_string_unchecked(self.to_string())
    }

    pub fn to_beancount<'a>(&'a self) -> beancount_core::Date<'a> {
        beancount_core::Date::from_str_unchecked(&self.0)
    }
}

impl From<Date> for DateString {
    fn from(date: Date) -> Self {
        Self(date.to_string())
    }
}

impl Display for DateString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let date: Date = self
            .clone()
            .try_into()
            .expect("DateString should be parseable w/ SERDE_FORMAT");

        let display_format = format_description!("[month repr:long] [day padding:none], [year]");
        let s = date
            .format(&display_format)
            .map_err(|_err| std::fmt::Error)?;

        f.write_str(&s)
    }
}

impl TryFrom<DateString> for Date {
    type Error = time::error::Parse;

    fn try_from(value: DateString) -> Result<Self, Self::Error> {
        let date = Date::parse(&value.0, &SERDE_FORMAT)?;

        Ok(date)
    }
}
