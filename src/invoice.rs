use std::path::Path;

use askama::Template;
use serde::{Deserialize, Serialize};
use time::Date;

use crate::{
    client::Client,
    id::Id,
    latex::{compile_latex, Asset},
    me::{Me, PaymentMethod},
    price::PriceUSD,
    project::Project,
    storage::{find_client, find_project, read_me},
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

impl Invoice {
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
