use bridge_parser::{
    fetch_indexed_files,
    parse_files,
    convert_to_assignments,
    read_local_files,
};
use bridge_parser::exporter::{
    Exporter, 
    PostgresExporter, 
    CsvExporter,
};
#[cfg(feature = "parquet_export")]
use bridge_parser::exporter::ParquetExporter;
use bridge_parser::collector::BridgeRawFile;
use bridge_parser::error::BridgeError;
use clap::Parser;
use tracing::{info, error};
use std::error::Error;
use std::path::{Path, PathBuf};
use dotenvy::dotenv;

/// Command-line options for bridge-parser
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// URL base for the Tor CollecTor index
    #[arg(long, default_value = "https://collector.torproject.org")]
    base: String,

    /// Directory path on the CollecTor index (e.g., recent/bridge-pool-assignments)
    #[arg(long, default_value = "recent/bridge-pool-assignments")]
    path: String,

    /// PostgreSQL connection string
   #[arg(long, env = "DB_PARAMS", default_value = "host=localhost user=postgres password=secret dbname=tor_metrics")]
   db: String,

    /// Clear the database tables before inserting
    #[arg(long, default_value_t = false)]
    clear: bool,

    ///Local fallback: load files from a local directory instead of fetching
    #[arg(long)]
    local_dir: Option<String>,

    ///Output format: postgres (default), csv, or parquet
    #[arg(long, default_value = "postgres")]
    format: String,

    ///Dry run: only parse, do not export to DB or file
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    ///Optional limit on the number of files parsed (for testing/debug)
    #[arg(long)]
    limit: Option<usize>,

    ///CSV export file path (used if --format=csv)
    #[arg(long, default_value = "output.csv")]
    csv_output: String,

    ///Parquet export file path (used if --format=parquet)
    #[arg(long, default_value = "output.parquet")]
    parquet_output: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing with env filter
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
        .try_init()
        .ok(); // Ignore if already initialized
        
    dotenv().ok();
    let opts = Options::parse();

    info!("Starting bridge parser");

    //  Step 1: Read files either from local or fetch from Tor CollecTor
    let mut content = if let Some(ref dir) = opts.local_dir {
        read_local_files(Path::new(dir))?
            .into_iter()
            .map(|bytes| -> Result<BridgeRawFile, BridgeError> {
                Ok(BridgeRawFile {
                    path: String::new(), // or appropriate path
                    timestamp: chrono::Utc::now().timestamp(), // or appropriate timestamp
                    content: String::from_utf8(bytes.clone())
                        .map_err(|e| BridgeError::Parse(e.to_string()))?,
                    raw: bytes,
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        tokio::runtime::Runtime::new()?.block_on(fetch_indexed_files(&opts.base, &opts.path))?
    };

    //  Step 2: If --limit N was passed, truncate file list for testing
    if let Some(max) = opts.limit {
        content.truncate(max);
        info!(" Truncated input to {} files due to --limit", max);
    }

    //  Step 3: Parse raw files and transform into bridge assignments
    let parsed = parse_files(content)?;
    let assignments = convert_to_assignments(parsed);

    //  Step 4: Only export if dry-run is NOT set
    if !opts.dry_run {
        match opts.format.as_str() {
            "postgres" => {
                //  PostgreSQL backend
                let exporter = PostgresExporter {
                    conn_str: opts.db.clone(),
                    truncate: opts.clear,
                };
                exporter.export(&assignments)?;
            }
            "csv" => {
                //  CSV backend: uses `--csv-output` path
                let exporter = CsvExporter {
                    output_path: PathBuf::from(opts.csv_output.clone()),
                };
                exporter.export(&assignments)?;
            }
            "parquet" => {
                #[cfg(feature = "parquet_export")]
                {
                    let exporter = ParquetExporter {
                        output_path: PathBuf::from(opts.parquet_output.clone()),
                    };
                    exporter.export(&assignments)?;
                }
                #[cfg(not(feature = "parquet_export"))]
                {
                    error!(" Parquet export support not enabled. Compile with --features parquet_export");
                    std::process::exit(1);
                }
            }
            other => {
                // Unknown backend
                error!("Unsupported format: '{}'. Use --format=postgres|csv|parquet", other);
                std::process::exit(1);
            }
        }
    } else {
        // Dry-run: skip export, useful for debugging parsing
        info!(" Dry run mode enabled – skipping export step");
    }

    info!(" Done");
    Ok(())
}
