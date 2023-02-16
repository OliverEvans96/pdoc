use std::{
    fmt::{Debug, Display},
    path::Path,
    str::FromStr,
};

use rand::{distributions::Standard, prelude::Distribution, Rng};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const ID_LEN: usize = 10;

#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct Id([u8; ID_LEN]);

impl TryFrom<String> for Id {
    type Error = IdDecodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Id::from_str(&value)
    }
}

impl From<Id> for String {
    fn from(id: Id) -> Self {
        id.to_string()
    }
}

impl Distribution<Id> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Id {
        Id(rng.gen())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let s: String = self.0.encode_hex();
        let s = base32::encode(base32::Alphabet::Crockford, &self.0).to_lowercase();
        f.write_str(&s)
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Id").field(&self.to_string()).finish()
    }
}

#[derive(Clone, Debug, Error)]
pub enum IdDecodeError {
    #[error("Invalid base32 id: {0:?}")]
    NotBase32(String),
    #[error("decoded base32 to incorrect length: {0} bytes (expected {})", ID_LEN)]
    IncorrectLength(usize),
    #[error("Filename {0:?} has no basename")]
    NoBasename(String),
}

impl FromStr for Id {
    type Err = IdDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = base32::decode(base32::Alphabet::Crockford, s)
            .ok_or(IdDecodeError::NotBase32(s.to_owned()))?;

        let decoded_len = decoded.len();
        if decoded_len != ID_LEN {
            return Err(IdDecodeError::IncorrectLength(decoded_len));
        }

        let mut bytes = [0; ID_LEN];
        bytes.copy_from_slice(&decoded);
        let id = Id(bytes);

        Ok(id)
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

        let id = Id::from_str(&basename)?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use super::Id;

    #[test]
    fn test_rand_id() {
        let mut rng = thread_rng();
        let id: Id = rng.gen();
        let id_str = id.to_string();
        assert_eq!(id_str.len(), 16)
    }
}
