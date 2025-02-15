use anyhow::bail;
use chrono::{Duration, Utc};
use reqwest;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashMap, f64::NAN};

// ChatGPT conversation: https://chatgpt.com/share/67b05bc2-8a50-800c-8080-d64aaf34be79

/// Global API endpoint (adjust as needed)
const API_ENDPOINT: &str = "https://projects.oliverevans.dev/api/v3";
const USERNAME: &str = "apikey";
const PASSWORD: Option<&'static str> =
    Some("179c173684d09e60d1c6814e4b82ad4c0bccd103abcbd31c22230c90e388ffa8");

// /// A simplified representation of a HAL Link.
// #[derive(Debug, Deserialize)]
// struct Link {
//     href: String,
//     title: Option<String>,
// }

// /// Generic HAL collection structure.
// #[derive(Debug, Deserialize)]
// struct Collection<T> {
//     #[serde(rename = "_embedded", default)]
//     embedded: Embedded<T>,
// }

// /// Representation of a time entry as returned by the API.
// #[derive(Debug, Deserialize)]
// struct TimeEntry {
//     id: u64,
//     /// Hours is given in an ISO8601 duration format (e.g. "PT5H" or "PT1H30M").
//     hours: String,
//     spentOn: String,
//     #[serde(rename = "_links")]
//     links: TimeEntryLinks,
// }

// #[derive(Debug, Deserialize)]
// struct TimeEntryLinks {
//     workPackage: Option<Link>,
// }

// /// The collection wrapper for time entries.
// #[derive(Debug, Deserialize)]
// struct TimeEntryCollection {
//     #[serde(rename = "_embedded")]
//     embedded: TimeEntryEmbedded,
// }

// #[derive(Debug, Deserialize)]
// struct TimeEntryEmbedded {
//     #[serde(default)]
//     elements: Vec<TimeEntry>,
// }

// /// A (simplified) work package representation. Note that custom fields
// /// (such as a “Completion Fee” or “Billed”) are captured in the `custom` map.
// #[derive(Debug, Deserialize)]
// struct WorkPackage {
//     id: u64,
//     subject: String,
//     updatedAt: String,
//     #[serde(rename = "_links")]
//     links: WPLinks,
//     #[serde(flatten)]
//     custom: HashMap<String, Value>,
// }

// /// We assume that the work package “type” and “status” are provided as linked resources.
// #[derive(Debug, Deserialize)]
// struct WPLinks {
//     #[serde(rename = "type")]
//     type_link: Link,
//     status: Link,
// }

// /// The collection wrapper for work packages.
// #[derive(Debug, Deserialize)]
// struct WorkPackageCollection {
//     #[serde(rename = "_embedded")]
//     embedded: WPEmbedded,
// }

// #[derive(Debug, Deserialize)]
// struct WPEmbedded {
//     #[serde(default)]
//     elements: Vec<WorkPackage>,
// }

// /// Work package type as returned by the API.
// #[derive(Debug, Default, Deserialize)]
// struct WPType {
//     id: u64,
//     name: String,
// }

// /// Work package status.
// #[derive(Debug, Default, Deserialize)]
// struct Status {
//     id: u64,
//     name: String,
// }

// /// Custom field.
// #[derive(Debug, Default, Deserialize)]
// struct CustomField {
//     id: u64,
//     name: String,
// }

// /// A very basic parser for ISO8601 durations like "PT5H" or "PT1H30M".
// fn parse_duration(duration: &str) -> f64 {
//     // Remove the leading "PT"
//     let s = duration.trim_start_matches("PT");
//     let mut hours = 0.0;
//     let mut minutes = 0.0;
//     if let Some(h_index) = s.find('H') {
//         let h_str = &s[..h_index];
//         hours = h_str.parse().unwrap_or(0.0);
//         let rest = &s[h_index + 1..];
//         if let Some(m_index) = rest.find('M') {
//             let m_str = &rest[..m_index];
//             minutes = m_str.parse().unwrap_or(0.0);
//         }
//     } else if let Some(m_index) = s.find('M') {
//         let m_str = &s[..m_index];
//         minutes = m_str.parse().unwrap_or(0.0);
//     }
//     hours + minutes / 60.0
// }
// /// Query the API for WP types and build a map: name -> id (as string)
// async fn fetch_wp_types() -> Result<HashMap<String, String>, reqwest::Error> {
//     let url = format!("{}/types", API_ENDPOINT);
//     let client = reqwest::Client::new();
//     let resp = client.get(&url).send().await?;
//     let collection: Collection<WPType> = resp.json().await?;
//     let mut map = HashMap::new();
//     for t in collection.embedded.elements {
//         map.insert(t.name.clone(), t.id.to_string());
//     }
//     Ok(map)
// }

// /// Query the API for statuses.
// async fn fetch_statuses() -> Result<HashMap<String, String>, reqwest::Error> {
//     let url = format!("{}/statuses", API_ENDPOINT);
//     let client = reqwest::Client::new();
//     let resp = client.get(&url).send().await?;
//     let collection: Collection<Status> = resp.json().await?;
//     let mut map = HashMap::new();
//     for s in collection.embedded.elements {
//         map.insert(s.name.clone(), s.id.to_string());
//     }
//     Ok(map)
// }

// /// Query the API for custom fields. In this example, we assume the API returns a collection
// /// where each custom field has an id and a name. For filters, we construct an alias like "customField10".
// async fn fetch_custom_fields() -> Result<HashMap<String, String>, reqwest::Error> {
//     let url = format!("{}/custom_fields", API_ENDPOINT);
//     let client = reqwest::Client::new();
//     let resp = client.get(&url).send().await?;
//     let collection: Collection<CustomField> = resp.json().await?;
//     let mut map = HashMap::new();
//     for cf in collection.embedded.elements {
//         // For filtering, assume the API expects custom fields keyed as "customField<ID>"
//         map.insert(cf.name.clone(), format!("customField{}", cf.id));
//     }
//     Ok(map)
// }

// /// Fetch all time entries for a given project (filtered by the "spent_on" date range).
// async fn fetch_time_entries(
//     project_id: u64,
//     start_date: &str,
//     end_date: &str,
// ) -> Result<Vec<TimeEntry>, reqwest::Error> {
//     let client = reqwest::Client::new();
//     // Build filters as JSON. Note: operator "<>d" means “between two dates.”
//     let filters = serde_json::json!([
//         { "project": { "operator": "=", "values": [project_id.to_string()] } },
//         { "spent_on": { "operator": "<>d", "values": [start_date, end_date] } }
//     ]);
//     let url = format!("{}/time_entries", API_ENDPOINT);
//     let resp = client
//         .get(&url)
//         .query(&[("filters", filters.to_string())])
//         .basic_auth(USERNAME, PASSWORD)
//         .send()
//         .await?;
//     let collection: TimeEntryCollection = resp.json().await?;
//     println!("time entries: {:#?}", collection);
//     Ok(collection.embedded.elements)
// }

// /// Fetch a single work package by its id.
// async fn fetch_work_package(wp_id: u64) -> Result<WorkPackage, reqwest::Error> {
//     let url = format!("{}/work_packages/{}", API_ENDPOINT, wp_id);
//     let client = reqwest::Client::new();
//     let resp = client
//         .get(&url)
//         .basic_auth(USERNAME, PASSWORD)
//         .send()
//         .await?;
//     let wp: WorkPackage = resp.json().await?;
//     println!("wp {}: {:#?}", wp_id, wp);
//     Ok(wp)
// }

// #[tokio::test]
// async fn test_openproject() -> anyhow::Result<()> {
//     // Parameters (set these as needed)
//     let project_id = 4; // the project you want to query
//     let n_days = 7; // look back N days

//     // Compute date range in "YYYY-MM-DD" format.
//     let now = Utc::now();
//     let start_date = (now - Duration::days(n_days))
//         .format("%Y-%m-%d")
//         .to_string();
//     let end_date = now.format("%Y-%m-%d").to_string();

//     // --- PART 1: Time logged per work package type ---

//     // Fetch all time entries in the project from the last N days.
//     let time_entries = fetch_time_entries(project_id, &start_date, &end_date).await?;

//     // Group time entries by work package id and sum hours.
//     let mut wp_hours: HashMap<u64, f64> = HashMap::new();
//     for te in time_entries {
//         if let Some(wp_link) = te.links.workPackage {
//             // The href is assumed to be like "/api/v3/work_packages/123"
//             if let Some(id_str) = wp_link.href.split('/').last() {
//                 if let Ok(wp_id) = id_str.parse::<u64>() {
//                     let hrs = parse_duration(&te.hours);
//                     *wp_hours.entry(wp_id).or_insert(0.0) += hrs;
//                 }
//             }
//         }
//     }

//     // For each work package, fetch its details (to know its type) and add to totals.
//     let mut task_total = 0.0;
//     let mut feature_total = 0.0;
//     for (&wp_id, &hours) in &wp_hours {
//         let wp = fetch_work_package(wp_id).await?;
//         let wp_type = wp.links.type_link.title.unwrap_or_default();
//         if wp_type.eq_ignore_ascii_case("Task") {
//             task_total += hours;
//         } else if wp_type.eq_ignore_ascii_case("Feature") {
//             feature_total += hours;
//         }
//     }
//     println!("Time logged in the last {} days:", n_days);
//     println!("  Tasks:    {:.2} hours", task_total);
//     println!("  Features: {:.2} hours", feature_total);

//     // --- PART 2: List features accepted (status "Accepted") in the last N days that are not billed ---
//     // We assume that only work packages of type "Feature" have two custom fields:
//     //   • One (say, customField10) holds the "Completion Fee"
//     //   • One (say, under the name "Billed") indicates if the feature is billed.
//     // We also assume that a status change to "Accepted" is reflected by the current status
//     // and that the work package was updated in the last N days.

//     // Query the API for translations.
//     let wp_types = fetch_wp_types().await?;
//     let statuses = fetch_statuses().await?;
//     let custom_fields = fetch_custom_fields().await?;
//     println!("Fetched WP Types: {:?}", wp_types);
//     println!("Fetched Statuses: {:?}", statuses);
//     println!("Fetched Custom Fields: {:?}", custom_fields);

//     // Use the queried translations:
//     let feature_type = wp_types
//         .get("Feature")
//         .cloned()
//         .unwrap_or("Feature".to_string());
//     let accepted_status = statuses
//         .get("Accepted")
//         .cloned()
//         .unwrap_or("Accepted".to_string());
//     let billed_field = custom_fields
//         .get("Billed")
//         .cloned()
//         .unwrap_or("Billed".to_string());

//     // Build a filters JSON array. (Depending on your installation you might need to use the type id rather than the string "Feature.")
//     let filters = json!([
//         { "project":   { "operator": "=", "values": [project_id.to_string()] } },
//         { "type":      { "operator": "=", "values": [feature_type] } },
//         { "status":    { "operator": "=", "values": [accepted_status] } },
//         { "updatedAt": { "operator": "<>d", "values": [start_date.clone(), end_date.clone()] } },
//         { billed_field: { "operator": "=", "values": ["false"] } }
//     ]);
//     let wp_url = format!("{}/work_packages", API_ENDPOINT);
//     let client = reqwest::Client::new();
//     let resp = client
//         .get(&wp_url)
//         .query(&[("filters", filters.to_string())])
//         .basic_auth(USERNAME, PASSWORD)
//         .send()
//         .await?;
//     println!("wp coll resp: {}", resp.text().await?);
//     bail!("pre");
//     let wp_collection: WorkPackageCollection = resp.json().await?;
//     println!("wp_collection: {:#?}", wp_collection);
//     println!(
//         "\nFeatures that changed to 'Accepted' in the last {} days and are not yet billed:",
//         n_days
//     );
//     for wp in wp_collection.embedded.elements {
//         // Here we try to extract the completion fee from the custom fields.
//         // In this example we assume it is provided in "customField10".
//         let fee = wp
//             .custom
//             .get("customField10")
//             .and_then(|v| v.as_str())
//             .unwrap_or("N/A");
//         println!("  Feature '{}': Completion Fee = {}", wp.subject, fee);
//     }

//     bail!("oops");

//     Ok(())
// }

// Attempt #2

/// Generic HAL collection structure.
#[derive(Debug, Deserialize)]
struct Collection<T> {
    #[serde(rename = "_embedded")]
    embedded: Embedded<T>,
}

#[derive(Debug, Deserialize, Default)]
struct Embedded<T> {
    elements: Vec<T>,
}

/// Work package type as returned by the API.
#[derive(Debug, Deserialize)]
struct WPType {
    id: u64,
    name: String,
}

/// Work package status.
#[derive(Debug, Deserialize)]
struct Status {
    id: u64,
    name: String,
}

/// Custom field.
#[derive(Debug, Deserialize)]
struct CustomField {
    id: u64,
    name: String,
}

/// Query the API for WP types and build a map: name -> id (as string)
async fn fetch_wp_types() -> Result<HashMap<String, String>, reqwest::Error> {
    let url = format!("{}/types", API_ENDPOINT);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .basic_auth(USERNAME, PASSWORD)
        .send()
        .await?;
    let collection: Collection<WPType> = resp.json().await?;
    let mut map = HashMap::new();
    for t in collection.embedded.elements {
        map.insert(t.name.clone(), t.id.to_string());
    }
    Ok(map)
}

/// Query the API for statuses.
async fn fetch_statuses() -> Result<HashMap<String, String>, reqwest::Error> {
    let url = format!("{}/statuses", API_ENDPOINT);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .basic_auth(USERNAME, PASSWORD)
        .send()
        .await?;
    let collection: Collection<Status> = resp.json().await?;
    let mut map = HashMap::new();
    for s in collection.embedded.elements {
        map.insert(s.name.clone(), s.id.to_string());
    }
    Ok(map)
}

/// Query the API for custom fields. In this example, we assume the API returns a collection
/// where each custom field has an id and a name. For filters, we construct an alias like "customField10".
async fn fetch_custom_fields() -> anyhow::Result<HashMap<String, String>> {
    // NOTE: This endpoint doesn't exist
    let url = format!("{}/custom_fields", API_ENDPOINT);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .basic_auth(USERNAME, PASSWORD)
        .send()
        .await?;
    println!("cf resp: {}", resp.text().await?);
    bail!("cf bail");
    let collection: Collection<CustomField> = resp.json().await?;
    let mut map = HashMap::new();
    for cf in collection.embedded.elements {
        // For filtering, assume the API expects custom fields keyed as "customField<ID>"
        map.insert(cf.name.clone(), format!("customField{}", cf.id));
    }
    Ok(map)
}

/// Structures for time entries and work packages remain as before.

#[derive(Debug, Deserialize)]
struct Link {
    href: String,
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TimeEntry {
    id: u64,
    hours: String, // ISO8601 duration (e.g. "PT5H" or "PT1H30M")
    spentOn: String,
    #[serde(rename = "_links")]
    links: TimeEntryLinks,
}

#[derive(Debug, Deserialize)]
struct TimeEntryLinks {
    workPackage: Option<Link>,
}

#[derive(Debug, Deserialize)]
struct TimeEntryCollection {
    #[serde(rename = "_embedded")]
    embedded: Embedded<TimeEntry>,
}

#[derive(Debug, Deserialize)]
struct WPLinks {
    #[serde(rename = "type")]
    type_link: Link,
    status: Link,
}

#[derive(Debug, Deserialize)]
struct WorkPackage {
    id: u64,
    subject: String,
    updatedAt: String,
    #[serde(rename = "_links")]
    links: WPLinks,
    #[serde(flatten)]
    custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct WorkPackageCollection {
    #[serde(rename = "_embedded")]
    embedded: Embedded<WorkPackage>,
}

/// Basic ISO8601 duration parser for "PT5H" or "PT1H30M"
fn parse_duration(duration: &str) -> f64 {
    let s = duration.trim_start_matches("PT");
    let mut hours = 0.0;
    let mut minutes = 0.0;
    if let Some(h_index) = s.find('H') {
        let h_str = &s[..h_index];
        hours = h_str.parse().unwrap_or(0.0);
        let rest = &s[h_index + 1..];
        if let Some(m_index) = rest.find('M') {
            let m_str = &rest[..m_index];
            minutes = m_str.parse().unwrap_or(0.0);
        }
    } else if let Some(m_index) = s.find('M') {
        let m_str = &s[..m_index];
        minutes = m_str.parse().unwrap_or(0.0);
    }
    hours + minutes / 60.0
}

/// Fetch time entries for a given project within a date range.
async fn fetch_time_entries(
    project_id: u64,
    n_days: i64,
) -> Result<Vec<TimeEntry>, reqwest::Error> {
    let client = reqwest::Client::new();
    let filters = json!([
        { "project": { "operator": "=", "values": [project_id.to_string()] } },
        { "spent_on": { "operator": ">t-", "values": [n_days] } }
    ]);
    let url = format!("{}/time_entries", API_ENDPOINT);
    let resp = client
        .get(&url)
        .query(&[("filters", filters.to_string())])
        .basic_auth(USERNAME, PASSWORD)
        .send()
        .await?;
    let collection: TimeEntryCollection = resp.json().await?;
    Ok(collection.embedded.elements)
}

/// Fetch a work package by its id.
async fn fetch_work_package(wp_id: u64) -> Result<WorkPackage, reqwest::Error> {
    let url = format!("{}/work_packages/{}", API_ENDPOINT, wp_id);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .basic_auth(USERNAME, PASSWORD)
        .send()
        .await?;
    let wp: WorkPackage = resp.json().await?;
    Ok(wp)
}

#[tokio::test]
async fn test_openproject() -> anyhow::Result<()> {
    // Query the API for translations.
    let wp_types = fetch_wp_types().await?;
    println!("Fetched WP Types: {:?}", wp_types);
    let statuses = fetch_statuses().await?;
    println!("Fetched Statuses: {:?}", statuses);
    // let custom_fields = fetch_custom_fields().await?;
    // println!("Fetched Custom Fields: {:?}", custom_fields);

    // Parameters (adjust as needed)
    let project_id = 4;
    let n_days = 7;
    let now = Utc::now();

    // --- PART 1: Sum time logged by WP type ---
    let time_entries = fetch_time_entries(project_id, n_days).await?;
    let mut wp_hours: HashMap<u64, f64> = HashMap::new();
    for te in time_entries {
        if let Some(wp_link) = te.links.workPackage {
            if let Some(id_str) = wp_link.href.split('/').last() {
                if let Ok(wp_id) = id_str.parse::<u64>() {
                    let hrs = parse_duration(&te.hours);
                    *wp_hours.entry(wp_id).or_insert(0.0) += hrs;
                }
            }
        }
    }

    let mut task_total = 0.0;
    let mut feature_total = 0.0;
    for (&wp_id, &hours) in &wp_hours {
        let wp = fetch_work_package(wp_id).await?;
        let wp_type_name = wp.links.type_link.title.unwrap_or_default();
        println!("{} #{}: {} - {} hours", wp_type_name, wp_id, wp.subject, hours);
        if wp_type_name.eq_ignore_ascii_case("Task") {
            task_total += hours;
        } else if wp_type_name.eq_ignore_ascii_case("Feature") {
            feature_total += hours;
        }
    }
    println!("\nTime logged in the last {} days:", n_days);
    println!("  Tasks:    {:.2} hours", task_total);
    println!("  Features: {:.2} hours", feature_total);

    // --- PART 2: Filter features that changed to 'Accepted' and are not billed ---
    // Use the queried translations:
    let feature_type = wp_types
        .get("Feature")
        .cloned()
        .unwrap_or("Feature".to_string());
    let accepted_status = statuses
        .get("Accepted")
        .cloned()
        .unwrap_or("Accepted".to_string());
    let billed_field = "customField6";
    let fee_field = "customField1";

    let filters = json!([
        { "project":   { "operator": "=", "values": [project_id.to_string()] } },
        { "type":      { "operator": "=", "values": [feature_type] } },
        { "status":    { "operator": "=", "values": [accepted_status] } },
        { "updatedAt": { "operator": ">t-", "values": [n_days] } },
        { billed_field: { "operator": "=", "values": ["f"] } }
    ]);
    let wp_url = format!("{}/work_packages", API_ENDPOINT);
    let client = reqwest::Client::new();
    let resp = client
        .get(&wp_url)
        .query(&[("filters", filters.to_string())])
        .basic_auth(USERNAME, PASSWORD)
        .send()
        .await?;

    // println!("wp coll resp: {}", resp.text().await?);
    // bail!("pre");
    let wp_collection: WorkPackageCollection = resp.json().await?;
    println!(
        "\nFeatures that changed to 'Accepted' in the last {} days and are not yet billed:",
        n_days
    );
    for wp in wp_collection.embedded.elements {
        // Here we extract the "Completion Fee" from custom fields.
        // println!("wp: {:#?}", &wp);
        let fee = wp
            .custom
            .get(fee_field)
            .and_then(|v| v.as_f64())
            .unwrap_or(NAN);
        println!("  Feature '{}': Completion Fee = {}", wp.subject, fee);
    }

    bail!("oops");

    Ok(())
}
