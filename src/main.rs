use std::fs::File;

use clap::{Parser, Subcommand};
use project::Project;
use time::Date;

use crate::{client::Client, invoice::Invoice};

mod address;
mod client;
mod contact;
mod date;
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
    let invoice = Invoice::create_from_user_input()?;
    invoice.save()?;

    println!("Invoice: {:#?}", invoice);

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
    let project = Project::create_from_user_input()?;
    project.save()?;

    println!("project: {:#?}", project);

    Ok(())
}

// TODO nested create (invoice, project, client)
// TODO unique name/number validators
// TODO partial completions (until ambiguity)
// TODO refactor autocomplete & save into `create_from_user_input`
// TODO use a trait for loading/saving from id? (but invoice uses number - can it also be an id?)
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
