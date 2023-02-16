use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContactInfo {
    pub email: String,
    pub phone: String,
}

impl ContactInfo {
    pub fn create_from_user_input() -> anyhow::Result<Self> {
        let email = inquire::Text::new("Email:").prompt()?;

        let phone = inquire::Text::new("Phone:").prompt()?;

        let contact = Self { email, phone };

        Ok(contact)
    }
}
