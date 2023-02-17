use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date};

/// A utility class for serializing / deserializing dates.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DateString(String);

impl From<Date> for DateString {
    fn from(date: Date) -> Self {
        Self(date.to_string())
    }
}

impl TryFrom<DateString> for Date {
    type Error = time::error::Parse;

    fn try_from(value: DateString) -> Result<Self, Self::Error> {
        let format = format_description!("[year]-[month]-[day]");
        let date = Date::parse(&value.0, &format)?;

        Ok(date)
    }
}
