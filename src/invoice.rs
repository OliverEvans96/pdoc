use std::{fs::File, path::Path};

use askama::Template;
use inquire::validator::{StringValidator, Validation};
use serde::{Deserialize, Serialize};
use time::{Date, Duration};

use crate::{
    client::Client,
    date::DateString,
    id::Id,
    latex::{compile_latex, Asset, Latex},
    me::{Me, PaymentMethod},
    price::PriceUSD,
    project::Project,
    storage::{find_client, find_project, get_invoices_dir, read_me},
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LineItem {
    pub description: String,
    pub quantity: u32,
    pub unit_price: PriceUSD,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Invoice {
    pub number: u32,
    pub project_ref: Id,
    pub date: DateString,
    pub due_date: DateString,
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

        let project_name = Project::get_or_create_from_user_input()?;

        let chrono_date = inquire::DateSelect::new("Invoice date:").prompt()?;
        // Convert `chrono::Date` to `time::Date`.
        let invoice_date_string = DateString::try_new(chrono_date.to_string())?;
        let invoice_date: Date = invoice_date_string.clone().try_into()?;

        let days_to_pay = inquire::CustomType::<u16>::new("Days to pay:")
            .with_default(7)
            .prompt()?;

        let due_date = invoice_date + Duration::days(days_to_pay.into());
        let due_date_string = DateString::try_from(due_date)?;

        // TODO create line items
        let invoice = Invoice {
            number: invoice_number,
            project_ref: project_name,
            date: invoice_date_string,
            due_date: due_date_string,
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
#[template(path = "invoice.tex")]
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

#[cfg(test)]
mod test {
    use crate::id::Id;

    use super::Invoice;

    use time::macros::date;

    #[test]
    fn test_serialize_invoice() -> anyhow::Result<()> {
        let invoice = Invoice {
            number: 5,
            project_ref: Id::new("Manhattan".to_owned()),
            days_to_pay: 7,
            date: date!(2023 - 02 - 17).try_into()?,
            items: Vec::new(),
        };

        let expected = r#"number: 5
project_ref: Manhattan
days_to_pay: 7
date: 2023-02-17
items: []
"#;

        let actual = serde_yaml::to_string(&invoice)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_deserialize_invoice() -> anyhow::Result<()> {
        let yaml = r#"number: 5
project_ref: Manhattan
days_to_pay: 7
date: 2023-02-17
items: []
"#;
        let expected = Invoice {
            number: 5,
            project_ref: Id::new("Manhattan".to_owned()),
            days_to_pay: 7,
            date: date!(2023 - 02 - 17).try_into()?,
            items: Vec::new(),
        };

        let actual: Invoice = serde_yaml::from_str(yaml)?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
