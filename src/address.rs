use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MailingAddress {
    pub addr1: String,
    pub addr2: Option<String>,
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
            .prompt()?;

        let addr2 = inquire::Text::new("Address Line 2 (optional):")
            .with_placeholder("Apt. 7")
            .prompt_skippable()?
            // Convert Some("") to None
            .filter(|line| !line.is_empty());

        let city = inquire::Text::new("City:")
            .with_placeholder("Springfield")
            .with_validator(required_validator.clone())
            .prompt()?;

        let state = inquire::Text::new("State:")
            .with_placeholder("Ohio")
            .prompt()?;

        let zip = inquire::Text::new("Zipcode:")
            .with_placeholder("12345")
            .with_validator(required_validator)
            .prompt()?;

        let contact = Self {
            addr1,
            addr2,
            city,
            state,
            zip,
        };

        Ok(contact)
    }
}
