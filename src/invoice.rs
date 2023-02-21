use std::{fs::File, path::Path};

use askama::Template;
use serde::{Deserialize, Serialize};
use time::{Date, Duration};

use crate::{
    cli::{print_header, NumberValidator, YamlValidator},
    client::Client,
    completion::PrefixAutocomplete,
    date::DateString,
    id::Id,
    latex::{compile_latex, Asset, Latex},
    me::{Me, PaymentMethod},
    price::PriceUSD,
    project::Project,
    storage::{find_client, find_project, get_invoices_dir, read_me},
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineItem {
    pub description: String,
    pub quantity: f32,
    pub unit_price: PriceUSD,
}

impl LineItem {
    pub fn create_from_user_input() -> anyhow::Result<Option<Self>> {
        let maybe_description = inquire::Text::new("Line item:")
            .prompt_skippable()?
            // Convert Some("") to None
            .filter(|line| !line.is_empty());

        if let Some(description) = maybe_description {
            let quantity = inquire::CustomType::<f32>::new("Quantity:").prompt()?;
            let unit_price = inquire::CustomType::<PriceUSD>::new("Unit Price:").prompt()?;

            let line_item = LineItem {
                description,
                quantity,
                unit_price,
            };

            Ok(Some(line_item))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Invoice {
    pub number: u32,
    pub project_ref: Id,
    pub date: DateString,
    pub due_date: DateString,
    pub items: Vec<LineItem>,
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

    pub fn load_from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref())?;
        let invoice: Invoice = serde_yaml::from_reader(file)?;

        Ok(invoice)
    }

    pub fn load(number: u32) -> anyhow::Result<Self> {
        let invoices_dir = get_invoices_dir()?;
        let filename = format!("{}.yaml", number);
        let path = invoices_dir.join(filename);
        let invoice = Invoice::load_from_path(path)?;

        Ok(invoice)
    }

    pub fn get_next_number() -> anyhow::Result<u32> {
        let existing_numbers = Self::list()?;
        let max = existing_numbers.iter().fold(0, |acc, &el| acc.max(el));
        let next = max + 1;

        Ok(next)
    }

    pub fn edit_yaml(&self) -> anyhow::Result<Self> {
        let yaml = serde_yaml::to_string(&self)?;

        // TODO: Show as markdown code block via termimad
        print_header("Final YAML");
        println!("{}", yaml);

        let yaml_validator = YamlValidator::<Invoice>::new();

        let edited = inquire::Editor::new("Edit...")
            .with_predefined_text(&yaml)
            .with_validator(yaml_validator)
            .with_file_extension(".yaml")
            .prompt()?;

        let parsed = serde_yaml::from_str(&edited)?;

        Ok(parsed)
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

        // TODO: Edit invoice if number already exists

        print_header(&format!("Create invoice {}", invoice_number));

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

        let mut items = Vec::new();
        while let Some(item) = LineItem::create_from_user_input()? {
            items.push(item);
        }

        let mut invoice = Invoice {
            number: invoice_number,
            project_ref: project_name,
            date: invoice_date_string,
            due_date: due_date_string,
            items,
        };

        invoice = invoice.edit_yaml()?;

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
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Debug)]
pub struct ClientAutocomplete {
    client_names: Vec<String>,
    lowercase_names: Vec<String>,
}

impl ClientAutocomplete {
    pub fn new(client_ids: Vec<Id>) -> Self {
        let client_names: Vec<String> = client_ids.into_iter().map(Into::into).collect();

        let lowercase_names = client_names.iter().map(|s| s.to_lowercase()).collect();

        Self {
            client_names,
            lowercase_names,
        }
    }
}

impl PrefixAutocomplete for ClientAutocomplete {
    fn get_options(&self) -> &[String] {
        &self.client_names
    }

    fn get_lowercase_options(&self) -> &[String] {
        &self.lowercase_names
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
            date: date!(2023 - 02 - 17).try_into()?,
            due_date: date!(2023 - 02 - 24).try_into()?,
            items: Vec::new(),
        };

        let expected = r#"number: 5
project_ref: Manhattan
date: 2023-02-17
due_date: 2023-02-24
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
date: 2023-02-17
due_date: 2023-02-24
items: []
"#;
        let expected = Invoice {
            number: 5,
            project_ref: Id::new("Manhattan".to_owned()),
            date: date!(2023 - 02 - 17).try_into()?,
            due_date: date!(2023 - 02 - 24).try_into()?,
            items: Vec::new(),
        };

        let actual: Invoice = serde_yaml::from_str(yaml)?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
