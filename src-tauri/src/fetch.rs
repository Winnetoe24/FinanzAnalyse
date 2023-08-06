use tauri::utils::config::parse::does_supported_file_name_exist;
use tauri::utils::debug_eprintln;
use tokio::fs;

fn get_cache_or_fetch(cache_pfad: String, url: String, api_key: String) -> Result<(bool, String), reqwest::Error> {
    if fs::try_exists(cache_pfad)? {
        fs::read_to_string(cache_pfad)
    }

    OK((false, "".to_string()))
}