// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::arch::x86_64::__m128;
use reqwest::{Client, Error};

struct State{
  client: reqwest::Client
}

fn main() {
  tauri::Builder::default()
      .manage(State{
        client: reqwest::Client::new()
      })
      .invoke_handler(tauri::generate_handler![my_custom_command])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

// Declare the async function using String instead of &str, as &str is borrowed and thus unsupported
#[tauri::command]
async fn my_custom_command(state: tauri::State<'_, State>, value: String) -> Result<String,String> {
  // Call another async function and wait for it to finish
  let v = fetchGoogle(state).await;
    if v.is_err() {
        return Err(v.err().expect("No Error when Error").to_string());
    }

    let string = v.expect("Fehler ohne Error");
    Ok(string.replace("href=\"/", "href=\"https://www.google.com/").replace("src=\"/", "src=\"https://www.google.com/"))
}

async fn fetchGoogle(state: tauri::State<'_, State>) -> Result<String, reqwest::Error> {
    state.client.get("https://www.google.com")
        .send()
        .await?
        .text()
        .await
}


async fn some_async_function() {
  println!("Async")
}
