use std::fmt::Display;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use time::{Date, Duration, Weekday};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DayKind {
    Calendar,
    Business,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PaymentTerms {
    pub days: u16,
    pub day_kind: DayKind,
}

struct DayKindOption {
    value: DayKind,
    label: String,
}

impl Display for DayKindOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

impl PaymentTerms {
    pub fn prompt_from_user() -> anyhow::Result<Self> {
        let day_kind = inquire::Select::new("Days to pay:", Self::day_kind_options())
            .prompt()
            .context("reading day kind from user input")?
            .value;

        Self::prompt_days(day_kind)
    }

    /// Skippable Select; returns `None` if the user skips the day-kind prompt.
    pub fn prompt_from_user_optional() -> anyhow::Result<Option<Self>> {
        match Self::prompt_day_kind_skippable()? {
            Some(day_kind) => Ok(Some(Self::prompt_days(day_kind)?)),
            None => Ok(None),
        }
    }

    fn day_kind_options() -> Vec<DayKindOption> {
        vec![
            DayKindOption {
                value: DayKind::Calendar,
                label: "Calendar days".to_owned(),
            },
            DayKindOption {
                value: DayKind::Business,
                label: "Business days".to_owned(),
            },
        ]
    }

    fn prompt_day_kind_skippable() -> anyhow::Result<Option<DayKind>> {
        inquire::Select::new("Default days to pay (optional):", Self::day_kind_options())
            .prompt_skippable()
            .context("reading day kind from user input")
            .map(|opt| opt.map(|choice| choice.value))
    }

    fn prompt_days(day_kind: DayKind) -> anyhow::Result<Self> {
        let days = inquire::CustomType::<u16>::new("Days to pay:")
            .with_default(7)
            .prompt()
            .context("reading days-to-pay from user input")?;

        Ok(Self { days, day_kind })
    }
}

pub fn compute_due_date(invoice_date: Date, terms: &PaymentTerms) -> Date {
    match terms.day_kind {
        DayKind::Calendar => invoice_date + Duration::days(terms.days.into()),
        DayKind::Business => add_business_days(invoice_date, terms.days),
    }
}

fn is_business_day(date: Date) -> bool {
    !matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday)
}

fn add_business_days(start: Date, days: u16) -> Date {
    let mut date = start;
    let mut remaining = days;

    while remaining > 0 {
        date += Duration::days(1);
        if is_business_day(date) {
            remaining -= 1;
        }
    }

    date
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::{compute_due_date, DayKind, PaymentTerms};

    #[test]
    fn calendar_days_match_duration_add() {
        let invoice_date = date!(2023 - 02 - 17);
        let terms = PaymentTerms {
            days: 7,
            day_kind: DayKind::Calendar,
        };

        assert_eq!(
            compute_due_date(invoice_date, &terms),
            date!(2023 - 02 - 24)
        );
    }

    #[test]
    fn one_business_day_from_friday_is_monday() {
        let invoice_date = date!(2023 - 02 - 17);
        let terms = PaymentTerms {
            days: 1,
            day_kind: DayKind::Business,
        };

        assert_eq!(
            compute_due_date(invoice_date, &terms),
            date!(2023 - 02 - 20)
        );
    }

    #[test]
    fn five_business_days_from_friday() {
        let invoice_date = date!(2023 - 02 - 17);
        let terms = PaymentTerms {
            days: 5,
            day_kind: DayKind::Business,
        };

        assert_eq!(
            compute_due_date(invoice_date, &terms),
            date!(2023 - 02 - 24)
        );
    }
}
