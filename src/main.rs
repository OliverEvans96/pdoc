use clap::{Parser, Subcommand};
use cli::print_title;
use project::Project;

use crate::{client::Client, invoice::Invoice, receipt::Receipt};

mod address;
mod cli;
mod client;
mod completion;
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
}

#[derive(Parser)]
struct Opts {
    #[command(subcommand)]
    command: Command,
}

fn get_or_create_client() -> anyhow::Result<()> {
    let client = Client::get_or_create_from_user_input()?;

    println!("got client: {:#?}", client);

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

fn generate_receipt() -> anyhow::Result<()> {
    let receipt = Receipt::create_from_user_input()?;
    receipt.save()?;

    println!("Receipt: {:#?}", receipt);

    let full_receipt = receipt.collect()?;

    full_receipt.render_pdf("out.pdf")?;

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
    let project = Project::get_or_create_from_user_input()?;

    println!("project: {:#?}", project);

    Ok(())
}

// TODO unique name/number validators
// TODO use a trait for loading/saving from id? (but invoice uses number - can it also be an id?)
// TODO render latex to pdf using tectonic or texrender crates
// TODO quiet latex rendering by default
// TODO generate receipts for invoices
// TODO display final yaml and allow user to open in editor
// TODO specify PDF output location from command line?
// TODO save PDFs to data dir?
// TODO re-render PDFs from yaml?
// TODO finalize CLI
// TODO delineate different input sections with underlined headers
// TODO edit me.yaml from CLI
// TODO pdoc TOML config file (e.g. to set data dir)
// TODO generate beancount files?
fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    print_title("pdoc");

    match opts.command {
        Command::Client => get_or_create_client()?,
        Command::ListClients => list_clients()?,
        Command::Invoice => generate_invoice()?,
        Command::Receipt => generate_receipt()?,
        Command::Project => get_or_create_project()?,
    }

    Ok(())
}
