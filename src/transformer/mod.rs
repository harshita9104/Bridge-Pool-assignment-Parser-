pub mod parser;

pub use parser::{parse_files, BridgeParsedAssignment, BridgeLineEntry};

/// Currently, this function just returns the input.
/// In future, you may transform the parsed assignment into another format here.
pub fn convert_to_assignments(input: Vec<BridgeParsedAssignment>) -> Vec<BridgeParsedAssignment> {
    input
}
