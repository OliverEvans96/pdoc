use std::fs::File;

use crate::{client::Client, contact::ContactInfo, invoice::Invoice};

mod address;
mod client;
mod contact;
mod id;
mod invoice;
mod latex;
mod me;
mod price;
mod project;
mod storage;

// TODO integrate inquire (questions cli)
// TODO nested create (invoice, project, client)
fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    let client = Client::create_from_user_input();

    println!("got client: {:#?}", client);

    // let invoice_file = File::open("invoice.yaml")?;
    // let invoice: Invoice = serde_yaml::from_reader(invoice_file)?;
    // let full_invoice = invoice.collect()?;

    // full_invoice.render_pdf("out.pdf")?;

    println!("Done!");

    Ok(())
}
