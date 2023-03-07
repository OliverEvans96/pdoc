use anyhow::Context;
use clap::{Parser, Subcommand};
use cli::print_title;
use project::Project;

use crate::{cli::print_header, client::Client, invoice::Invoice, me::Me, receipt::Receipt};

mod address;
mod cli;
mod client;
mod completion;
mod config;
mod contact;
mod date;
mod id;
mod invoice;
mod latex;
mod me;
mod price;
mod project;
mod receipt;
mod storage;

#[derive(Subcommand)]
enum Command {
    /// Get or create client.
    Client,
    /// List all saved clients.
    ListClients,
    /// Generate an invoice.
    Invoice,
    /// Generate a receipt.
    Receipt,
    /// Get or create project.
    Project,
    /// Edit personal info.
    Me,
}

#[derive(Parser)]
struct Opts {
    #[command(subcommand)]
    command: Command,
}

fn get_or_create_client() -> anyhow::Result<()> {
    let client = Client::get_or_create_from_user_input()
        .context("getting or creating client from user input")?;

    println!("got client: {:#?}", client);

    Ok(())
}

fn generate_invoice() -> anyhow::Result<()> {
    let invoice = Invoice::create_from_user_input().context("creating invoice from user input")?;
    invoice.save().context("saving invoice yaml")?;

    let full_invoice = invoice
        .collect()
        .context("collecting all invoice information")?;

    let path = full_invoice.save_pdf().context("saving invoice PDF")?;
    println!("\nInvoice PDF saved to {:?}", path);

    Ok(())
}

fn generate_receipt() -> anyhow::Result<()> {
    let receipt = Receipt::create_from_user_input().context("creating receipt from user input")?;
    receipt.save().context("saving receipt")?;

    let full_receipt = receipt
        .collect()
        .context("collecting all receipt information")?;

    let path = full_receipt.save_pdf().context("saving receipt PDF")?;
    println!("\nReceipt PDF saved to {:?}", path);

    Ok(())
}

fn edit_personal_info() -> anyhow::Result<()> {
    print_header("Edit personal info");

    if let Ok(me) = Me::load() {
        let edited_me = me.edit_yaml().context("editing personal info yaml")?;
        edited_me
            .save()
            .context("saving edited personal info yaml")?;
    } else {
        let me =
            Me::create_from_user_input().context("creating personal info data from user input")?;
        me.save().context("saving personal info yaml")?;
    }

    println!("\nPersonal info saved!");

    Ok(())
}

fn list_clients() -> anyhow::Result<()> {
    let client_names = Client::list().context("listing clients")?;

    for name in client_names {
        println!("- {}", name)
    }

    Ok(())
}

fn get_or_create_project() -> anyhow::Result<()> {
    let project = Project::get_or_create_from_user_input()
        .context("getting or creating project from user input")?;

    println!("project: {:#?}", project);

    Ok(())
}

// TODO unique name/number validators
// TODO use a trait for loading/saving from id? (but invoice uses number - can it also be an id?)
// TODO render latex to pdf using tectonic or texrender crates
// TODO quiet latex rendering by default
// TODO specify PDF output location from command line?
// TODO re-render PDFs from yaml?
// TODO finalize CLI
// TODO generate beancount files?
fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    print_title("pdoc");

    Me::create_if_necessary()?;

    match opts.command {
        Command::Client => get_or_create_client()?,
        Command::ListClients => list_clients()?,
        Command::Invoice => generate_invoice()?,
        Command::Receipt => generate_receipt()?,
        Command::Project => get_or_create_project()?,
        Command::Me => edit_personal_info()?,
    }

    Ok(())
}
