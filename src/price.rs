use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PriceUSD(f32);

impl PriceUSD {
    pub fn as_f32(&self) -> f32 {
        self.0
    }
}

impl Display for PriceUSD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", &self.0)
    }
}

impl FromStr for PriceUSD {
    type Err = <f32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let amt: f32 = s.parse()?;
        Ok(Self(amt))
    }
}
