use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::id::Id;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub client_ref: Id,
}
