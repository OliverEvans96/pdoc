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
        let addr1 = inquire::Text::new("Address Line 1:").prompt()?;

        let addr2 = inquire::Text::new("Address Line 2:").prompt_skippable()?;

        let city = inquire::Text::new("City:").prompt()?;

        let state = inquire::Text::new("State:").prompt()?;

        let zip = inquire::Text::new("Zipcode:").prompt()?;

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
