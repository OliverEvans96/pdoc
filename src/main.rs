use anyhow::Context;
use clap::{Parser, Subcommand};
use cli::print_title;
use config::Config;
use project::Project;

use crate::{client::Client, invoice::Invoice, receipt::Receipt};

mod address;
mod cli;
mod client;
mod completion;
mod config;
mod contact;
mod date;
mod filters;
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
    Invoice {
        /// Print latex source before rendering
        #[arg(long)]
        show_tex: bool,
    },
    /// Generate a receipt.
    Receipt {
        /// Print latex source before rendering
        #[arg(long)]
        show_tex: bool,
    },
    /// Get or create project.
    Project,
    // Edit personal info.
    // Me,
}

#[derive(Parser)]
struct Opts {
    #[command(subcommand)]
    command: Command,
}

fn get_or_create_client(config: &Config) -> anyhow::Result<()> {
    let client = Client::get_or_create_from_user_input(config)
        .context("getting or creating client from user input")?;

    println!("got client: {:#?}", client);

    Ok(())
}

fn generate_invoice(config: &Config, show_tex: bool) -> anyhow::Result<()> {
    let invoice =
        Invoice::create_from_user_input(config).context("creating invoice from user input")?;
    invoice.save(config).context("saving invoice yaml")?;

    let full_invoice = invoice
        .collect(config)
        .context("collecting all invoice information")?;

    println!("\nGenerating PDF...");
    let pdf_path = full_invoice
        .save_pdf(config, show_tex)
        .context("saving invoice PDF")?;
    println!("Invoice PDF saved to {:?}", pdf_path);

    let beancount_path = full_invoice
        .save_beancount(config)
        .context("saving invoice beancount file")?;
    println!("Invoice beancount file saved to {:?}", beancount_path);

    Ok(())
}

fn generate_receipt(config: &Config, show_tex: bool) -> anyhow::Result<()> {
    let receipt =
        Receipt::create_from_user_input(config).context("creating receipt from user input")?;
    receipt.save(config).context("saving receipt")?;

    let full_receipt = receipt
        .collect(config)
        .context("collecting all receipt information")?;

    println!("\nGenerating PDF...");
    let path = full_receipt
        .save_pdf(config, show_tex)
        .context("saving receipt PDF")?;
    println!("Receipt PDF saved to {:?}", path);

    Ok(())
}

// TODO: re-enable editing personal info from CLI
// fn edit_personal_info(config: &Config) -> anyhow::Result<()> {
//     print_header("Edit personal info");

//     if let Ok(me) = Me::load(config) {
//         let edited_me = config
//             .me
//             .edit_yaml()
//             .context("editing personal info yaml")?;
//         edited_me
//             .save(config)
//             .context("saving edited personal info yaml")?;
//     } else {
//         let me =
//             Me::create_from_user_input().context("creating personal info data from user input")?;
//         me.save(config).context("saving personal info yaml")?;
//     }

//     println!("\nPersonal info saved!");

//     Ok(())
// }

fn list_clients(config: &Config) -> anyhow::Result<()> {
    let client_names = Client::list(config).context("listing clients")?;

    for name in client_names {
        println!("- {}", name)
    }

    Ok(())
}

fn get_or_create_project(config: &Config) -> anyhow::Result<()> {
    let project = Project::get_or_create_from_user_input(config)
        .context("getting or creating project from user input")?;

    println!("project: {:#?}", project);

    Ok(())
}

// TODO re-render PDFs from yaml?
// TODO finalize CLI
// TODO combine `me.yaml` and `config.toml`?
// TODO beancount config in config.toml
//      - enable/disable
//      - AccountsReceivable name
//      - income account name format
//      - narration format?
//      - payee? tags?
// TODO edit me.yaml at startup if invalid
// TODO switch from `time` crate to `chrono`?
// TODO support non-USD currencies
// TODO more `inquire` help texts (especially indicate which prompts are skippable)
// TODO beancount decimal math
// TODO beancount for receipts
// TODO master beancount file that imports all others?
fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    print_title("pdoc");

    // Me::create_if_necessary()?;

    let config = Config::load()?;

    match opts.command {
        Command::Client => get_or_create_client(&config)?,
        Command::ListClients => list_clients(&config)?,
        Command::Invoice { show_tex } => generate_invoice(&config, show_tex)?,
        Command::Receipt { show_tex } => generate_receipt(&config, show_tex)?,
        Command::Project => get_or_create_project(&config)?,
        // Command::Me => edit_personal_info(&config)?,
    }

    Ok(())
}
