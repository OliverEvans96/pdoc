use std::fs::File;

use inquire::autocompletion::Replacement;
use serde::{Deserialize, Serialize};

use crate::{
    client::{Client, ClientAutocomplete},
    id::Id,
    storage::get_projects_dir,
};

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
            .with_validator(required_validator.clone())
            .prompt()?
            .into();

        Self::create_from_user_input_with_name(name)
    }

    pub fn create_from_user_input_with_name(name: Id) -> anyhow::Result<Self> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let description = inquire::Text::new("Project Description:")
            .with_placeholder("Doing what we can, one day at a time.")
            .with_validator(required_validator.clone())
            .prompt()?;

        let client_names = Client::list()?;
        let autocomplete = ClientAutocomplete::new(client_names.clone());

        let client_name: Id = inquire::Text::new("Client Name:")
            .with_autocomplete(autocomplete)
            .with_validator(required_validator)
            .prompt()?
            .into();

        if !client_names.contains(&client_name) {
            let client = Client::create_from_user_input_with_name(client_name.clone())?;
            client.save()?;
        };

        let project = Self {
            name,
            description,
            client_ref: client_name,
        };

        Ok(project)
    }

    pub fn filename(&self) -> String {
        self.name.to_filename()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let projects_dir = get_projects_dir()?;
        let path = projects_dir.join(self.filename());
        let file = File::create(path)?;

        serde_yaml::to_writer(file, self)?;

        Ok(())
    }

    pub fn list() -> anyhow::Result<Vec<Id>> {
        let projects_dir = get_projects_dir()?;

        let project_names = projects_dir
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

        Ok(project_names)
    }
}

#[derive(Clone, Debug)]
pub struct ProjectAutocomplete {
    project_names: Vec<String>,
    lowercase_names: Vec<String>,
}

impl ProjectAutocomplete {
    pub fn new(project_ids: Vec<Id>) -> Self {
        let project_names: Vec<String> = project_ids.into_iter().map(Into::into).collect();

        let lowercase_names = project_names.iter().map(|s| s.to_lowercase()).collect();

        Self {
            project_names,
            lowercase_names,
        }
    }

    pub fn try_new() -> anyhow::Result<Self> {
        let project_names = Project::list()?;

        Ok(Self::new(project_names))
    }

    fn get_matches(&self, input: &str) -> anyhow::Result<Vec<String>> {
        let lowercase_input = input.to_lowercase();

        let matches = self
            .lowercase_names
            .iter()
            .enumerate()
            // Filter to matching names
            .filter(|(_i, name)| name.starts_with(&lowercase_input))
            // Get normal-case name with same index
            .map(|(i, _name)| self.project_names[i].clone())
            .collect();

        Ok(matches)
    }
}

impl inquire::Autocomplete for ProjectAutocomplete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let matches = self.get_matches(input)?;
        Ok(matches)
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, inquire::CustomUserError> {
        if let Some(suggestion) = highlighted_suggestion {
            return Ok(Replacement::Some(suggestion));
        } else {
            let matches = self.get_matches(input)?;
            // Is there at least one match?
            if let Some((first, rest)) = matches.split_first() {
                // Is there exactly one match?
                if rest.len() == 0 {
                    return Ok(Replacement::Some(first.clone()));
                }
            }
        }

        // Fallback to no completion
        Ok(Replacement::None)
    }
}
