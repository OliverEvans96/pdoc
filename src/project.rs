use serde::{Deserialize, Serialize};

use crate::id::Id;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub name: Id,
    pub description: String,
    pub client_ref: Id,
}

impl Project {
    pub fn create_from_user_input() -> anyhow::Result<Self> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let name: Id = inquire::Text::new("Project Name:")
            .with_placeholder("Save the Earth")
            .with_validator(required_validator)
            .prompt()?
            .into();

        todo!()
    }
}
