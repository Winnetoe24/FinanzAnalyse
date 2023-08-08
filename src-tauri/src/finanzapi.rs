use std::{cmp, fmt};
use std::collections::HashMap;
use std::fmt::{Debug, format, Formatter, Write};
use std::marker::PhantomData;
use std::ops::Add;
use std::time::Instant;
use chrono::{DateTime, Local, TimeZone};
use reqwest::{Error};
use serde::{de, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::Deserialize;
use tauri::async_runtime::TokioHandle;
use crate::fetch::{FetchAble, FetchError};
use reqwest::blocking::Client;
use tauri::api::path::resolve_path;

pub struct FinanzApiFetchData {
    pub(crate) api_key: String,
    pub(crate) url: String,
}

pub enum FinanzApiRequestInformation {
    Search{
        input: String,
    },
    GetWeekly {
        key: String,
        from: Option<DateTime<Local>>,
        bis: DateTime<Local>
    }
}

impl FinanzApiRequestInformation {

    fn getCacheDirPath(&self) -> String {
        return match self {
            FinanzApiRequestInformation::Search { input } => {
                "./cache/finanzapi/search".to_string()
            }
            FinanzApiRequestInformation::GetWeekly { key, from, bis } => {
                "./cache/finanzapi/getweekly/".to_string().add(key)
            }
        }
    }
    pub fn getCachePath(&self) -> String {
        return match self {
            FinanzApiRequestInformation::Search { input } => {
                self.getCacheDirPath().add("/").add(input.as_str()).add(".json")
            }
            FinanzApiRequestInformation::GetWeekly { key, from, bis } => {
                self.getCacheDirPath().add("/").add(format!("{}", bis.format("%d-%m-%Y")).as_str()).add(".json")
            }
        }
    }



    fn setBis(&mut self, neues_bis: String) -> Result<FinanzApiRequestInformation, FetchError>{
        return match self {
            FinanzApiRequestInformation::Search { .. } => { todo!()}
            FinanzApiRequestInformation::GetWeekly { key, from, bis } => {
                let result = Local.datetime_from_str(neues_bis.as_str(), "%Y-%m-%d")?;
                Ok(FinanzApiRequestInformation::GetWeekly {
                    key: key.clone(),
                    from: from.clone(),
                    bis: result,
                })
            }
        }
    }
}

impl FetchAble<FinanzData, FinanzApiRequestInformation> for FinanzApiFetchData {
    fn get_cache(&self, request_information: &mut FinanzApiRequestInformation) -> Result<Option<FinanzData>, FetchError> {
        let cache_path = request_information.getCachePath();
        let file_content_result = std::fs::read_to_string(cache_path);
         if file_content_result.is_err() {
             return Ok(None);
         }
        let file_content = file_content_result.expect("Get File Content");
        let result = serde_json::from_str(file_content.as_str())?;
        Ok(Some(result))
    }

    fn invalidate_cache(&self, request_information: &mut FinanzApiRequestInformation) -> Result<(), FetchError> {
        let cache_path = request_information.getCachePath();
        let _ = std::fs::remove_file(cache_path);
        Ok(())
    }

    fn fetch(&self, client: &Client, request_information: &mut FinanzApiRequestInformation) -> Result<FinanzData, FetchError> {
        return  match request_information {
            FinanzApiRequestInformation::Search { input } => { todo!()}
            FinanzApiRequestInformation::GetWeekly { key, from, bis } => {
                let response = client.get(&self.url)
                    .query(&[("function","TIME_SERIES_WEEKLY"),("symbol", key), ("apikey",&self.api_key),("datatype","json")])
                    .send()?
                    .text()?;
                println!("{}", response);
                let result = serde_json::from_str(response.as_str())?;
                Ok(result)
            }
        }

    }

    fn write_to_cache(&self, fetched: &FinanzData, request_information: &mut FinanzApiRequestInformation) -> Result<(), FetchError> {
        let request_information = request_information.setBis(fetched.mata_data.last_refreshed.clone())?;
        let cache_dir_path = request_information.getCacheDirPath();
        std::fs::create_dir_all(cache_dir_path)?;
        let cache_path = request_information.getCachePath();
        let string = serde_json::to_string(fetched)?;
        std::fs::write(cache_path,string)?;
        Ok(())
    }
}




#[derive(Serialize, Deserialize,  PartialEq)]
pub struct FinanzData {
    #[serde(alias = "Meta Data")]
    pub mata_data: MetaData,
    #[serde(alias = "Weekly Time Series")]
    pub weekly_time_series: HashMap<String, ValueInformation>
}


#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct ValueInformation {
    #[serde(alias = "1. open")]
    #[serde(deserialize_with = "from_str_to_float")]
    #[serde(serialize_with = "serialize_f32")]
    pub open: f32,
    #[serde(alias = "2. high")]
    #[serde(deserialize_with = "from_str_to_float")]
    #[serde(serialize_with = "serialize_f32")]
    pub high: f32,
    #[serde(alias = "3. low")]
    #[serde(deserialize_with = "from_str_to_float")]
    #[serde(serialize_with = "serialize_f32")]
    pub low: f32,
    #[serde(alias = "4. close")]
    #[serde(deserialize_with = "from_str_to_float")]
    #[serde(serialize_with = "serialize_f32")]
    pub close: f32,
    #[serde(alias = "5. volume")]
    #[serde(deserialize_with = "from_str_to_float")]
    #[serde(serialize_with = "serialize_f32")]
    pub volume: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

fn serialize_f32<S>(x: &f32, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    s.serialize_str(&format!("{:.4}", x))
}



