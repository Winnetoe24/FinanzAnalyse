// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod finanzapi;
mod fetch;
mod data;
mod config;

use std::arch::x86_64::__m128;
use std::fs;
use std::time::Instant;
use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
use reqwest::Error;
use reqwest::blocking::Client;
use crate::config::get_config;
use crate::data::parse_to_view_data;
use crate::fetch::FetchAble;
use crate::finanzapi::{FinanzApiFetchData, FinanzApiRequestInformation, FinanzData};

struct State{
  client: reqwest::blocking::Client
}

fn main() {
    let time = Local::now();
    println!("{}", (&time).format("%Y-%m-%d"));
    NaiveDate::parse_from_str("2023-08-13", "%Y-%m-%d").expect("");
    // let  cache: FinanzData = serde_json::from_str(fs::read_to_string("./cache.json").expect("File").as_str()).expect("serde");
    let client = reqwest::blocking::Client::new();
    let config = unsafe { get_config() }.expect("Keine Config vorhanden.");
    let data = FinanzApiFetchData::from(config);
    let mut information = FinanzApiRequestInformation::GetWeekly {
        key: "IBM".to_string(),
        from: None,
        bis: Local::now().date_naive()
    };

    println!("{}", &information.get_cache_path());

    let finanz_box: Box<dyn FetchAble<FinanzData, FinanzApiRequestInformation>> = Box::new(data);
    let result1 = fetch::get_cache_or_fetch(&finanz_box, &client, &mut information);
    if result1.is_err() {
        let error = result1.err().expect("");
        println!("{}", &error);
        panic!("{}", error);

    }
    let result = result1.expect("Cache or fetch");
    let to_view_data = parse_to_view_data(result).expect("View Data");

    println!("{:?}", to_view_data);
    // println!("{:?}", result);
    // println!("Equals: {}",result.eq(&cache));


    // println!("{:?}", cache);
    // tauri::Builder::default()
  //     .manage(State{
  //       client: reqwest::Client::new()
  //     })
  //     .invoke_handler(tauri::generate_handler![my_custom_command])
  //   .run(tauri::generate_context!())
  //   .expect("error while running tauri application");
}

// Declare the async function using String instead of &str, as &str is borrowed and thus unsupported
// #[tauri::command]
// async fn my_custom_command(state: tauri::State<'_, State>, value: String) -> Result<String,String> {
//   // Call another async function and wait for it to finish
//   let v = fetchGoogle(state).await;
//     if v.is_err() {
//         return Err(v.err().expect("No Error when Error").to_string());
//     }
//
//     let string = v.expect("Fehler ohne Error");
//     Ok(string.replace("href=\"/", "href=\"https://www.google.com/").replace("src=\"/", "src=\"https://www.google.com/"))
// }

// async fn fetchGoogle(state: tauri::State<'_, State>) -> Result<String, reqwest::Error> {
//     state.client.get("https://www.google.com")
//         .send()
//         .await?
//         .text()
//         .await
// }


async fn some_async_function() {
  println!("Async")
}
