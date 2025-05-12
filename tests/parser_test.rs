//! Unit tests for parsing logic, digest trait, regex fingerprint check and version fallback

use bridge_parser::transformer::parser::*;
use bridge_parser::helper::{Digest, Sha256Digest};
use bridge_parser::error::BridgeError;

mod common;

#[test]
fn test_parse_line_valid() {
    common::setup();
    let line = "005fd4d7decbb250055b861579e6fdc79ad17bee email transport=obfs4 ip=4 blocklist=ru distributed=true state=functional bandwidth=accepted ratio=1.902";
    let result = parse_line(line);
    assert!(result.is_ok());
    
    if let Ok(entry) = result {
        assert_eq!(entry.fingerprint, "005fd4d7decbb250055b861579e6fdc79ad17bee");
        assert_eq!(entry.distribution_method, "email");
        assert_eq!(entry.transport, Some("obfs4".to_string()));
    }
}

#[test]
fn test_digest_trait_hash_entry_works() {
    common::setup();
    let test_content = "test content\nfor hashing";
    let hasher = Sha256Digest;
    let digest = hasher.hash_bytes(test_content.as_bytes());
    assert_eq!(digest.len(), 64);
}

#[test]
fn test_invalid_fingerprint_rejected_by_regex() {
    common::setup();
    // Invalid fingerprint format (not 40 hex chars)
    let invalid_line = "INVALID12345 vanilla transport=obfs4";
    let result = parse_line(invalid_line);
    assert!(matches!(result, Err(BridgeError::InvalidLine(_))));
}

#[test]
fn test_fallback_version_if_type_missing() {
    common::setup();
    let line_no_version = "1234567890ABCDEF1234567890ABCDEF12345678 vanilla";
    let result = parse_line(line_no_version);
    assert!(result.is_ok());
    
    if let Ok(entry) = result {
        assert_eq!(entry.distribution_method, "vanilla");
        assert_eq!(entry.transport, None);
    }
}

#[test]
fn test_parse_line_invalid() {
    common::setup();
    let line = "invalid_fingerprint email";
    let result = parse_line(line);
    assert!(matches!(result, Err(BridgeError::InvalidLine(_))));
}
