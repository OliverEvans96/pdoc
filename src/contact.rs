use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContactInfo {
    pub email: String,
    pub phone: String,
}

impl ContactInfo {
    pub fn create_from_user_input() -> anyhow::Result<Self> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let email = inquire::Text::new("Email:")
            .with_placeholder("test@example.com")
            .with_validator(required_validator.clone())
            .prompt()
            .context("reading email from user input")?;

        let phone = inquire::Text::new("Phone:")
            .with_placeholder("(412) 555-1827")
            .with_validator(required_validator)
            .prompt()
            .context("reading phone number from user input")?;

        let contact = Self { email, phone };

        Ok(contact)
    }
}
