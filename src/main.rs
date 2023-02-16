use std::fs::File;

use clap::{Parser, Subcommand};

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

#[derive(Subcommand)]
enum Command {
    /// Get or create client
    Client,
}

#[derive(Parser)]
struct Opts {
    #[command(subcommand)]
    command: Command,
}

fn get_or_create_client() -> anyhow::Result<()> {
    let client = Client::create_from_user_input()?;

    println!("got client: {:#?}", client);

    client.save()?;

    Ok(())
}

fn generate_invoice() -> anyhow::Result<()> {
    // let invoice_file = File::open("invoice.yaml")?;
    // let invoice: Invoice = serde_yaml::from_reader(invoice_file)?;
    // let full_invoice = invoice.collect()?;

    // full_invoice.render_pdf("out.pdf")?;

    Ok(())
}

// TODO nested create (invoice, project, client)
fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    println!("Hello!");

    match opts.command {
        Command::Client => get_or_create_client()?,
    }

    println!("Done!");

    Ok(())
}
