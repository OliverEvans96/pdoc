use serde::{Deserialize, Serialize};

use crate::{address::MailingAddress, contact::ContactInfo};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PaymentMethod {
    Text(String),
    Link { text: String, url: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Me {
    pub name: String,
    pub address: MailingAddress,
    pub contact: ContactInfo,
    pub payment: Vec<PaymentMethod>,
}
