use std::fs;
use std::path::Path;
use crate::error::BridgeError;

pub fn read_local_files(path: &Path) -> Result<Vec<Vec<u8>>, BridgeError> {
    let entries = fs::read_dir(path)?;  // Now works with From implementation
    let mut files = Vec::new();

    for entry in entries {
        let entry = entry?;  // Now works with From implementation
        if entry.file_type()?.is_file() {  // Now works with From implementation
            let content = fs::read(entry.path())?;  // Now works with From implementation
            files.push(content);
        }
    }

    Ok(files)
}
