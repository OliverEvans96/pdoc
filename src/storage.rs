use std::{fs::File, path::PathBuf};

use anyhow::{anyhow, bail, Context};

use crate::{client::Client, config::Config, id::Id, invoice::Invoice, project::Project};

fn get_config_dir() -> anyhow::Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("", "", "pdoc")
        .ok_or(anyhow!("Couldn't get data directory"))?;
    let data_dir = project_dirs.config_dir();

    return Ok(data_dir.to_owned());
}

pub fn get_config_file_path() -> anyhow::Result<PathBuf> {
    let config_dir = get_config_dir().context("getting config directory")?;
    let config_path = config_dir.join("config.toml");

    Ok(config_path)
}

fn get_default_data_dir() -> anyhow::Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("", "", "pdoc")
        .ok_or(anyhow!("Couldn't get data directory"))?;
    let data_dir = project_dirs.data_dir();

    return Ok(data_dir.to_owned());
}

fn expand_tilde(path: &PathBuf) -> PathBuf {
    let path_str = path.as_os_str().to_string_lossy();
    let expanded_str = shellexpand::tilde(&path_str).to_string();
    let expanded_path = expanded_str.into();

    expanded_path
}

pub fn get_data_dir(config: &Config) -> anyhow::Result<PathBuf> {
    let data_dir_opt = config.storage.data_dir.as_ref().map(expand_tilde);

    if let Some(data_dir) = data_dir_opt {
        if data_dir.is_absolute() {
            Ok(data_dir)
        } else {
            bail!("data_dir must be absolute (found {:?})", data_dir)
        }
    } else {
        get_default_data_dir()
    }
}

pub fn get_projects_dir(config: &Config) -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir(config).context("getting data directory")?;
    let projects_dir = data_dir.join("projects");
    std::fs::create_dir_all(&projects_dir).context("creating projects directory")?;
    Ok(projects_dir)
}

pub fn get_clients_dir(config: &Config) -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir(config).context("getting data directory")?;
    let clients_dir = data_dir.join("clients");
    std::fs::create_dir_all(&clients_dir).context("creating clients directory")?;
    Ok(clients_dir)
}

pub fn get_invoices_dir(config: &Config) -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir(config).context("getting data directory")?;
    let invoices_dir = data_dir.join("invoices");
    std::fs::create_dir_all(&invoices_dir).context("creating invoices directory")?;
    Ok(invoices_dir)
}

pub fn get_receipts_dir(config: &Config) -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir(config).context("getting data directory")?;
    let receipts_dir = data_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir).context("creating receipts directory")?;
    Ok(receipts_dir)
}

pub fn get_pdfs_dir(config: &Config) -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir(config).context("getting data directory")?;
    let pdfs_dir = data_dir.join("pdfs");
    std::fs::create_dir_all(&pdfs_dir).context("creating pdfs directory")?;
    Ok(pdfs_dir)
}

pub fn get_beancount_dir(config: &Config) -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir(config).context("getting data directory")?;
    let beancount_dir = data_dir.join("beancount");
    std::fs::create_dir_all(&beancount_dir).context("creating beancount directory")?;
    Ok(beancount_dir)
}

pub fn find_project(id: &Id, config: &Config) -> anyhow::Result<Project> {
    let dir = get_projects_dir(config).context("getting projects directory")?;
    let filename = format!("{}.yaml", id);
    let path = dir.join(filename);
    let file = File::open(path).context("opening project file")?;
    let project: Project = serde_yaml::from_reader(file).context("deserializing project yaml")?;

    Ok(project)
}

pub fn find_client(id: &Id, config: &Config) -> anyhow::Result<Client> {
    let dir = get_clients_dir(config).context("getting clients directory")?;
    let filename = format!("{}.yaml", id);
    let path = dir.join(filename);
    let file = File::open(path).context("opening client file")?;
    let client: Client = serde_yaml::from_reader(file).context("deserializing client yaml")?;

    Ok(client)
}

pub fn find_invoice(number: u32, config: &Config) -> anyhow::Result<Invoice> {
    let dir = get_invoices_dir(config).context("getting invoices directory")?;
    let filename = format!("{}.yaml", number);
    let path = dir.join(filename);
    let file = File::open(path).context("opening invoice file")?;
    let invoice: Invoice = serde_yaml::from_reader(file).context("deserializing invoice yaml")?;

    Ok(invoice)
}
