use std::ptr::addr_of;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug,  PartialEq, Clone)]
pub struct Config {
    pub finanz_api_key: String,
    pub finanz_api_url: String,
    pub auto_refresh: bool,
    pub offline_mode: bool,
}

static mut LOADED_CONFIG: Option<Config> = Option::None;

pub unsafe fn get_config() -> Option<Config> {
    if  LOADED_CONFIG.is_some(){
        return LOADED_CONFIG.clone();
    }
    LOADED_CONFIG = load_config();
    return LOADED_CONFIG.clone();
}
fn load_config() -> Option<Config> {
    let result = std::fs::read_to_string("./config/config.json");
    if  result.is_err() {
        return Option::None;
    }
    let result = serde_json::from_str(result.expect("Get Config File Content").as_str());
    if result.is_err() {
        return Option::None;
    }
    Option::Some(result.expect("Get Config Parsing Result"))
}
