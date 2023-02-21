use std::{collections::HashSet, fmt::Display, fs::File, path::Path};

use askama::Template;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    cli::{print_header, YamlValidator},
    client::Client,
    date::DateString,
    invoice::Invoice,
    latex::{compile_latex, Asset, Latex},
    me::Me,
    project::Project,
    storage::{find_client, find_invoice, find_project, get_receipts_dir, read_me},
};

#[derive(Clone, Copy, Debug, Deserialize, EnumIter, Eq, PartialEq, Serialize)]
pub enum PaymentMethod {
    PersonalCheck,
    Venmo,
    PayPal,
}

impl Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PaymentMethod::PersonalCheck => "Personal Check",
            PaymentMethod::Venmo => "Venmo",
            PaymentMethod::PayPal => "PayPal",
        };

        f.write_str(s)
    }
}

struct SelectOption<T> {
    pub value: T,
    pub description: String,
}

impl<T> Display for SelectOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.description)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub invoice_num: u32,
    pub date: DateString,
    pub payment_method: PaymentMethod,
}

impl Receipt {
    pub fn list() -> anyhow::Result<Vec<u32>> {
        let receipts_dir = get_receipts_dir()?;

        let receipt_numbers: Vec<u32> = receipts_dir
            .read_dir()?
            .filter_map(|entry_res| {
                let entry = entry_res.ok()?;
                let path = entry.path();
                let stem = path.file_stem()?.to_string_lossy();
                let number = u32::from_str_radix(&stem, 10).ok()?;

                Some(number)
            })
            .collect();

        Ok(receipt_numbers)
    }

    pub fn edit_yaml(&self) -> anyhow::Result<Self> {
        let yaml = serde_yaml::to_string(&self)?;

        // TODO: Show as markdown code block via termimad
        print_header("Final YAML");
        println!("{}", yaml);

        let yaml_validator = YamlValidator::<Receipt>::new();

        let edited = inquire::Editor::new("Edit...")
            .with_predefined_text(&yaml)
            .with_validator(yaml_validator)
            .with_file_extension(".yaml")
            .prompt()?;

        let parsed = serde_yaml::from_str(&edited)?;

        Ok(parsed)
    }

    pub fn create_from_user_input() -> anyhow::Result<Self> {
        let invoice_nums = Invoice::list()?.into_iter().collect::<HashSet<_>>();
        let receipt_nums = Receipt::list()?.into_iter().collect::<HashSet<_>>();

        let unpaid_invoice_nums = &invoice_nums - &receipt_nums;

        let mut invoice_options: Vec<SelectOption<u32>> = unpaid_invoice_nums
            .into_iter()
            // // Load invoice from disk
            // .map(Invoice::load)
            // // Silently discard any invoices that
            // // could not be read sucessfully.
            // .filter_map(Result::ok)
            .filter_map(|number| {
                let inv = Invoice::load(number).ok()?;
                let description = format!(
                    "#{} on {} (due {}) for {}",
                    inv.number, inv.date, inv.due_date, inv.project_ref
                );

                let choice = SelectOption {
                    value: number,
                    description,
                };

                Some(choice)
            })
            .collect();

        invoice_options.sort_by_key(|opt| u32::MAX - opt.value);

        let invoice_choice = inquire::Select::new("Invoice number:", invoice_options).prompt()?;

        let invoice_num = invoice_choice.value;

        // TODO: Edit receipt if number already exists

        print_header(&format!("Create receipt {}", invoice_num));

        let chrono_date = inquire::DateSelect::new("Receipt date:").prompt()?;
        // Convert `chrono::Date` to `time::Date`.
        let date_string = DateString::try_new(chrono_date.to_string())?;

        let payment_options = PaymentMethod::iter()
            .map(|method| SelectOption {
                value: method,
                description: method.to_string(),
            })
            .collect();
        let payment_method = inquire::Select::new("Payment method:", payment_options)
            .prompt()?
            .value;

        let mut receipt = Receipt {
            invoice_num,
            date: date_string,
            payment_method,
        };

        receipt = receipt.edit_yaml()?;

        Ok(receipt)
    }

    pub fn filename(&self) -> String {
        format!("{}.yaml", self.invoice_num)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let projects_dir = get_receipts_dir()?;
        let path = projects_dir.join(self.filename());
        let file = File::create(path)?;

        serde_yaml::to_writer(file, self)?;

        Ok(())
    }

    pub fn collect(self) -> anyhow::Result<FullReceipt> {
        let me = read_me()?;
        let invoice = find_invoice(self.invoice_num)?;
        let project = find_project(&invoice.project_ref)?;
        let client = find_client(&project.client_ref)?;

        let full_receipt = FullReceipt {
            me,
            receipt: self,
            invoice,
            project,
            client,
        };
        Ok(full_receipt)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Template)]
#[serde(deny_unknown_fields)]
#[template(path = "receipt.tex")]
pub struct FullReceipt {
    pub me: Me,
    pub receipt: Receipt,
    pub invoice: Invoice,
    pub project: Project,
    pub client: Client,
}

impl FullReceipt {
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
