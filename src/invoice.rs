use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use askama::Template;
use beancount_core::{Account, AccountType, Amount, Posting, Transaction};
use beancount_render::{BasicRenderer, Renderer};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::{Date, Duration};

use crate::{
    cli::{print_header, NumberValidator, YamlValidator},
    client::Client,
    completion::PrefixAutocomplete,
    config::Config,
    date::DateString,
    id::Id,
    latex::{compile_latex, Asset, Latex},
    me::Me,
    price::PriceUSD,
    project::Project,
    storage::{find_client, find_project, get_beancount_dir, get_invoices_dir, get_pdfs_dir},
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
    pub fn list(config: &Config) -> anyhow::Result<Vec<u32>> {
        let invoices_dir = get_invoices_dir(config).context("getting invoices directory")?;

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

    pub fn load(number: u32, config: &Config) -> anyhow::Result<Self> {
        let invoices_dir = get_invoices_dir(config).context("getting invoices directory")?;
        let filename = format!("{}.yaml", number);
        let path = invoices_dir.join(filename);
        let invoice = Invoice::load_from_path(path).context("loading invoice from file")?;

        Ok(invoice)
    }

    pub fn get_next_number(config: &Config) -> anyhow::Result<u32> {
        let existing_numbers = Self::list(config).context("listing invoices")?;
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

    pub fn create_from_user_input(config: &Config) -> anyhow::Result<Self> {
        let required_validator = inquire::validator::ValueRequiredValidator::default();
        let number_validator = NumberValidator::new();

        let next_number = Self::get_next_number(config).context("getting next invoice number")?;

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

        let project_name = Project::get_or_create_from_user_input(config)
            .context("getting or creating project")?;

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

    pub fn save(&self, config: &Config) -> anyhow::Result<()> {
        let projects_dir = get_invoices_dir(config).context("getting invoices directory")?;
        let path = projects_dir.join(self.filename());
        let file = File::create(path).context("opening invoice output file")?;

        serde_yaml::to_writer(file, self).context("serializing invoice yaml")?;

        Ok(())
    }

    pub fn collect(self, config: &Config) -> anyhow::Result<FullInvoice> {
        let project = find_project(&self.project_ref, config).context("finding project")?;
        let client = find_client(&project.client_ref, config).context("finding client")?;

        let full_invoice = FullInvoice {
            me: config.me.clone(),
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

    fn render_pdf(&self, pdf_output_path: impl AsRef<Path>, show_tex: bool) -> anyhow::Result<()> {
        let rendered_tex = Template::render(self).context("rendering invoice template")?;

        if show_tex {
            println!("Final LaTeX:\n\n{}", &rendered_tex);
        }

        let invoice_class = Asset {
            data: include_bytes!("../assets/CSMinimalInvoice.cls").to_vec(),
            filename: "CSMinimalInvoice.cls".to_owned(),
        };
        let assets = &[invoice_class];
        compile_latex(&rendered_tex, pdf_output_path.as_ref(), assets)
            .context("compiling invoice LaTeX to PDF")?;

        Ok(())
    }

    pub fn save_pdf(&self, config: &Config, show_tex: bool) -> anyhow::Result<PathBuf> {
        let pdfs_dir = get_pdfs_dir(config).context("getting PDF directory")?;
        let path = pdfs_dir.join(self.filename());

        self.render_pdf(&path, show_tex)
            .context("generating invoice PDF")?;

        Ok(path)
    }

    fn write_beancount_to<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        let total_cost: f32 = self
            .invoice
            .items
            .iter()
            .map(|item| item.quantity * item.unit_price.as_f32())
            .sum();
        // TODO: proper decimal math
        let total_cost_str = format!("{:.2}", total_cost);
        let total_cost_decimal = Decimal::from_str_exact(&total_cost_str)?;

        let date = self.invoice.date.to_beancount();
        let account_name: String = self
            .client
            .name
            .to_string()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        let narration = format!("Invoice #{} - {}", self.invoice.number, self.project.name);
        let src_account = Account::builder()
            .ty(AccountType::Income)
            .parts([account_name.into()].to_vec())
            .build();
        let dst_account = Account::builder()
            .ty(AccountType::Assets)
            .parts(["AccountsReceivable".into()].to_vec())
            .build();
        let amount = Amount::builder()
            .num(total_cost_decimal)
            .currency("USD".into())
            .build();
        let src_posting = Posting::builder()
            .account(src_account)
            .units(amount.clone().into())
            .build();
        let dst_posting = Posting::builder()
            .account(dst_account)
            .units(amount.into())
            .build();
        let txn = Transaction::builder()
            .date(date)
            .postings([src_posting, dst_posting].to_vec())
            .narration(narration.into())
            .build();

        let renderer = BasicRenderer::new();

        renderer.render(&txn, writer)?;

        Ok(())
    }

    pub fn write_beancount_to_string(&self) -> anyhow::Result<String> {
        let mut buf = Vec::<u8>::new();
        self.write_beancount_to(&mut buf)?;
        let string = String::from_utf8(buf)?;

        Ok(string)
    }

    pub fn save_beancount(&self, config: &Config) -> anyhow::Result<PathBuf> {
        let beancount_dir = get_beancount_dir(config).context("getting beancount directory")?;
        let filename = format!("Invoice_{}.beancount", self.invoice.number);
        let out_path = beancount_dir.join(&filename);
        let mut out_file = File::create(&out_path)?;

        self.write_beancount_to(&mut out_file)?;

        Ok(out_path)
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
    use std::{borrow::Cow, str::FromStr};

    use crate::{
        address::MailingAddress,
        client::Client,
        contact::ContactInfo,
        date::DateString,
        id::Id,
        me::{Me, PaymentMethod},
        price::PriceUSD,
        project::Project,
    };

    use super::{FullInvoice, Invoice, LineItem};

    use beancount_core::{Account, AccountType, Amount, Directive, Ledger, Posting, Transaction};
    use rust_decimal::Decimal;
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

    fn create_full_test_invoice() -> FullInvoice {
        FullInvoice {
            me: Me {
                name: "Test User".to_owned(),
                address: MailingAddress {
                    addr1: "123 Test Street".to_owned(),
                    addr2: None,
                    city: "Twin Falls".to_owned(),
                    state: "Idaho".to_owned(),
                    zip: "12345".to_owned(),
                },
                contact: ContactInfo {
                    email: "test@example.com".to_owned(),
                    phone: "(123) 456-7890".to_owned(),
                },
                payment: [PaymentMethod {
                    name: "PayPal".to_owned(),
                    display_text: None,
                    url: None,
                }]
                .to_vec(),
            },
            invoice: Invoice {
                number: 17,
                project_ref: "Test Project #1".to_owned().into(),
                date: DateString::try_new("2023-01-07".to_owned()).unwrap(),
                due_date: DateString::try_new("2023-01-21".to_owned()).unwrap(),
                items: [
                    LineItem {
                        description: "Test the first thing".to_owned(),
                        quantity: 1.0,
                        unit_price: PriceUSD::from_str("10.3").unwrap(),
                    },
                    LineItem {
                        description: "Test the second thing".to_owned(),
                        quantity: 2.0,
                        unit_price: PriceUSD::from_str("9.6").unwrap(),
                    },
                ]
                .to_vec(),
            },
            project: Project {
                name: "Test Project #1".to_owned().into(),
                description: "A great project for testing".to_owned(),
                client_ref: "Test Client #1".to_owned().into(),
            },
            client: Client {
                name: "Test Client #1".to_owned().into(),
                address: MailingAddress {
                    addr1: "124 Test Avenue".to_owned(),
                    addr2: None,
                    city: "New York".to_owned(),
                    state: "New York".to_owned(),
                    zip: "54321".to_owned(),
                },
                contact: ContactInfo {
                    email: "client@example.com".to_owned(),
                    phone: "(321) 654-0987".to_owned(),
                },
            },
        }
    }

    #[test]
    fn test_write_beancount() -> anyhow::Result<()> {
        let full_invoice = create_full_test_invoice();

        let beancount_string = full_invoice.write_beancount_to_string()?;

        let ledger = beancount_parser::parse(&beancount_string)?;

        let date = beancount_core::Date::from_str_unchecked("2023-01-07");
        let src_account = Account::builder()
            .ty(AccountType::Income)
            .parts([Cow::Borrowed("TestClient1")].to_vec())
            .build();
        let dst_account = Account::builder()
            .ty(AccountType::Assets)
            .parts([Cow::Borrowed("AccountsReceivable")].to_vec())
            .build();
        let amount = Amount::builder()
            .num(Decimal::from_str_exact("29.50").unwrap())
            .currency("USD".into())
            .build();
        let src_posting = Posting::builder()
            .account(src_account)
            .units(amount.clone().into())
            .build();
        let dst_posting = Posting::builder()
            .account(dst_account)
            .units(amount.into())
            .build();
        let txn = Transaction::builder()
            .date(date)
            .postings([src_posting, dst_posting].to_vec())
            .narration("Invoice #17 - Test Project #1".into())
            .source(Some(
                r#"2023-01-07 * "Invoice #17 - Test Project #1"
	Income:TestClient1	29.50 USD
	Assets:AccountsReceivable	29.50 USD
"#,
            ))
            .build();

        let directive = Directive::Transaction(txn);
        let expected_ledger = Ledger::builder().directives([directive].to_vec()).build();

        println!("Expected: {:#?}", expected_ledger);
        println!("Actual: {:#?}", ledger);
        println!();

        assert_eq!(expected_ledger, ledger);

        Ok(())
    }
}
