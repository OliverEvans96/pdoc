use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MailingAddress {
    pub addr1: String,
    pub addr2: Option<String>,
    pub city: String,
    pub state: String,
    pub zip: String,
}
