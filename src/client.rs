use std::fs::File;

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{address::MailingAddress, contact::ContactInfo, id::Id, storage::get_clients_dir};

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

        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let name = inquire::Text::new("Client Name:")
            .with_placeholder("Acme Co.")
            .with_validator(required_validator)
            .prompt()?;

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

    pub fn filename(&self) -> String {
        self.id.to_filename()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let clients_dir = get_clients_dir()?;
        let path = clients_dir.join(self.filename());
        let file = File::create(path)?;

        serde_yaml::to_writer(file, self)?;

        Ok(())
    }
}