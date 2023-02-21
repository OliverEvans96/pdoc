use std::{fs::File, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    address::MailingAddress,
    cli::{print_header, YamlValidator},
    contact::ContactInfo,
    storage::get_data_dir,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PaymentMethod {
    Text(String),
    Link { text: String, url: String },
}

impl PaymentMethod {
    pub fn create_from_user_input() -> anyhow::Result<Option<Self>> {
        let maybe_description = inquire::Text::new("Payment method:")
            .prompt_skippable()?
            // Convert Some("") to None
            .filter(|line| !line.is_empty());

        if let Some(description) = maybe_description {
            let maybe_url = inquire::Text::new("Add link URL?")
                .prompt_skippable()?
                // Convert Some("") to None
                .filter(|line| !line.is_empty());

            let method = if let Some(url) = maybe_url {
                PaymentMethod::Link {
                    text: description,
                    url,
                }
            } else {
                PaymentMethod::Text(description)
            };

            Ok(Some(method))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Me {
    pub name: String,
    pub address: MailingAddress,
    pub contact: ContactInfo,
    pub payment: Vec<PaymentMethod>,
}

impl Me {
    pub fn edit_yaml(&self) -> anyhow::Result<Self> {
        let yaml = serde_yaml::to_string(&self)?;

        // TODO: Show as markdown code block via termimad
        print_header("Final YAML");
        println!("{}", yaml);

        let yaml_validator = YamlValidator::<Me>::new();

        let edited = inquire::Editor::new("Edit...")
            .with_predefined_text(&yaml)
            .with_validator(yaml_validator)
            .with_file_extension(".yaml")
            .prompt()?;

        let parsed = serde_yaml::from_str(&edited)?;

        Ok(parsed)
    }
    pub fn create_from_user_input() -> anyhow::Result<Me> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let name: String = inquire::Text::new("Your Name:")
            .with_validator(required_validator)
            .prompt()?
            .into();

        println!("Mailing address:");
        let address = MailingAddress::create_from_user_input()?;

        println!("Contact info:");
        let contact = ContactInfo::create_from_user_input()?;

        let mut payment_methods = Vec::new();
        while let Some(method) = PaymentMethod::create_from_user_input()? {
            payment_methods.push(method);
        }

        let mut me = Self {
            name,
            address,
            contact,
            payment: payment_methods,
        };

        me = me.edit_yaml()?;

        Ok(me)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let data_dir = get_data_dir()?;
        std::fs::create_dir_all(&data_dir)?;
        let path = data_dir.join("me.yaml");
        let file = File::create(path)?;

        serde_yaml::to_writer(file, self)?;

        Ok(())
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref())?;
        let me: Me = serde_yaml::from_reader(file)?;

        Ok(me)
    }

    pub fn load() -> anyhow::Result<Self> {
        let data_dir = get_data_dir()?;
        let filename = "me.yaml";
        let path = data_dir.join(filename);
        let client = Me::load_from_path(path)?;

        Ok(client)
    }
}
