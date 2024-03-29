use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MailingAddress {
    pub addr1: String,
    pub addr2: Option<String>,
    pub addr3: Option<String>,
    pub city: String,
    pub state: String,
    pub zip: String,
}

impl MailingAddress {
    pub fn create_from_user_input() -> anyhow::Result<Self> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let addr1 = inquire::Text::new("Address Line 1:")
            .with_placeholder("123 Happy Lane")
            .with_validator(required_validator.clone())
            .prompt()
            .context("reading address line 1 from user input")?;

        let addr2 = inquire::Text::new("Address Line 2 (optional):")
            .with_placeholder("Apt. 7")
            .prompt_skippable()
            .context("reading address line 2 from user input")?
            // Convert Some("") to None
            .filter(|line| !line.is_empty());

        let addr3 = if addr2.is_some() {
            inquire::Text::new("Address Line 3 (optional):")
                .with_placeholder("Closet under the stairs")
                .prompt_skippable()
                .context("reading address line 3 from user input")?
                // Convert Some("") to None
                .filter(|line| !line.is_empty())
        } else {
            None
        };

        let city = inquire::Text::new("City:")
            .with_placeholder("Springfield")
            .with_validator(required_validator.clone())
            .prompt()
            .context("reading city from user input")?;

        let state = inquire::Text::new("State:")
            .with_placeholder("Ohio")
            .prompt()
            .context("reading state from user input")?;

        let zip = inquire::Text::new("Zipcode:")
            .with_placeholder("12345")
            .with_validator(required_validator)
            .prompt()
            .context("reading zipcode from user input")?;

        let contact = Self {
            addr1,
            addr2,
            addr3,
            city,
            state,
            zip,
        };

        Ok(contact)
    }
}
