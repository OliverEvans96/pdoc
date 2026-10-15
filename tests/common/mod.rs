use std::{path::Path, process::Command, str::FromStr};

use pdoc::{
    address::MailingAddress,
    client::Client,
    config::{Config, StorageConfig},
    contact::ContactInfo,
    date::DateString,
    id::Id,
    invoice::{Invoice, LineItem},
    me::{Me, PaymentMethod},
    price::PriceUSD,
    project::Project,
};
use tempfile::TempDir;

pub fn assert_latexmk_available() {
    let ok = Command::new("latexmk")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(
        ok,
        "latexmk is required for PDF integration tests; install TeX (see README)"
    );
}

pub fn sandbox_config(temp: &TempDir) -> anyhow::Result<Config> {
    let data_dir = temp.path().canonicalize()?;
    Ok(Config {
        me: Me {
            name: "Test User".to_owned(),
            address: MailingAddress {
                addr1: "123 Test Street".to_owned(),
                addr2: None,
                addr3: None,
                city: "Twin Falls".to_owned(),
                state: "Idaho".to_owned(),
                zip: "12345".to_owned(),
            },
            contact: ContactInfo {
                email: "test@example.com".to_owned(),
                phone: "(123) 456-7890".to_owned(),
            },
            payment: vec![PaymentMethod {
                name: "PayPal".to_owned(),
                display_text: None,
                url: None,
            }],
        },
        storage: StorageConfig {
            data_dir: Some(data_dir),
        },
    })
}

pub fn seed_client_and_project(config: &Config) -> anyhow::Result<(Id, Id)> {
    let client_id = Id::new("Test Client #1".to_owned());
    let project_id = Id::new("Test Project #1".to_owned());

    let client = Client {
        name: client_id.clone(),
        address: MailingAddress {
            addr1: "124 Test Avenue".to_owned(),
            addr2: None,
            addr3: None,
            city: "New York".to_owned(),
            state: "New York".to_owned(),
            zip: "54321".to_owned(),
        },
        contact: ContactInfo {
            email: "client@example.com".to_owned(),
            phone: "(321) 654-0987".to_owned(),
        },
    };
    client.save(config)?;

    let project = Project {
        name: project_id.clone(),
        description: "A great project for testing".to_owned(),
        client_ref: client_id.clone(),
        payment_terms: None,
    };
    project.save(config)?;

    Ok((client_id, project_id))
}

pub fn sample_invoice(project_ref: Id) -> anyhow::Result<Invoice> {
    Ok(Invoice {
        number: 17,
        project_ref,
        date: DateString::try_new("2023-01-07".to_owned())?,
        due_date: DateString::try_new("2023-01-21".to_owned())?,
        items: vec![LineItem {
            description: "Test the first thing".to_owned(),
            quantity: 1.0,
            unit_price: PriceUSD::from_str("10.30")?,
        }],
        conditions: None,
    })
}

pub fn assert_valid_pdf(path: &Path) -> anyhow::Result<()> {
    let bytes = std::fs::read(path)?;
    assert!(!bytes.is_empty(), "PDF at {:?} is empty", path);
    assert!(
        bytes.starts_with(b"%PDF"),
        "PDF at {:?} does not start with %PDF magic",
        path
    );
    Ok(())
}
