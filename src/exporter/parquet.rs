#[cfg(feature = "parquet_export")]
use std::fs::File;
#[cfg(feature = "parquet_export")]
use std::path::PathBuf;
#[cfg(feature = "parquet_export")]
use std::sync::Arc;
#[cfg(feature = "parquet_export")]
use arrow::array::{Array, StringArray, BooleanArray, Float64Array};
#[cfg(feature = "parquet_export")]
use arrow::record_batch::RecordBatch;
#[cfg(feature = "parquet_export")]
use arrow::datatypes::{Schema, Field, DataType};
#[cfg(feature = "parquet_export")]
use parquet::arrow::ArrowWriter;
use crate::error::BridgeError;
use crate::transformer::parser::BridgeParsedAssignment;
use crate::exporter::Exporter;
use tracing::{info, warn};

#[cfg(feature = "parquet_export")]
pub struct ParquetExporter {
    pub output_path: PathBuf,
}

#[cfg(feature = "parquet_export")]
impl Exporter for ParquetExporter {
    fn export(&self, data: &[BridgeParsedAssignment]) -> Result<(), BridgeError> {
        info!("Exporting {} assignments to Parquet format...", data.len());
        
        if data.is_empty() {
            warn!("⚠️ No data to export");
            return Ok(());
        }

        let schema = Arc::new(Schema::new(vec![
            Field::new("file_sha", DataType::Utf8, false),
            Field::new("published_timestamp", DataType::Int64, false),
            Field::new("entry_sha", DataType::Utf8, false),
            Field::new("fingerprint", DataType::Utf8, false),
            Field::new("distribution_method", DataType::Utf8, false),
            Field::new("transport", DataType::Utf8, true),
            Field::new("ip", DataType::Utf8, true),
            Field::new("blocklist", DataType::Utf8, true),
            Field::new("distributed", DataType::Boolean, true),
            Field::new("state", DataType::Utf8, true),
            Field::new("bandwidth", DataType::Utf8, true),
            Field::new("ratio", DataType::Float64, true),
        ]));

        let mut file_shas = Vec::new();
        let mut timestamps = Vec::new();
        let mut entry_shas = Vec::new();
        let mut fingerprints = Vec::new();
        let mut methods = Vec::new();
        let mut transports = Vec::new();
        let mut ips = Vec::new();
        let mut blocklists = Vec::new();
        let mut distributed = Vec::new();
        let mut states = Vec::new();
        let mut bandwidths = Vec::new();
        let mut ratios = Vec::new();

        let total_entries = data.iter().map(|a| a.lines.len()).sum::<usize>();
        info!(" Processing {} total entries...", total_entries);

        for assignment in data {
            for line in &assignment.lines {
                file_shas.push(assignment.file_sha.as_str());
                timestamps.push(Some(assignment.published));
                entry_shas.push(line.sha.as_str());
                fingerprints.push(line.fingerprint.as_str());
                methods.push(line.distribution_method.as_str());
                transports.push(line.transport.as_deref());
                ips.push(line.ip.as_deref());
                blocklists.push(line.blocklist.as_deref());
                distributed.push(line.distributed);
                states.push(line.state.as_deref());
                bandwidths.push(line.bandwidth.as_deref());
                ratios.push(line.ratio.map(|r| r as f64));
            }
        }

        let arrays: Vec<Arc<dyn Array>> = vec![
            Arc::new(StringArray::from(file_shas)),
            Arc::new(arrow::array::Int64Array::from(timestamps)),
            Arc::new(StringArray::from(entry_shas)),
            Arc::new(StringArray::from(fingerprints)),
            Arc::new(StringArray::from(methods)),
            Arc::new(StringArray::from(transports)),
            Arc::new(StringArray::from(ips)),
            Arc::new(StringArray::from(blocklists)),
            Arc::new(BooleanArray::from(distributed)),
            Arc::new(StringArray::from(states)),
            Arc::new(StringArray::from(bandwidths)),
            Arc::new(Float64Array::from(ratios)),
        ];

        info!(" Creating Parquet file: {}", self.output_path.display());

        let batch = RecordBatch::try_new(schema.clone(), arrays)
            .map_err(|e| BridgeError::Export(format!("Failed to create record batch: {}", e)))?;

        let file = File::create(&self.output_path)
            .map_err(|e| BridgeError::Export(format!("Failed to create file: {}", e)))?;

        let mut writer = ArrowWriter::try_new(file, schema, None)
            .map_err(|e| BridgeError::Export(format!("Failed to create writer: {}", e)))?;

        writer.write(&batch)
            .map_err(|e| BridgeError::Export(format!("Failed to write batch: {}", e)))?;

        writer.close()
            .map_err(|e| BridgeError::Export(format!("Failed to close writer: {}", e)))?;

        info!("✅ Successfully exported to Parquet format");
        Ok(())
    }
}

#[cfg(not(feature = "parquet_export"))]
pub struct ParquetExporter;

#[cfg(not(feature = "parquet_export"))]
impl Exporter for ParquetExporter {
    fn export(&self, _data: &[BridgeParsedAssignment]) -> Result<(), BridgeError> {
        Err(BridgeError::Export("Parquet export support not enabled. Compile with --features parquet_export".into()))
    }
}
