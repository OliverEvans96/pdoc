use std::{fs::File, path::Path};

use askama::Template;
use inquire::validator::{StringValidator, Validation};
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};

use crate::{
    client::Client,
    id::Id,
    latex::{compile_latex, Asset},
    me::{Me, PaymentMethod},
    price::PriceUSD,
    project::{Project, ProjectAutocomplete},
    storage::{find_client, find_project, get_invoices_dir, read_me},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LineItem {
    pub description: String,
    pub quantity: u32,
    pub unit_price: PriceUSD,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Invoice {
    pub number: u32,
    pub project_ref: Id,
    pub days_to_pay: u16,
    pub date: Date,
    pub items: Vec<LineItem>,
}

#[derive(Clone, Copy, Debug)]
struct NumberValidator;

impl NumberValidator {
    pub fn new() -> Self {
        Self {}
    }
}

impl StringValidator for NumberValidator {
    fn validate(&self, input: &str) -> Result<Validation, inquire::CustomUserError> {
        let is_valid = input.chars().all(|c| c.is_numeric());
        let validation = if is_valid {
            Validation::Valid
        } else {
            let msg = inquire::validator::ErrorMessage::Custom("number required.".to_owned());
            Validation::Invalid(msg)
        };

        Ok(validation)
    }
}

impl Invoice {
    pub fn list() -> anyhow::Result<Vec<u32>> {
        let invoices_dir = get_invoices_dir()?;

        let invoice_numbers: Vec<u32> = invoices_dir
            .read_dir()?
            .filter_map(|entry_res| {
                let entry = entry_res.ok()?;
                let path = entry.path();
                let stem = path.file_stem()?.to_string_lossy();
                let number = u32::from_str_radix(&stem, 10).ok()?;

                Some(number)
            })
            .collect();

        Ok(invoice_numbers)
    }

    pub fn get_next_number() -> anyhow::Result<u32> {
        let existing_numbers = Self::list()?;
        let max = existing_numbers.iter().fold(0, |acc, &el| acc.max(el));
        let next = max + 1;

        Ok(next)
    }

    pub fn create_from_user_input() -> anyhow::Result<Self> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();
        let number_validator = NumberValidator::new();

        let next_number = Self::get_next_number()?;

        let invoice_number: u32 = inquire::Text::new("Invoice number:")
            .with_initial_value(&next_number.to_string())
            .with_validator(required_validator.clone())
            .with_validator(number_validator)
            .prompt()?
            .parse()?;

        let project_names = Project::list()?;
        let autocomplete = ProjectAutocomplete::new(project_names.clone());

        let project_name: Id = inquire::Text::new("Project Name:")
            .with_autocomplete(autocomplete)
            .with_validator(required_validator)
            .prompt()?
            .into();

        if !project_names.contains(&project_name) {
            let project = Project::create_from_user_input_with_name(project_name.clone())?;
            project.save()?;
        };

        // let days_to_pay = inquire::Text::new("Invoice number:")
        //     .with_initial_value("7")
        //     .with_validator(required_validator)
        //     .with_validator(number_validator)
        //     .prompt()?
        //     .into();

        // TODO
        let invoice = Invoice {
            number: invoice_number,
            project_ref: project_name,
            days_to_pay: 7,
            date: OffsetDateTime::now_local()?.date(),
            items: Vec::new(),
        };

        Ok(invoice)
    }

    pub fn filename(&self) -> String {
        format!("{}.yaml", self.number)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let projects_dir = get_invoices_dir()?;
        let path = projects_dir.join(self.filename());
        let file = File::create(path)?;

        serde_yaml::to_writer(file, self)?;

        Ok(())
    }

    pub fn collect(self) -> anyhow::Result<FullInvoice> {
        let me = read_me()?;
        let project = find_project(&self.project_ref)?;
        let client = find_client(&project.client_ref)?;

        let full_invoice = FullInvoice {
            me,
            invoice: self,
            project,
            client,
        };
        Ok(full_invoice)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Template)]
#[template(path = "invoice.tex", escape = "none")]
pub struct FullInvoice {
    pub me: Me,
    pub invoice: Invoice,
    pub project: Project,
    pub client: Client,
}

impl FullInvoice {
    pub fn render_pdf(&self, pdf_output_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let rendered_tex = Template::render(self)?;

        let invoice_class = Asset {
            data: include_bytes!("../assets/CSMinimalInvoice.cls").to_vec(),
            filename: "CSMinimalInvoice.cls".to_owned(),
        };
        let assets = &[invoice_class];
        compile_latex(&rendered_tex, pdf_output_path.as_ref(), assets)?;

        Ok(())
    }
}
