use std::fmt::{Debug, Display, Formatter};
use std::fs::metadata;
use std::io::Error;
use std::sync::atomic::fence;
use chrono::ParseError;
use tauri::utils::config::parse::does_supported_file_name_exist;
use tauri::utils::debug_eprintln;
use tokio::fs;
use crate::fetch::FetchError::{IO, NoValue, Reqwest};
use reqwest::blocking::Client;
use crate::config::get_config;


/// Trait für alle Methoden die eine Api können muss.
/// T ist das Daten-Objekt das erzeugt wird.
pub trait FetchAble<T, R> {
    fn get_cache(&self, request_information: &mut R) -> Result<Option<T>, FetchError>;
    fn invalidate_cache(&self, request_information: &mut R) -> Result<(), FetchError>;
    fn fetch(&self, client: &Client, request_information: &mut R) -> Result<T, FetchError>;
    fn write_to_cache(&self, fetched: &T, request_information: &mut R) -> Result<(), FetchError>;
    fn find_last_cache(&self, request_information: &mut R) -> Result<Option<T>, FetchError>;
}

pub fn get_cache_or_fetch<T,R>(fetchAble: &Box<dyn FetchAble<T,R>>, client: &Client, request_information: &mut R) -> Result<T, FetchError> {
    let cache = fetchAble.get_cache(request_information)?;
    println!("Cache read");
    if  cache.is_some() {
        println!("Value from Cache");
        return Ok(cache.expect("Get From Cache"));
    }
    let config_option = unsafe { get_config() };
    let fetched;
    if config_option.is_none() || !config_option.expect("Get Config").offline_mode {
        fetched = fetchAble.fetch(client, request_information)?;
        println!("Fetched");
        fetchAble.invalidate_cache(request_information)?;
        println!("Cache removed");
        fetchAble.write_to_cache(&fetched, request_information)?;
        println!("Cached");
    } else {
        let fetched_option = fetchAble.find_last_cache(request_information)?;
        if  fetched_option.is_none(){
            return Err(NoValue("Kein Wert konnte aus dem gesamten Cache gelesen werden".to_string()));
        }
        fetched = fetched_option.expect("Wert aus gesamten Cache");
    }
    return Ok(fetched);
}

pub enum FetchError {
    Reqwest(reqwest::Error),
    IO(std::io::Error),
    SerdeJson(serde_json::Error),
    ChronoParse(ParseError),
    NoValue(String),
}

impl From<reqwest::Error> for FetchError {
    fn from(value: reqwest::Error) -> Self {
        Reqwest(value)
    }
}

impl From<std::io::Error> for FetchError {
    fn from(value: Error) -> Self {
        FetchError::IO(value)
    }
}

impl From<serde_json::Error> for FetchError {
    fn from(value: serde_json::Error) -> Self {
       FetchError::SerdeJson(value)
    }
}

impl From<ParseError> for FetchError {
    fn from(value: ParseError) -> Self {
        FetchError::ChronoParse(value)
    }
}

impl Debug for FetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            Reqwest(err) => {Debug::fmt(&err, f)}
            IO(err) => {Debug::fmt(&err, f)}
            FetchError::SerdeJson(err) => {Debug::fmt(&err, f)}
            FetchError::ChronoParse(err) => {Debug::fmt(&err, f)}
            FetchError::NoValue(description) => {writeln!(f,"Kein Wert konnte an der Stelle {} erhalten werden", description)}
        }
    }
}

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            Reqwest(err) => {
                std::fmt::Display::fmt(&err, f)
            }
            IO(err) => {std::fmt::Display::fmt(&err, f)}
            FetchError::SerdeJson(err) => {std::fmt::Display::fmt(&err, f)}
            FetchError::ChronoParse(err) => {
                writeln!(f,"Chrono Parse {:?}", &err.kind())?;
                std::fmt::Display::fmt(&err, f)}
            FetchError::NoValue(description) => {writeln!(f,"Kein Wert konnte an der Stelle {} erhalten werden", description)}
        }
    }
}

