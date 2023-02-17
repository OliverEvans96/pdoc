use serde::{Deserialize, Serialize};

use crate::{
    client::{Client, ClientAutocomplete},
    id::Id,
};

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

        let client_names = Client::list()?;
        let autocomplete = ClientAutocomplete::new(client_names.clone());

        let client_name: Id = inquire::Text::new("Client Name:")
            .with_autocomplete(autocomplete)
            .with_validator(required_validator)
            .prompt()?
            .into();

        if client_names.contains(&client_name) {
            println!("Existing client: {}", client_name);
        } else {
            println!("New client: {}", client_name);
        }

        Ok(())
    }
}
