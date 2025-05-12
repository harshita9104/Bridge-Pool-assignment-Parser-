use std::fs::File;
use std::path::PathBuf;
use csv;  // Changed from csv::Writer since we use it through csv::Writer::from_writer
use crate::error::BridgeError;
use crate::transformer::parser::BridgeParsedAssignment;
use crate::exporter::Exporter;

pub struct CsvExporter {
    pub output_path: PathBuf,
}

impl Exporter for CsvExporter {
    fn export(&self, data: &[BridgeParsedAssignment]) -> Result<(), BridgeError> {
        let file = File::create(&self.output_path)?;  // Now works with From implementation
        let mut writer = csv::Writer::from_writer(file);

        // Write header
        writer.write_record(&[
            "file_sha",
            "published_timestamp",
            // ... other fields ...
        ])?;  // Now works with From implementation

        for assignment in data {
            for line in &assignment.lines {
                writer.write_record(&[
                    &line.sha,
                    &line.fingerprint,
                    &line.distribution_method,
                    line.transport.as_deref().unwrap_or(""),
                    line.ip.as_deref().unwrap_or(""),
                    line.blocklist.as_deref().unwrap_or(""),
                    &line.distributed.map_or("".to_string(), |b| b.to_string()),
                    line.state.as_deref().unwrap_or(""),
                    line.bandwidth.as_deref().unwrap_or(""),
                    &line.ratio.map_or("".to_string(), |r| r.to_string()),
                ])?;
            }
        }

        writer.flush()?;  // Now works with From implementation
        Ok(())
    }
}
