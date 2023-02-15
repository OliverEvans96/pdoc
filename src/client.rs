use serde::{Deserialize, Serialize};

use crate::{address::MailingAddress, contact::ContactInfo, id::Id};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Client {
    pub id: Id,
    pub name: String,
    pub address: MailingAddress,
    pub contact: ContactInfo,
}
