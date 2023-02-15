use std::{
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail};
use askama::Template;
use hex::ToHex;
use rand::{distributions::Standard, prelude::Distribution, Rng};
use serde::{Deserialize, Serialize};
use time::Date;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct MailingAddress {
    addr1: String,
    addr2: Option<String>,
    city: String,
    state: String,
    zip: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ContactInfo {
    email: String,
    phone: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Client {
    id: Id,
    name: String,
    address: MailingAddress,
    contact: ContactInfo,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct Id([u8; 8]);

impl Distribution<Id> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Id {
        Id(rng.gen())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.0.encode_hex();
        f.write_str(&s)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PriceUSD(f32);

impl Display for PriceUSD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", &self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct LineItem {
    description: String,
    quantity: u32,
    unit_price: PriceUSD,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
enum PaymentMethod {
    Text(String),
    Link { text: String, url: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Me {
    name: String,
    address: MailingAddress,
    contact: ContactInfo,
    payment: Vec<PaymentMethod>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Project {
    id: Id,
    name: String,
    description: String,
    client: Client,
}

#[derive(Clone, Debug, Deserialize, Serialize, Template)]
#[template(path = "invoice.tex", escape = "none")]
struct Invoice {
    id: Id,
    me: Me,
    invoice_no: u32,
    project: Project,
    days_to_pay: u16,
    invoice_date: Date,
    items: Vec<LineItem>,
}

fn get_data_dir() -> anyhow::Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("", "", "invgen")
        .ok_or(anyhow!("Couldn't get data directory"))?;
    let data_dir = project_dirs.data_dir();

    return Ok(data_dir.to_owned());
}

fn get_projects_dir() -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir()?;
    let projects_dir = data_dir.join("projects");
    std::fs::create_dir_all(&projects_dir)?;
    Ok(projects_dir)
}

fn get_clients_dir() -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir()?;
    let clients_dir = data_dir.join("clients");
    std::fs::create_dir_all(&clients_dir)?;
    Ok(clients_dir)
}

/// Reads personal info from yaml in app data dir
fn read_me() -> anyhow::Result<Me> {
    let data_dir = get_data_dir()?;

    let me_path = data_dir.join("me.yaml");
    let me_file = File::open(me_path)?;
    let me_yaml: Me = serde_yaml::from_reader(me_file)?;

    Ok(me_yaml)
}

impl Invoice {
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

fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    let invoice_file = File::open("invoice.yaml")?;
    let invoice: Invoice = serde_yaml::from_reader(invoice_file)?;

    invoice.render_pdf("out.pdf")?;

    println!("Done!");

    Ok(())
}

#[derive(Clone, Debug)]
struct Asset {
    data: Vec<u8>,
    filename: String,
}

fn compile_latex(
    tex: &str,
    pdf_output_path: impl AsRef<Path>,
    assets: &[Asset],
) -> anyhow::Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    let tmp_dir_path = tmp_dir.path();

    let basename = "invoice";
    let tex_filename = format!("{}.tex", basename);
    let pdf_filename = format!("{}.pdf", basename);

    let tex_path = tmp_dir_path.join(tex_filename);
    let pdf_path = tmp_dir_path.join(pdf_filename);

    // Write latex file
    std::fs::write(&tex_path, tex.as_bytes())?;

    // Copy assets to compilation directory
    for asset in assets {
        let filename = tmp_dir.path().join(&asset.filename);
        std::fs::write(filename, &asset.data)?;
    }

    let mut compile_command = std::process::Command::new("pdflatex");

    compile_command.current_dir(&tmp_dir_path).arg(&tex_path);

    let exit_status = compile_command.status()?;

    if !exit_status.success() {
        bail!("Non-success exit status: {:?}", exit_status);
    }

    std::fs::copy(pdf_path, pdf_output_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use crate::Id;

    #[test]
    fn test_rand_id() {
        let mut rng = thread_rng();
        let id: Id = rng.gen();
        let id_str = id.to_string();
        assert_eq!(id_str.len(), 16)
    }
}
