use std::{cmp, fmt};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Write};
use std::marker::PhantomData;
use serde::{de, Deserializer, Serialize};
use serde::de::{SeqAccess, Visitor};
use serde::Deserialize;

#[derive(Serialize, Deserialize)]
pub struct FinanzData {
    #[serde(alias = "Meta Data")]
    pub mata_data: MetaData,
    #[serde(alias = "Weekly Time Series")]
    pub weekly_time_series: HashMap<String, ValueInformation>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ValueInformation {
    #[serde(alias = "1. open")]
    #[serde(deserialize_with = "from_str_to_float")]
    pub open: f32,
    #[serde(alias = "2. high")]
    #[serde(deserialize_with = "from_str_to_float")]
    pub high: f32,
    #[serde(alias = "3. low")]
    #[serde(deserialize_with = "from_str_to_float")]
    pub low: f32,
    #[serde(alias = "4. close")]
    #[serde(deserialize_with = "from_str_to_float")]
    pub close: f32,
    #[serde(alias = "5. volume")]
    #[serde(deserialize_with = "from_str_to_float")]
    pub volume: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaData {
    #[serde(alias = "1. Information")]
    pub information: String,

    #[serde(alias = "2. Symbol")]
    pub symbol: String,

    #[serde(alias = "3. Last Refreshed")]
    pub last_refreshed: String,

    #[serde(alias = "4. Time Zone")]
    pub time_zone: String
}

impl Debug for FinanzData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.mata_data)?;

        for (week, data) in &self.weekly_time_series {
            writeln!(f, "{}: {:?}", week, data)?;
        }

        write!(f, "")
    }
}

fn from_str_to_float<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let f = s.parse::<f32>().map_err(serde::de::Error::custom)?;
    Ok(f)
}
