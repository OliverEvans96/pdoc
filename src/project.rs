use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::{
    client::Client,
    completion::{LocalAutocompleter, PrefixAutocomplete},
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
    pub fn get_or_create_from_user_input() -> anyhow::Result<Id> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let project_names = Project::list()?;
        let autocomplete = LocalAutocompleter::new(ProjectAutocomplete::new(project_names.clone()));

        let project_name: Id = inquire::Text::new("Project Name:")
            .with_autocomplete(autocomplete)
            .with_validator(required_validator)
            .prompt()?
            .into();

        if !project_names.contains(&project_name) {
            let project = Project::create_from_user_input_with_name(project_name.clone())?;
            project.save()?;
        };

        Ok(project_name)
    }

    pub fn create_from_user_input_with_name(name: Id) -> anyhow::Result<Self> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let description = inquire::Text::new("Project Description:")
            .with_placeholder("Doing what we can, one day at a time.")
            .with_validator(required_validator.clone())
            .prompt()?;

        let client_name = Client::get_or_create_from_user_input()?;

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
}

impl PrefixAutocomplete for ProjectAutocomplete {
    fn get_options(&self) -> &[String] {
        &self.project_names
    }

    fn get_lowercase_options(&self) -> &[String] {
        &self.lowercase_names
    }
}
