use std::{fs::File, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    address::MailingAddress,
    completion::{LocalAutocompleter, PrefixAutocomplete},
    contact::ContactInfo,
    id::Id,
    storage::get_clients_dir,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Client {
    pub name: Id,
    pub address: MailingAddress,
    pub contact: ContactInfo,
}

impl Client {
    pub fn get_or_create_from_user_input() -> anyhow::Result<Id> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let client_names = Client::list()?;
        let autocomplete = LocalAutocompleter::new(ClientAutocomplete::new(client_names.clone()));

        let name: Id = inquire::Text::new("Client Name:")
            .with_autocomplete(autocomplete)
            .with_validator(required_validator)
            .prompt()?
            .into();

        if !client_names.contains(&name) {
            let client = Client::create_from_user_input_with_name(name.clone())?;
            client.save()?;
        };

        Ok(name)
    }

    pub fn create_from_user_input_with_name(name: Id) -> anyhow::Result<Self> {
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

    pub fn load_from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref())?;
        let client: Client = serde_yaml::from_reader(file)?;

        Ok(client)
    }

    pub fn load(name: Id) -> anyhow::Result<Self> {
        let clients_dir = get_clients_dir()?;
        let filename = name.to_filename();
        let path = clients_dir.join(filename);
        let client = Client::load_from_path(path)?;

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

#[derive(Clone, Debug)]
pub struct ClientAutocomplete {
    client_names: Vec<String>,
    lowercase_names: Vec<String>,
}

impl ClientAutocomplete {
    pub fn new(client_ids: Vec<Id>) -> Self {
        let client_names: Vec<String> = client_ids.into_iter().map(Into::into).collect();

        let lowercase_names = client_names.iter().map(|s| s.to_lowercase()).collect();

        Self {
            client_names,
            lowercase_names,
        }
    }
}

impl PrefixAutocomplete for ClientAutocomplete {
    fn get_options(&self) -> &[String] {
        &self.client_names
    }

    fn get_lowercase_options(&self) -> &[String] {
        &self.lowercase_names
    }
}
