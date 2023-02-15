use std::fmt::Display;

use hex::ToHex;
use rand::{distributions::Standard, prelude::Distribution, Rng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Id([u8; 8]);

impl Distribution<Id> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Id {
        Id(rng.gen())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.0.encode_hex();
        f.write_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use crate::Id;

    #[test]
    fn test_rand_id() {
        let mut rng = thread_rng();
        let id: Id = rng.gen();
        let id_str = id.to_string();
        assert_eq!(id_str.len(), 16)
    }
}
