use chrono::NaiveDateTime;
use serde_json::Value;
use tokio_retry::Retry;
use tokio_retry::strategy::ExponentialBackoff;
use crate::error::BridgeError;
use reqwest::ClientBuilder;
use std::time::Duration;

#[derive(Debug)]
pub struct BridgeRawFile {
    pub path: String,
    pub content: String,
    pub raw: Vec<u8>,
    pub timestamp: i64,
}

pub async fn fetch_indexed_files(base_url: &str, folder: &str) -> Result<Vec<BridgeRawFile>, BridgeError> {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| BridgeError::Fetch(e.to_string()))?;

    let full_url = format!("{}/index/index.json", base_url.trim_end_matches('/'));

    let response = client.get(&full_url)
        .send()
        .await
        .map_err(|e| BridgeError::Fetch(e.to_string()))?;

    if !response.status().is_success() {
        return Err(BridgeError::HttpError(format!("HTTP error: {}", response.status())));
    }

    let index: Value = response.json().await
        .map_err(|e| BridgeError::Parse(format!("JSON parse failed: {}", e)))?;

    let items = extract_latest_entries(&index, folder, 10)?;

    let mut downloads = Vec::new();
    for (relative_path, timestamp) in items {
        let file_url = format!("{}/{}", base_url.trim_end_matches('/'), relative_path);

        // Retry network call if it fails
        let strategy = ExponentialBackoff::from_millis(100).take(5);
        let res = Retry::spawn(strategy, || reqwest::get(&file_url))
            .await
            .map_err(|e| BridgeError::Fetch(e.to_string()))?;

        let txt = res.text().await
            .map_err(|e| BridgeError::Fetch(e.to_string()))?;
        let raw = txt.as_bytes().to_vec();

        downloads.push(BridgeRawFile {
            path: relative_path,
            content: txt,
            raw,
            timestamp,
        });
    }

    Ok(downloads)
}

/// Traverse index.json to extract latest file paths for the target directory.
fn extract_latest_entries(index: &Value, target_dir: &str, limit: usize) -> Result<Vec<(String, i64)>, BridgeError> {
    let mut files = Vec::new();

    let segments: Vec<&str> = target_dir.trim_matches('/').split('/').collect();
    let mut node = &index["directories"];

    for (i, part) in segments.iter().enumerate() {
        let array = node.as_array()
            .ok_or_else(|| BridgeError::Parse("Expected array while traversing index.json".into()))?;

        let entry = array.iter()
            .find(|n| n["path"] == *part)
            .ok_or_else(|| BridgeError::Parse(format!("Directory '{}' not found", part)))?;

        if i == segments.len() - 1 {
            node = entry;
            break;
        }
        node = &entry["directories"];
    }

    let files_arr = node["files"].as_array()
        .ok_or_else(|| BridgeError::Parse("Final segment missing 'files' array".into()))?;

    for file in files_arr {
        let path = file["path"].as_str().ok_or_else(|| BridgeError::Parse("file missing 'path'".into()))?;

        let timestamp_str = file["last_modified"].as_str().unwrap_or("1970-01-01 00:00");
        let naive = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M")
            .unwrap_or_else(|_| chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap().naive_utc());

        let millis = naive.and_utc().timestamp_millis();
        let full_path = format!("{}/{}", target_dir.trim_end_matches('/'), path);

        files.push((full_path, millis));
    }

    files.sort_by(|a, b| b.1.cmp(&a.1)); // newest first
    Ok(files.into_iter().take(limit).collect())
}

pub async fn fetch_bridge_data(url: &str) -> Result<Vec<u8>, BridgeError> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| BridgeError::Fetch(e.to_string()))?;

    if !response.status().is_success() {
        return Err(BridgeError::HttpError(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| BridgeError::Fetch(e.to_string()))
}
