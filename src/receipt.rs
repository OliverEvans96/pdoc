use std::{
    collections::HashSet,
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use askama::Template;
use serde::{Deserialize, Serialize};

use crate::{
    cli::{print_header, YamlValidator},
    client::Client,
    date::DateString,
    invoice::Invoice,
    latex::{compile_latex, Asset, Latex},
    me::Me,
    project::Project,
    storage::{find_client, find_invoice, find_project, get_pdfs_dir, get_receipts_dir, read_me},
};

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
    pub payment_method: String,
}

impl Receipt {
    pub fn list() -> anyhow::Result<Vec<u32>> {
        let receipts_dir = get_receipts_dir().context("getting receipts directory")?;

        let receipt_numbers: Vec<u32> = receipts_dir
            .read_dir()
            .context("listing files in receipts directory")?
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
        let yaml = serde_yaml::to_string(&self).context("serializing receipt yaml")?;

        // TODO: Show as markdown code block via termimad
        print_header("Final YAML");
        println!("{}", yaml);

        let yaml_validator = YamlValidator::<Receipt>::new();

        let edited = inquire::Editor::new("Edit...")
            .with_predefined_text(&yaml)
            .with_validator(yaml_validator)
            .with_file_extension(".yaml")
            .prompt()
            .context("reading edited receipt yaml from user input")?;

        let parsed = serde_yaml::from_str(&edited).context("deserializing edited receipt yaml")?;

        Ok(parsed)
    }

    pub fn create_from_user_input() -> anyhow::Result<Self> {
        let me = Me::load().context("loading personal info")?;

        let invoice_nums = Invoice::list()
            .context("listing invoices")?
            .into_iter()
            .collect::<HashSet<_>>();
        let receipt_nums = Receipt::list()
            .context("listing receipts")?
            .into_iter()
            .collect::<HashSet<_>>();

        let unpaid_invoice_nums = &invoice_nums - &receipt_nums;

        if unpaid_invoice_nums.len() == 0 {
            bail!("All invoices have been paid!");
        }

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

        let invoice_choice = inquire::Select::new("Invoice number:", invoice_options)
            .prompt()
            .context("reading invoice number for receipt from user input")?;

        let invoice_num = invoice_choice.value;

        // TODO: Edit receipt if number already exists

        print_header(&format!("Create receipt {}", invoice_num));

        let chrono_date = inquire::DateSelect::new("Receipt date:")
            .prompt()
            .context("reading receipt date from user input")?;
        // Convert `chrono::Date` to `time::Date`.
        let date_string = DateString::try_new(chrono_date.to_string())
            .context("parsing invoice DateString from user input")?;

        let payment_options = me
            .payment
            .into_iter()
            .map(|method| method.name)
            .map(|text| SelectOption {
                value: text.clone(),
                description: text,
            })
            .collect();

        let payment_method = inquire::Select::new("Payment method:", payment_options)
            .prompt()
            .context("reading payment method from user input")?
            .value;

        let mut receipt = Receipt {
            invoice_num,
            date: date_string,
            payment_method,
        };

        receipt = receipt.edit_yaml().context("editing receipt yaml")?;

        Ok(receipt)
    }

    pub fn filename(&self) -> String {
        format!("{}.yaml", self.invoice_num)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let projects_dir = get_receipts_dir().context("getting receipts directory")?;
        let path = projects_dir.join(self.filename());
        let file = File::create(path).context("creating receipt yaml file")?;

        serde_yaml::to_writer(file, self).context("serializing receipt yaml")?;

        Ok(())
    }

    pub fn collect(self) -> anyhow::Result<FullReceipt> {
        let me = read_me().context("reading personal info")?;
        let invoice = find_invoice(self.invoice_num).context("finding invoice")?;
        let project = find_project(&invoice.project_ref).context("finding project")?;
        let client = find_client(&project.client_ref).context("finding client")?;

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
    pub fn filename(&self) -> String {
        let name_no_whitespace = self.me.name.split_whitespace().collect::<Vec<_>>().join("");

        format!("Receipt_{}_{}.pdf", name_no_whitespace, self.invoice.number)
    }

    pub fn render_pdf(&self, pdf_output_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let rendered_tex = Template::render(self).context("rendering receipt template")?;

        let invoice_class = Asset {
            data: include_bytes!("../assets/CSMinimalInvoice.cls").to_vec(),
            filename: "CSMinimalInvoice.cls".to_owned(),
        };
        let assets = &[invoice_class];
        compile_latex(&rendered_tex, pdf_output_path.as_ref(), assets)
            .context("compiling receipt LaTeX to PDF")?;

        Ok(())
    }

    pub fn save_pdf(&self) -> anyhow::Result<PathBuf> {
        let pdfs_dir = get_pdfs_dir().context("getting receipts directory")?;
        let path = pdfs_dir.join(self.filename());

        self.render_pdf(&path).context("generating receipt PDF")?;

        Ok(path)
    }
}
