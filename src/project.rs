use serde::{Deserialize, Serialize};

use crate::{client::ClientAutocomplete, id::Id};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub name: Id,
    pub description: String,
    pub client_ref: Id,
}

impl Project {
    pub fn create_from_user_input() -> anyhow::Result<()> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        // let name: Id = inquire::Text::new("Project Name:")
        //     .with_placeholder("Save the Earth")
        //     .with_validator(required_validator.clone())
        //     .prompt()?
        //     .into();

        let autocomplete = ClientAutocomplete::try_new()?;

        let client = inquire::Text::new("Client Name:")
            .with_autocomplete(autocomplete)
            .with_validator(required_validator)
            .prompt()?;

        println!("Client: {}", client);

        Ok(())
    }
}
