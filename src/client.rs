use std::{fs::File, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    address::MailingAddress,
    cli::{print_header, YamlValidator},
    completion::{LocalAutocompleter, PrefixAutocomplete},
    config::Config,
    contact::ContactInfo,
    id::Id,
    storage::get_clients_dir,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    pub name: Id,
    pub address: MailingAddress,
    pub contact: ContactInfo,
}

impl Client {
    pub fn edit_yaml(&self) -> anyhow::Result<Self> {
        let yaml = serde_yaml::to_string(&self).context("serializing client yaml")?;

        // TODO: Show as markdown code block via termimad
        print_header("Final YAML");
        println!("{}", yaml);

        let yaml_validator = YamlValidator::<Client>::new();

        let edited = inquire::Editor::new("Edit...")
            .with_predefined_text(&yaml)
            .with_validator(yaml_validator)
            .with_file_extension(".yaml")
            .prompt()
            .context("reading edited client yaml from user input")?;

        let parsed = serde_yaml::from_str(&edited).context("parsing edited client yaml")?;

        Ok(parsed)
    }

    pub fn get_or_create_from_user_input(config: &Config) -> anyhow::Result<Id> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let client_names = Client::list(config).context("listing clients")?;
        let autocomplete = LocalAutocompleter::new(ClientAutocomplete::new(client_names.clone()));

        let name: Id = inquire::Text::new("Client Name:")
            .with_autocomplete(autocomplete)
            .with_validator(required_validator)
            .prompt()
            .context("reading client name from user input")?
            .into();

        if !client_names.contains(&name) {
            let client = Client::create_from_user_input_with_name(name.clone())
                .context("creating client from user input")?;
            client.save(config).context("saving client yaml file")?;
        };

        Ok(name)
    }

    pub fn create_from_user_input_with_name(name: Id) -> anyhow::Result<Self> {
        println!("Mailing address:");
        let address = MailingAddress::create_from_user_input()
            .context("creating mailing address from user input")?;

        println!("Contact info:");
        let contact = ContactInfo::create_from_user_input()
            .context("creating contact info from user input")?;

        let mut client = Self {
            name,
            address,
            contact,
        };

        client = client.edit_yaml().context("editing client yaml")?;

        Ok(client)
    }

    pub fn filename(&self) -> String {
        self.name.to_filename()
    }

    pub fn save(&self, config: &Config) -> anyhow::Result<()> {
        let clients_dir = get_clients_dir(config).context("getting clients directory")?;
        let path = clients_dir.join(self.filename());
        let file = File::create(path).context("creating client yaml file")?;

        serde_yaml::to_writer(file, self).context("serializing client yaml")?;

        Ok(())
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref()).context("opening client yaml file")?;
        let client: Client = serde_yaml::from_reader(file).context("deserializing client yaml")?;

        Ok(client)
    }

    pub fn load(name: Id, config: &Config) -> anyhow::Result<Self> {
        let clients_dir = get_clients_dir(config).context("getting clients directory")?;
        let filename = name.to_filename();
        let path = clients_dir.join(filename);
        let client = Client::load_from_path(path).context("loading client from file")?;

        Ok(client)
    }

    pub fn list(config: &Config) -> anyhow::Result<Vec<Id>> {
        let clients_dir = get_clients_dir(config).context("getting clients directory")?;

        let client_names = clients_dir
            .read_dir()
            .context("listing files in clients directory")?
            .map(|entry_res| -> anyhow::Result<Id> {
                let entry = entry_res.context("reading directory entry")?;
                let filename = entry.file_name();
                let name =
                    Id::from_filename(filename).context("parsing client id from filename")?;
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
