use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContactInfo {
    pub email: String,
    pub phone: String,
}
