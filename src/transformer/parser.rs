use crate::collector::BridgeRawFile;
use crate::helper::{Sha256Digest, Digest};
use crate::error::BridgeError;

use chrono::NaiveDateTime;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref FINGERPRINT_REGEX: Regex = Regex::new(r"^[a-fA-F0-9]{40}$")
        .expect("Invalid fingerprint regex pattern");
}

/// Represents a full parsed bridge assignment file (with header, SHA, and entries).
#[derive(Debug, Clone)]  //  Add Clone here
pub struct BridgeParsedAssignment {
    pub file_sha: String,
    pub published: i64,
    pub header: String,
    pub lines: Vec<BridgeLineEntry>,
}

/// Represents an individual line in the bridge assignment (parsed into fields).
#[derive(Debug, Clone)]  // Add Clone derive
pub struct BridgeLineEntry {
    pub sha: String,
    pub fingerprint: String,
    pub distribution_method: String,
    pub transport: Option<String>,
    pub ip: Option<String>,
    pub blocklist: Option<String>,
    pub distributed: Option<bool>,
    pub state: Option<String>,
    pub bandwidth: Option<String>,
    pub ratio: Option<f32>,
}

/// Parse the list of bridge files into structured assignments.
/// This performs fingerprint validation and computes SHA digests.
pub fn parse_files(raw_files: Vec<BridgeRawFile>) -> Result<Vec<BridgeParsedAssignment>, BridgeError> {
    let mut parsed = Vec::new();

    for BridgeRawFile { content, raw, .. } in raw_files {
        let lines: Vec<&str> = content.lines().collect();
        
        let first = lines
            .iter()
            .find(|line| line.starts_with("bridge-pool-assignment"))
            .ok_or_else(|| BridgeError::Parse("missing header line".into()))?;

        let time = extract_time(first)?;

        let hasher = Sha256Digest;
        let sha_file = hasher.hash_bytes(&raw);

        let mut entries = Vec::new();
        for line in lines.iter().skip(1) {
            match parse_line(line) {
                Ok(mut entry) => {
                    if FINGERPRINT_REGEX.is_match(&entry.fingerprint) {
                        entry.sha = hasher.hash_entry(line.as_bytes(), &sha_file);
                        entries.push(entry);
                    }
                }
                Err(_) => continue, // Skip invalid lines
            }
        }

        parsed.push(BridgeParsedAssignment {
            file_sha: sha_file,
            published: time,
            header: first.to_string(),
            lines: entries,
        });
    }

    Ok(parsed)
}

/// Extract timestamp in milliseconds from the header line.
fn extract_time(line: &str) -> Result<i64, BridgeError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(BridgeError::Parse("invalid header timestamp format".into()));
    }

    let dt = NaiveDateTime::parse_from_str(&format!("{} {}", parts[1], parts[2]), "%Y-%m-%d %H:%M:%S")
        .map_err(|e| BridgeError::Parse(format!("Invalid timestamp: {}", e)))?;

    Ok(dt.and_utc().timestamp_millis())
}

/// Parse a single line into a BridgeLineEntry
pub fn parse_line(line: &str) -> Result<BridgeLineEntry, BridgeError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return Err(BridgeError::InvalidLine("Insufficient parts".into()));
    }

    let fingerprint = parts[0].to_string();
    if fingerprint.len() != 40 {
        return Err(BridgeError::InvalidLine("Invalid fingerprint length".into()));
    }

    let distribution_method = parts[1].to_string();
    
    let mut entry = BridgeLineEntry {
        sha: String::new(),
        fingerprint,
        distribution_method,
        transport: None,
        ip: None,
        blocklist: None,
        distributed: None,
        state: None,
        bandwidth: None,
        ratio: None,
    };

    // Parse additional parameters
    for part in parts.iter().skip(2) {
        let kv: Vec<&str> = part.split('=').collect();
        if kv.len() != 2 { continue; }
        
        match kv[0] {
            "transport" => entry.transport = Some(kv[1].to_string()),
            "ip" => entry.ip = Some(kv[1].to_string()),
            "blocklist" => entry.blocklist = Some(kv[1].to_string()),
            "distributed" => entry.distributed = Some(kv[1] == "true"),
            "state" => entry.state = Some(kv[1].to_string()),
            "bandwidth" => entry.bandwidth = Some(kv[1].to_string()),
            "ratio" => entry.ratio = kv[1].parse().ok(),
            _ => continue,
        }
    }

    Ok(entry)
}
