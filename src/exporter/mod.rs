use crate::transformer::BridgeParsedAssignment;
use crate::error::BridgeError;

mod pg;
mod csv;
#[cfg(feature = "parquet_export")]
mod parquet;

pub use pg::PostgresExporter;
pub use csv::CsvExporter;
#[cfg(feature = "parquet_export")]
pub use parquet::ParquetExporter;

pub trait Exporter {
    fn export(&self, data: &[BridgeParsedAssignment]) -> Result<(), BridgeError>;
}
