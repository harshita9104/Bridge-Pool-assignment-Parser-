#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_line_valid() {
        // Using format matching BridgeLineEntry structure:
        // fingerprint distribution_method [key=value pairs...]
        let valid_line = "1234567890ABCDEF1234567890ABCDEF12345678 vanilla transport=obfs4 ip=4 blocklist=none distributed=true state=functional";
        let result = transformer::parser::parse_line(valid_line);
        assert!(result.is_ok());
        
        if let Ok(entry) = result {
            assert_eq!(entry.fingerprint, "1234567890ABCDEF1234567890ABCDEF12345678");
            assert_eq!(entry.distribution_method, "vanilla");
            assert_eq!(entry.transport, Some("obfs4".to_string()));
        }
    }
}

pub mod collector;
pub mod transformer;
pub mod exporter;
pub mod helper;
pub mod error;

// Re-export commonly used functions
pub use collector::local::read_local_files;
pub use collector::fetch::fetch_indexed_files;
pub use transformer::parser::parse_files;
pub use transformer::convert_to_assignments;
