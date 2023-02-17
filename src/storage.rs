use std::{fs::File, path::PathBuf};

use anyhow::anyhow;

use crate::{client::Client, id::Id, me::Me, project::Project};

fn get_data_dir() -> anyhow::Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("", "", "pdoc")
        .ok_or(anyhow!("Couldn't get data directory"))?;
    let data_dir = project_dirs.data_dir();

    return Ok(data_dir.to_owned());
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
