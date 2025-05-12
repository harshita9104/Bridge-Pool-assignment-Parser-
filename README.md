
##  Project Goals

- Re-implement `metrics-lib` logic for parsing bridge assignments.
- Digest hashing using SHA-256 (file + per-line digests).
- Export data into structured formats like PostgreSQL and CSV.
- Enable modular, testable, production-grade Rust tooling for Tor network analysis.

##  Features

- Modular architecture: fetcher, parser, transformer, exporter
- Built with async I/O using tokio
- SHA-256 digest abstraction (trait-based)
- Offline support (--local-dir) for local testing
- PostgreSQL + CSV + Parquet export support
- Retry & backoff (via tokio-retry) for resilient network fetches
- CLI flags for format, limit, dry-run, etc.
- Unit tested & tracing-enabled logs
## Diagram
![Library Architecture](./diagram.png)

##  Quick Start

### 1. Installation

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Build project
cargo build --release
```

### 2. Basic Usage

```bash
# Run with default settings (PostgreSQL export)
cargo run

# Export to CSV
cargo run -- --format csv --csv-output bridges.csv

# Process local files
cargo run -- --local-dir ./test_data

# Dry run (parse only)
cargo run -- --dry-run
```

##  Quick Start 

### One-Click Setup Script

# Install Rust if needed
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
fi

# Setup PostgreSQL (Ubuntu/Debian)
if ! command -v psql &> /dev/null; then
    sudo apt update
    sudo apt install -y postgresql postgresql-contrib
fi

# Start PostgreSQL
```bash
sudo service postgresql start

# Setup Database
# Access PostgreSQL
sudo -u postgres psql

# In PostgreSQL console:
CREATE DATABASE tor_metrics;
ALTER USER postgres WITH PASSWORD 'your_password';
```
# Create .env file

DB_PARAMS="host=localhost user=postgres password=your_password dbname=tor_metrics"

# Build and run
```bash
cargo build --release
```

### Quick Setup Steps

1. **Database Setup**
   ```bash
   # Start PostgreSQL
   sudo service postgresql start
   
   # Create database and set password
   sudo -u postgres psql
   CREATE DATABASE tor_metrics;
   ALTER USER postgres WITH PASSWORD 'your_password';
   \q
   ```

2. **Environment Configuration**
   ```bash
   # Create .env file in project root
   echo 'DB_PARAMS="host=localhost user=postgres password=your_password dbname=tor_metrics"' > .env
   ```

3. **Build and Run**
   ```bash
   cargo run --release
   ```

### Common Issues and Solutions

#### Database Connection
If you see: `Error: Database("PostgreSQL connection failed: db error: FATAL: password authentication failed for user \"postgres\"")`

Solutions:
1. **Check .env file exists** in project root with correct credentials
2. **Verify PostgreSQL is running:**
   ```bash
   sudo service postgresql status
   ```
3. **Test connection:**
   ```bash
   psql -U postgres -h localhost -d tor_metrics
   ```

#### Alternative Connection Methods

 **Use Command Line Parameters**
   ```bash
   cargo run -- --db "host=localhost user=postgres password=your_password dbname=tor_metrics""
   ```

### Running Without Database (CSV Mode)
For quick testing without database setup:
```bash
cargo run -- --format csv --csv-output output.csv
```



## Implemented Features

| Feature                     | Status | Location                                    |
|----------------------------|--------|---------------------------------------------|
| Retry Logic               | done     | `fetch.rs` via tokio_retry                  |
| Fingerprint Validation    | done    | `parser.rs` with Regex patterns             |
| Digest Abstraction       | done     | `digest.rs`, Sha256Digest implementation    |
| Version-Aware Parsing    | done    | `parser.rs`, fallback to 1.0                |
| Offline Mode             | done     | `main.rs`, `local.rs`                       |
| PostgreSQL Export        | done     | `pg.rs`, using tokio-postgres               |
| CSV Export              | done     | `csv.rs`, using csv crate                   |
| Parquet Export          | done     | `parquet.rs` (optional feature)             |
| CLI Arguments           | done     | `main.rs` using clap                        |
| Structured Logging      | done     | Using tracing crate                         |

## 🛠 Advanced Usage

### PostgreSQL Export

```bash
# Method 1: Using .env file
echo 'DB_PARAMS="host=localhost user=postgres password=your_password dbname=tor_metrics"' > .env
cargo run -- --format postgres --clear

# Method 2: Using command line parameter (preferred)
cargo run -- --db "host=localhost user=postgres password=your_passworddbname=tor_metrics" --format postgres --clear
```

### CSV Export

```bash
# Basic CSV export (no database needed)
cargo run -- --format csv --csv-output data.csv

# With processing limit
cargo run -- --format csv --csv-output sample.csv --limit 10
```

### Parquet Export 

```bash
# Enable feature and export (no database needed)
cargo run --features parquet_export -- --format parquet --parquet-output data.parquet
```

### Local File Processing

```bash
# Process local files with PostgreSQL
cargo run -- --local-dir ./test_data --db "host=localhost user=postgres password=your_password dbname=tor_metrics"

# Process local files with CSV (recommended for testing)
cargo run -- --local-dir ./test_data --format csv --csv-output bridges.csv
```

### Testing Different Export Formats

```bash
# PostgreSQL with explicit connection
 cargo run -- --db "host=localhost user=postgres password=your_password dbname
=tor_metrics"

# CSV with limit
cargo run -- --format csv --csv-output output.csv --limit 100

```

### Testing & Development

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_parse_line_valid

# Run with debug logging
RUST_LOG=debug cargo run
```

##  Detailed Feature Reference

### 1. Core Bridge Assignment Parsing
**Location**: `src/transformer/parser.rs`
- Version-aware parsing with fallback support
- Regex-based fingerprint validation
- Structured data transformation

```bash
# Test parser functionality
cargo test test_parse_line_valid
cargo test --package bridge-parser --test parser_test
```

### 2. Multi-Format Export System
**Location**: `src/exporter/`
- PostgreSQL export (`pg.rs`)
- CSV export (`csv.rs`)
- Parquet export (optional feature)
Note: Before running these commands:
1. Ensure PostgreSQL is running: `sudo service postgresql status`
2. Verify database exists: `createdb tor_metrics` (if needed)
3. Set correct password in the connection string
```bash
# Test PostgreSQL export
cargo run -- --format postgres --db "host=localhost user=postgres password=your_password dbname=tor_metrics"

# Test CSV export
cargo run -- --format csv --csv-output test.csv

# Test Parquet export (requires feature flag)
cargo run --features parquet_export -- --format parquet --parquet-output test.parquet
```

### 3. SHA-256 Digest System
**Location**: `src/helper/digest.rs`
- File-level cryptographic hashes
- Per-line entry digests
- Trait-based abstraction

```bash
# Run digest-specific tests
cargo test test_digest
# Run all digest-related tests
cargo test digest
```

### 4. Resilient Network Fetching
**Location**: `src/collector/fetch.rs`
- Exponential backoff retry logic
- HTTPS support via reqwest
- Compression handling (gzip, xz)

```bash
# Test network fetching with debug logs (CSV output)
RUST_LOG=debug cargo run -- --base https://collector.torproject.org --format csv --csv-output output.csv

# Test network fetching with PostgreSQL
RUST_LOG=debug cargo run -- --base https://collector.torproject.org --db "host=localhost user=postgres password=your_password dbname=tor_metrics"

# Test retry logic
cargo test --test parser_test test_fetch
```

### 5. Local Development Mode
**Location**: `src/collector/local.rs`
- Offline development support
- Local file processing

```bash
# Test local file processing with CSV output (recommended for testing)
cargo run -- --local-dir ./test_data --format csv --csv-output local_output.csv

# Test local file processing with PostgreSQL
cargo run -- --local-dir ./test_data --format postgres --db "host=localhost user=postgres password=your_password dbname=tor_metrics"
```

### 6. Structured Logging
**Implementation**: Throughout codebase
- Hierarchical logging via tracing
- Environment-aware log levels

```bash
# Run with debug logs (CSV format to avoid database requirements)
RUST_LOG=debug cargo run -- --format csv --csv-output debug.csv

# Run with info logs (CSV format)
RUST_LOG=info cargo run -- --format csv --csv-output info.csv

# Run with trace logs and dry-run (no output needed)
RUST_LOG=trace cargo run -- --dry-run

# Run with debug logs and PostgreSQL (if database is configured)
RUST_LOG=debug cargo run -- --format postgres --db "host=localhost user=postgres password=your_password dbname=tor_metrics"

```

### 7. Database Schema Management
**Location**: `src/exporter/pg.rs`
- Automatic table creation
- Optimized indexes
- Referential integrity
Note: Before running these commands:
1. Ensure PostgreSQL is running: `sudo service postgresql status`
2. Verify database exists: `createdb tor_metrics` (if needed)
3. Set correct password in the connection string
```bash
# View schema and initialize database
cargo run -- --format postgres --clear --db "host=localhost user=postgres password=abcd12345 dbname=tor_metrics" --local-dir ./test_data
```

### 8. Error Handling System
**Location**: `src/error.rs`
- Custom error types
- Structured error reporting
- Error chain tracking

```bash
# Run error handling unit tests
cargo test --package bridge-parser --lib error::tests

# Test with debug logging to see error handling in action
RUST_LOG=debug cargo run -- --format csv --csv-output test.csv --local-dir ./test_data
```

### 9. Unit Testing Framework
**Location**: `tests/`
- Comprehensive test coverage
- Integration tests
- Parser validation

```bash
# Run all tests
cargo test


```

### 10. Feature Flags
**Location**: `Cargo.toml`
- Optional Parquet support
- Conditional compilation

```bash
# Build with optional features
cargo build --features parquet_export

# Run with features
cargo run --features parquet_export -- --format parquet
```

### 11. Configuration Management
**Location**: Uses dotenvy
- Environment variable support
- Default configurations


##  Digest Strategy

### File Digest
- SHA-256 hash of complete file content
- Used as unique identifier in database

### Entry Digest
- SHA-256(line_content + file_digest)
- Ensures global uniqueness across files

##  Database Schema

### bridge_file Table
```sql
CREATE TABLE bridge_file (
    sha TEXT PRIMARY KEY,
    header TEXT NOT NULL,
    published TIMESTAMP NOT NULL
);
```

### bridge_entry Table
```sql
CREATE TABLE bridge_entry (
    sha TEXT PRIMARY KEY,
    fingerprint TEXT NOT NULL,
    method TEXT NOT NULL,
    file_sha TEXT REFERENCES bridge_file(sha),
    transport TEXT,
    ip TEXT,
    block TEXT,
    distributed BOOLEAN,
    state TEXT,
    bandwidth TEXT,
    ratio REAL,
    published TIMESTAMP NOT NULL
);
```

##  Error Handling

Comprehensive error handling via `BridgeError` enum:
- Network errors (Fetch)
- Parse errors
- Database errors
- Export errors
- Validation errors

##  Acknowledgments

- Tor Project's CollecTor service
- Rust community and my mentors 
