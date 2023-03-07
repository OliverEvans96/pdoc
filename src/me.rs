use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Context;
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
            .prompt_skippable()
            .context("reading payment method from user input")?
            // Convert Some("") to None
            .filter(|line| !line.is_empty());

        if let Some(description) = maybe_description {
            let maybe_url = inquire::Text::new("Add link URL?")
                .prompt_skippable()
                .context("reading link URL from user input")?
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
        let yaml = serde_yaml::to_string(&self).context("serializing personal info yaml")?;

        // TODO: Show as markdown code block via termimad
        print_header("Final YAML");
        println!("{}", yaml);

        let yaml_validator = YamlValidator::<Me>::new();

        let edited = inquire::Editor::new("Edit...")
            .with_predefined_text(&yaml)
            .with_validator(yaml_validator)
            .with_file_extension(".yaml")
            .prompt()
            .context("reading edited personal info yaml from user input")?;

        let parsed = serde_yaml::from_str(&edited).context("parsing edited personal info yaml")?;

        Ok(parsed)
    }

    pub fn create_from_user_input() -> anyhow::Result<Me> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let name: String = inquire::Text::new("Your Name:")
            .with_validator(required_validator)
            .prompt()
            .context("reading name from user input")?
            .into();

        println!("Mailing address:");
        let address = MailingAddress::create_from_user_input()
            .context("reading mailing address from user input")?;

        println!("Contact info:");
        let contact = ContactInfo::create_from_user_input()
            .context("reading contact info from user input")?;

        println!("Acceptable payment methods:");
        let mut payment_methods = Vec::new();
        while let Some(method) = PaymentMethod::create_from_user_input()
            .context("creating payment method from user input")?
        {
            payment_methods.push(method);
        }

        let mut me = Self {
            name,
            address,
            contact,
            payment: payment_methods,
        };

        me = me.edit_yaml().context("editing personal info yaml")?;

        Ok(me)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let data_dir = get_data_dir().context("getting data directory")?;
        std::fs::create_dir_all(&data_dir).context("creating data directory")?;
        let path = data_dir.join("me.yaml");
        let file = File::create(path).context("opening personal info yaml file")?;

        serde_yaml::to_writer(file, self).context("serializing personal info yaml")?;

        Ok(())
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref()).context("opening personal info yaml file")?;
        let me: Me = serde_yaml::from_reader(file).context("deserializing personal info yaml")?;

        Ok(me)
    }

    pub fn path() -> anyhow::Result<PathBuf> {
        let data_dir = get_data_dir().context("getting data directory")?;
        let filename = "me.yaml";
        let path = data_dir.join(filename);

        Ok(path)
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Me::path()?;
        let me = Me::load_from_path(path).context("loading personal info from file")?;

        Ok(me)
    }

    pub fn create_if_necessary() -> anyhow::Result<()> {
        let file_exists = !Me::path()
            .context("getting personal info yaml path")?
            .exists();

        if file_exists {
            println!("Welcome to pdoc! Enter your personal info to begin.");
            let me = Me::create_from_user_input().context("creating personal info")?;
            me.save().context("saving personal info")?
        } else {
            if let Err(err) = Me::load() {
                eprintln!("Error parsing personal info: {:?}", err);
                eprintln!("\nPlease correct the issue before continuing.");
                std::process::exit(1);
            }
        }

        Ok(())
    }
}
