// Declare submodules in this collector directory
pub mod fetch;
pub mod local;

// Re-export key types and functions so they can be used in main.rs
pub use fetch::{fetch_indexed_files, BridgeRawFile};
pub use local::read_local_files;
