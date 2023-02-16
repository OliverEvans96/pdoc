use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{address::MailingAddress, contact::ContactInfo, id::Id};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Client {
    pub id: Id,
    pub name: String,
    pub address: MailingAddress,
    pub contact: ContactInfo,
}

impl Client {
    pub fn create_from_user_input() -> anyhow::Result<Self> {
        let mut rng = thread_rng();

        let id: Id = rng.gen();

        let name = inquire::Text::new("Client Name:").prompt()?;

        println!("Mailing address:");
        let address = MailingAddress::create_from_user_input()?;

        println!("Contact info:");
        let contact = ContactInfo::create_from_user_input()?;

        let client = Self {
            id,
            name,
            address,
            contact,
        };

        Ok(client)
    }
}
