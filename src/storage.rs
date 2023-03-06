use std::{fs::File, path::PathBuf};

use anyhow::{anyhow, bail};

use crate::{
    client::Client, config::read_config, id::Id, invoice::Invoice, me::Me, project::Project,
};

fn get_config_dir() -> anyhow::Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("", "", "pdoc")
        .ok_or(anyhow!("Couldn't get data directory"))?;
    let data_dir = project_dirs.config_dir();

    return Ok(data_dir.to_owned());
}

pub fn get_config_file_path() -> anyhow::Result<PathBuf> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");

    Ok(config_path)
}

fn get_default_data_dir() -> anyhow::Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("", "", "pdoc")
        .ok_or(anyhow!("Couldn't get data directory"))?;
    let data_dir = project_dirs.data_dir();

    return Ok(data_dir.to_owned());
}

fn expand_tilde(path: PathBuf) -> PathBuf {
    let path_str = path.as_os_str().to_string_lossy();
    let expanded_str = shellexpand::tilde(&path_str).to_string();
    let expanded_path = expanded_str.into();

    expanded_path
}

pub fn get_data_dir() -> anyhow::Result<PathBuf> {
    let data_dir_opt = read_config()
        .ok()
        .and_then(|config| config.data_dir)
        .map(expand_tilde);

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

pub fn get_projects_dir() -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir()?;
    let projects_dir = data_dir.join("projects");
    std::fs::create_dir_all(&projects_dir)?;
    Ok(projects_dir)
}

pub fn get_clients_dir() -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir()?;
    let clients_dir = data_dir.join("clients");
    std::fs::create_dir_all(&clients_dir)?;
    Ok(clients_dir)
}

pub fn get_invoices_dir() -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir()?;
    let invoices_dir = data_dir.join("invoices");
    std::fs::create_dir_all(&invoices_dir)?;
    Ok(invoices_dir)
}

pub fn get_receipts_dir() -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir()?;
    let receipts_dir = data_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir)?;
    Ok(receipts_dir)
}

pub fn get_pdfs_dir() -> anyhow::Result<PathBuf> {
    let data_dir = get_data_dir()?;
    let pdfs_dir = data_dir.join("pdfs");
    std::fs::create_dir_all(&pdfs_dir)?;
    Ok(pdfs_dir)
}

/// Reads personal info from yaml in app data dir
pub fn read_me() -> anyhow::Result<Me> {
    let data_dir = get_data_dir()?;

    let me_path = data_dir.join("me.yaml");
    let me_file = File::open(me_path)?;
    let me_yaml: Me = serde_yaml::from_reader(me_file)?;

    Ok(me_yaml)
}

pub fn find_project(id: &Id) -> anyhow::Result<Project> {
    let dir = get_projects_dir()?;
    let filename = format!("{}.yaml", id);
    let path = dir.join(filename);
    let file = File::open(path)?;
    let project: Project = serde_yaml::from_reader(file)?;

    Ok(project)
}

pub fn find_client(id: &Id) -> anyhow::Result<Client> {
    let dir = get_clients_dir()?;
    let filename = format!("{}.yaml", id);
    let path = dir.join(filename);
    let file = File::open(path)?;
    let client: Client = serde_yaml::from_reader(file)?;

    Ok(client)
}

pub fn find_invoice(number: u32) -> anyhow::Result<Invoice> {
    let dir = get_invoices_dir()?;
    let filename = format!("{}.yaml", number);
    let path = dir.join(filename);
    let file = File::open(path)?;
    let invoice: Invoice = serde_yaml::from_reader(file)?;

    Ok(invoice)
}
