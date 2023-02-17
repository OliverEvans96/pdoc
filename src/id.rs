use std::{
    convert::Infallible,
    fmt::{Debug, Display},
    path::Path,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct Id(String);

impl Id {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl From<String> for Id {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<Id> for String {
    fn from(id: Id) -> Self {
        id.0
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Id").field(&self.to_string()).finish()
    }
}

#[derive(Clone, Debug, Error)]
pub enum IdDecodeError {
    #[error("Filename {0:?} has no basename")]
    NoBasename(String),
}

impl FromStr for Id {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Id {
    pub fn to_filename(&self) -> String {
        format!("{}.yaml", self)
    }

    pub fn from_filename(filename: impl AsRef<Path>) -> Result<Self, IdDecodeError> {
        let basename = filename
            .as_ref()
            .file_stem()
            .map(|stem| stem.to_string_lossy())
            .ok_or(IdDecodeError::NoBasename(
                filename.as_ref().to_string_lossy().to_string(),
            ))?;

        let id = Id::new(basename.to_string());

        Ok(id)
    }
}
