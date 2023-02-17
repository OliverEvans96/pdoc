use std::fs::File;

use clap::{Parser, Subcommand};
use project::Project;

use crate::{client::Client, invoice::Invoice};

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
    /// Get or create client.
    Client,
    /// List all saved clients.
    ListClients,
    /// Generate an invoice.
    Invoice,
    /// Get or create project.
    Project,
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
    let invoice_file = File::open("invoice.yaml")?;
    let invoice: Invoice = serde_yaml::from_reader(invoice_file)?;
    let full_invoice = invoice.collect()?;

    full_invoice.render_pdf("out.pdf")?;

    Ok(())
}

fn list_clients() -> anyhow::Result<()> {
    let client_names = Client::list()?;

    for name in client_names {
        println!("- {}", name)
    }

    Ok(())
}

fn get_or_create_project() -> anyhow::Result<()> {
    Project::create_from_user_input()?;

    Ok(())
}

// TODO nested create (invoice, project, client)
fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    println!("Hello!");

    match opts.command {
        Command::Client => get_or_create_client()?,
        Command::ListClients => list_clients()?,
        Command::Invoice => generate_invoice()?,
        Command::Project => get_or_create_project()?,
    }

    println!("Done!");

    Ok(())
}
