use std::{cmp, fmt};
use std::collections::HashMap;
use std::fmt::{Debug, format, Formatter, Write};
use std::marker::PhantomData;
use std::ops::Add;
use std::time::Instant;
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use itertools::Itertools;
use reqwest::{Error};
use serde::{de, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::Deserialize;
use tauri::async_runtime::TokioHandle;
use crate::fetch::{FetchAble, FetchError};
use reqwest::blocking::Client;
use reqwest::header::IF_MATCH;
use tauri::api::path::resolve_path;
use tauri::utils::resources::resource_relpath;
use crate::config::Config;

pub struct FinanzApiFetchData {
    pub(crate) api_key: String,
    pub(crate) url: String,
}

impl From<Config> for FinanzApiFetchData {
    fn from(value: Config) -> Self {
        return FinanzApiFetchData {
            api_key: value.finanz_api_key,
            url: value.finanz_api_url,
        }
    }
}

pub enum FinanzApiRequestInformation {
    Search{
        input: String,
    },
    GetWeekly {
        key: String,
        from: Option<NaiveDate>,
        bis: NaiveDate
    }
}

impl FinanzApiRequestInformation {
    pub(self) fn get_cache_dir_path(&self) -> String {
        return match self {
            FinanzApiRequestInformation::Search { input } => {
                "./cache/finanzapi/search".to_string()
            }
            FinanzApiRequestInformation::GetWeekly { key, from, bis } => {
                "./cache/finanzapi/getweekly/".to_string().add(key)
            }
        }
    }
    pub fn get_cache_path(&self) -> String {
        return match self {
            FinanzApiRequestInformation::Search { input } => {
                self.get_cache_dir_path().add("/").add(input.as_str()).add(".json")
            }
            FinanzApiRequestInformation::GetWeekly { key, from, bis } => {
                self.get_cache_dir_path().add("/").add(format!("{}", bis.format("%d-%m-%Y")).as_str()).add(".json")
            }
        }
    }

    fn set_bis(&mut self, neues_bis: String) -> Result<FinanzApiRequestInformation, FetchError>{
        return match self {
            FinanzApiRequestInformation::Search { .. } => { todo!()}
            FinanzApiRequestInformation::GetWeekly { key, from, bis } => {
                println!("{}", &neues_bis);
                let result = NaiveDate::parse_from_str(&neues_bis.as_str(), "%Y-%m-%d")?;
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
        let cache_path = request_information.get_cache_path();
        let file_content_result = std::fs::read_to_string(cache_path);
         if file_content_result.is_err() {
             return Ok(None);
         }
        let file_content = file_content_result.expect("Get File Content");
        let result = serde_json::from_str(file_content.as_str())?;
        Ok(Some(result))
    }

    fn invalidate_cache(&self, request_information: &mut FinanzApiRequestInformation) -> Result<(), FetchError> {
        let cache_path = request_information.get_cache_path();
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
                let result = serde_json::from_str(response.as_str())?;
                Ok(result)
            }
        }

    }

    fn write_to_cache(&self, fetched: &FinanzData, request_information: &mut FinanzApiRequestInformation) -> Result<(), FetchError> {
        let request_information = request_information.set_bis(fetched.mata_data.last_refreshed.clone())?;
        let cache_dir_path = request_information.get_cache_dir_path();
        std::fs::create_dir_all(cache_dir_path)?;
        let cache_path = request_information.get_cache_path();
        let string = serde_json::to_string(fetched)?;
        std::fs::write(cache_path,string)?;
        Ok(())
    }

    fn find_last_cache(&self, request_information: &mut FinanzApiRequestInformation) -> Result<Option<FinanzData>, FetchError> {
        let cache_dir_path = request_information.get_cache_dir_path();
        let mut date_vec = vec![];
        for dir_entry in std::fs::read_dir(&cache_dir_path)? {
            if dir_entry.is_err() { continue; }
            let entry = dir_entry.expect("Get Dir Entry");
            println!("Dir entry: \"{:?}\"", &entry.file_name());
            let len = &entry.file_name().len();
            if len <= &5 { continue; }
            let name_without_json = &entry
                .file_name()
                .into_string()
                .expect("String")
                .chars()
                .take(len - 5)
                .join("");
            println!("Date: \"{}\"", &name_without_json);
            let date = NaiveDate::parse_from_str(name_without_json, "%d-%m-%Y")?;
            println!("Date parsed");
            date_vec.push(date);
        }
        date_vec.sort();
        date_vec.reverse();
        let last_date = date_vec.get(0);
        if last_date.is_none() {
            println!("No Date Found");
            return Ok(None);
        }
        println!("Date Vec {:?}", &date_vec);
        let cache_path = cache_dir_path.add("/").add(&last_date.expect("Get Date").format("%d-%m-%Y").to_string()).add(".json");
        println!("CachePath {}", &cache_path);
        let file_content_result = std::fs::read_to_string(cache_path);
        if file_content_result.is_err() {
            return Ok(None);
        }
        let file_content = file_content_result.expect("Get File Content");
        let result = serde_json::from_str(file_content.as_str())?;
        Ok(Some(result))
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



