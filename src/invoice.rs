use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Context;
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
    storage::{find_client, find_project, get_invoices_dir, get_pdfs_dir, read_me},
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
            .prompt_skippable()
            .context("reading line item from user input")?
            // Convert Some("") to None
            .filter(|line| !line.is_empty());

        if let Some(description) = maybe_description {
            let quantity = inquire::CustomType::<f32>::new("Quantity:")
                .prompt()
                .context("reading quantity from user input")?;
            let unit_price = inquire::CustomType::<PriceUSD>::new("Unit Price:")
                .prompt()
                .context("reading unit price from user input")?;

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
        let invoices_dir = get_invoices_dir().context("getting invoices directory")?;

        let invoice_numbers: Vec<u32> = invoices_dir
            .read_dir()
            .context("listing invoice files")?
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
        let file = File::open(path.as_ref()).context("reading invoice file")?;
        let invoice: Invoice = serde_yaml::from_reader(file).context("parsing invoice yaml")?;

        Ok(invoice)
    }

    pub fn load(number: u32) -> anyhow::Result<Self> {
        let invoices_dir = get_invoices_dir().context("getting invoices directory")?;
        let filename = format!("{}.yaml", number);
        let path = invoices_dir.join(filename);
        let invoice = Invoice::load_from_path(path).context("loading invoice from file")?;

        Ok(invoice)
    }

    pub fn get_next_number() -> anyhow::Result<u32> {
        let existing_numbers = Self::list().context("listing invoices")?;
        let max = existing_numbers.iter().fold(0, |acc, &el| acc.max(el));
        let next = max + 1;

        Ok(next)
    }

    pub fn edit_yaml(&self) -> anyhow::Result<Self> {
        let yaml = serde_yaml::to_string(&self).context("serializing invoice")?;

        // TODO: Show as markdown code block via termimad
        print_header("Final YAML");
        println!("{}", yaml);

        let yaml_validator = YamlValidator::<Invoice>::new();

        let edited = inquire::Editor::new("Edit...")
            .with_predefined_text(&yaml)
            .with_validator(yaml_validator)
            .with_file_extension(".yaml")
            .prompt()?;

        let parsed = serde_yaml::from_str(&edited).context("parsing edited invoice yaml")?;

        Ok(parsed)
    }

    pub fn create_from_user_input() -> anyhow::Result<Self> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();
        let number_validator = NumberValidator::new();

        let next_number = Self::get_next_number().context("getting next invoice number")?;

        let invoice_number: u32 = inquire::Text::new("Invoice number:")
            .with_initial_value(&next_number.to_string())
            .with_validator(required_validator.clone())
            .with_validator(number_validator)
            .prompt()
            .context("reading invoice number from user input")?
            .parse()
            .context("parsing invoice number")?;

        // TODO: Edit invoice if number already exists

        print_header(&format!("Create invoice {}", invoice_number));

        let project_name =
            Project::get_or_create_from_user_input().context("getting or creating project")?;

        let chrono_date = inquire::DateSelect::new("Invoice date:")
            .prompt()
            .context("reading invoice date from user input")?;
        // Convert `chrono::Date` to `time::Date`.
        let invoice_date_string = DateString::try_new(chrono_date.to_string())
            .context("parsing invoice DateString from user input")?;
        let invoice_date: Date = invoice_date_string
            .clone()
            .try_into()
            .context("parsing invoice Date from user input")?;

        let days_to_pay = inquire::CustomType::<u16>::new("Days to pay:")
            .with_default(7)
            .prompt()
            .context("reading days-to-pay from user input")?;

        let due_date = invoice_date + Duration::days(days_to_pay.into());
        let due_date_string =
            DateString::try_from(due_date).context("converting due date to DateString")?;

        let mut items = Vec::new();

        while let Some(item) =
            LineItem::create_from_user_input().context("creating line item from user input")?
        {
            items.push(item);
        }

        let mut invoice = Invoice {
            number: invoice_number,
            project_ref: project_name,
            date: invoice_date_string,
            due_date: due_date_string,
            items,
        };

        invoice = invoice.edit_yaml().context("editing invoice yaml")?;

        Ok(invoice)
    }

    pub fn filename(&self) -> String {
        format!("{}.yaml", self.number)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let projects_dir = get_invoices_dir().context("getting invoices directory")?;
        let path = projects_dir.join(self.filename());
        let file = File::create(path).context("opening invoice output file")?;

        serde_yaml::to_writer(file, self).context("serializing invoice yaml")?;

        Ok(())
    }

    pub fn collect(self) -> anyhow::Result<FullInvoice> {
        let me = read_me().context("reading personal info")?;
        let project = find_project(&self.project_ref).context("finding project")?;
        let client = find_client(&project.client_ref).context("finding client")?;

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
    pub fn filename(&self) -> String {
        let name_no_whitespace = self.me.name.split_whitespace().collect::<Vec<_>>().join("");

        format!("Invoice_{}_{}.pdf", name_no_whitespace, self.invoice.number)
    }

    fn render_pdf(&self, pdf_output_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let rendered_tex = Template::render(self).context("rendering invoice template")?;

        let invoice_class = Asset {
            data: include_bytes!("../assets/CSMinimalInvoice.cls").to_vec(),
            filename: "CSMinimalInvoice.cls".to_owned(),
        };
        let assets = &[invoice_class];
        compile_latex(&rendered_tex, pdf_output_path.as_ref(), assets)
            .context("compiling invoice LaTeX to PDF")?;

        Ok(())
    }

    pub fn save_pdf(&self) -> anyhow::Result<PathBuf> {
        let pdfs_dir = get_pdfs_dir().context("getting PDF directory")?;
        let path = pdfs_dir.join(self.filename());

        self.render_pdf(&path).context("generating invoice PDF")?;

        Ok(path)
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
