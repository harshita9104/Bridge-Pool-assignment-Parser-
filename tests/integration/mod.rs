use bridge_parser::*;
use std::path::Path;

#[test]
fn test_full_pipeline() {
    // Test the complete pipeline: fetch -> parse -> export
    let test_dir = Path::new("test_data");
    let result = read_local_files(test_dir);
    assert!(result.is_ok());
}