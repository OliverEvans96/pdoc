use std::{fs::File, path::Path};

use serde::{Deserialize, Serialize};

use crate::{address::MailingAddress, contact::ContactInfo, id::Id, storage::get_clients_dir};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Client {
    pub name: Id,
    pub address: MailingAddress,
    pub contact: ContactInfo,
}

impl Client {
    pub fn create_from_user_input() -> anyhow::Result<Self> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let name: Id = inquire::Text::new("Client Name:")
            .with_placeholder("Acme Co.")
            .with_validator(required_validator)
            .prompt()?
            .into();

        println!("Mailing address:");
        let address = MailingAddress::create_from_user_input()?;

        println!("Contact info:");
        let contact = ContactInfo::create_from_user_input()?;

        let client = Self {
            name,
            address,
            contact,
        };

        Ok(client)
    }

    pub fn filename(&self) -> String {
        self.name.to_filename()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let clients_dir = get_clients_dir()?;
        let path = clients_dir.join(self.filename());
        let file = File::create(path)?;

        serde_yaml::to_writer(file, self)?;

        Ok(())
    }

    pub fn read(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref())?;
        let client: Client = serde_yaml::from_reader(file)?;

        Ok(client)
    }

    pub fn list() -> anyhow::Result<Vec<Id>> {
        let clients_dir = get_clients_dir()?;

        let client_names = clients_dir
            .read_dir()?
            .map(|entry_res| -> anyhow::Result<Id> {
                let entry = entry_res?;
                let filename = entry.file_name();
                let name = Id::from_filename(filename)?;
                Ok(name)
            })
            // Ignore any filenames that could not be parsed
            .filter_map(|res| res.ok())
            .collect();

        Ok(client_names)
    }
}
