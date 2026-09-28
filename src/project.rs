use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::{
    cli::{print_header, YamlValidator},
    client::Client,
    completion::{LocalAutocompleter, PrefixAutocomplete},
    config::Config,
    id::Id,
    payment_terms::PaymentTerms,
    storage::get_projects_dir,
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: Id,
    pub description: String,
    pub client_ref: Id,
    #[serde(default)]
    pub payment_terms: Option<PaymentTerms>,
}

impl Project {
    pub fn edit_yaml(&self) -> anyhow::Result<Self> {
        let yaml = serde_yaml::to_string(&self)?;

        // TODO: Show as markdown code block via termimad
        print_header("Final YAML");
        println!("{}", yaml);

        let yaml_validator = YamlValidator::<Project>::new();

        let edited = inquire::Editor::new("Edit...")
            .with_predefined_text(&yaml)
            .with_validator(yaml_validator)
            .with_file_extension(".yaml")
            .prompt()?;

        let parsed = serde_yaml::from_str(&edited)?;

        Ok(parsed)
    }

    pub fn get_or_create_from_user_input(config: &Config) -> anyhow::Result<Id> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let project_names = Project::list(config)?;
        let autocomplete = LocalAutocompleter::new(ProjectAutocomplete::new(project_names.clone()));

        let project_name: Id = inquire::Text::new("Project Name:")
            .with_autocomplete(autocomplete)
            .with_validator(required_validator)
            .prompt()?
            .into();

        if !project_names.contains(&project_name) {
            let project = Project::create_from_user_input_with_name(project_name.clone(), config)?;
            project.save(config)?;
        };

        Ok(project_name)
    }

    pub fn create_from_user_input_with_name(name: Id, config: &Config) -> anyhow::Result<Self> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();

        let description = inquire::Text::new("Project Description:")
            .with_placeholder("Doing what we can, one day at a time.")
            .with_validator(required_validator.clone())
            .prompt()?;

        let client_name = Client::get_or_create_from_user_input(config)?;

        let payment_terms = PaymentTerms::prompt_from_user_optional()?;

        let mut project = Self {
            name,
            description,
            client_ref: client_name,
            payment_terms,
        };

        project = project.edit_yaml()?;

        Ok(project)
    }

    pub fn filename(&self) -> String {
        self.name.to_filename()
    }

    pub fn save(&self, config: &Config) -> anyhow::Result<()> {
        let projects_dir = get_projects_dir(config)?;
        let path = projects_dir.join(self.filename());
        let file = File::create(path)?;

        serde_yaml::to_writer(file, self)?;

        Ok(())
    }

    pub fn list(config: &Config) -> anyhow::Result<Vec<Id>> {
        let projects_dir = get_projects_dir(config)?;

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

#[cfg(test)]
mod tests {
    use crate::payment_terms::{DayKind, PaymentTerms};

    use super::Project;

    #[test]
    fn project_deserializes_without_payment_terms() -> anyhow::Result<()> {
        let yaml = r#"name: Manhattan
description: A project
client_ref: Acme
"#;
        let project: Project = serde_yaml::from_str(yaml)?;

        assert_eq!(format!("{}", project.name), "Manhattan");
        assert!(project.payment_terms.is_none());

        Ok(())
    }

    #[test]
    fn project_round_trips_payment_terms() -> anyhow::Result<()> {
        let project = Project {
            name: "Manhattan".to_owned().into(),
            description: "A project".to_owned(),
            client_ref: "Acme".to_owned().into(),
            payment_terms: Some(PaymentTerms {
                days: 14,
                day_kind: DayKind::Business,
            }),
        };

        let yaml = serde_yaml::to_string(&project)?;
        let parsed: Project = serde_yaml::from_str(&yaml)?;

        assert_eq!(project, parsed);

        Ok(())
    }
}
