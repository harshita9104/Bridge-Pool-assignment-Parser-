use crate::transformer::{BridgeParsedAssignment, BridgeLineEntry};
use crate::error::BridgeError;
use chrono::{DateTime, NaiveDateTime, Utc};
use tokio_postgres::{NoTls, Transaction};
use crate::exporter::Exporter;

/// Write parsed bridge assignments into PostgreSQL.
pub async fn write_to_postgres(
    items: Vec<BridgeParsedAssignment>,
    conn_str: &str,
    truncate: bool,
) -> Result<(), BridgeError> {
    let (mut client, connection) = tokio_postgres::connect(conn_str, NoTls)
        .await
        .map_err(|e| BridgeError::Database(format!("PostgreSQL connection failed: {}", e)))?;

    // Spawn a background task to monitor connection errors.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("PostgreSQL async error: {e}");
        }
    });

    let tx = client.transaction()
        .await
        .map_err(|e| BridgeError::Database(format!("Begin transaction failed: {}", e)))?;

    prepare_schema(&tx).await?;

    if truncate {
        tx.execute("TRUNCATE TABLE bridge_entry, bridge_file", &[])
            .await
            .map_err(|e| BridgeError::Database(format!("Failed to truncate tables: {}", e)))?;
    }

    for file in items {
        insert_file(&tx, &file).await?;
        insert_lines(&tx, &file.file_sha, &file.lines, file.published).await?;
    }

    tx.commit()
        .await
        .map_err(|e| BridgeError::Database(format!("Commit failed: {}", e)))?;

    Ok(())
}

/// Ensure both bridge_file and bridge_entry tables exist.
async fn prepare_schema(tx: &Transaction<'_>) -> Result<(), BridgeError> {
    tx.execute(
        "CREATE TABLE IF NOT EXISTS bridge_file (
            sha TEXT PRIMARY KEY,
            header TEXT NOT NULL,
            published TIMESTAMP NOT NULL
        )",
        &[],
    )
    .await
    .map_err(|e| BridgeError::Database(format!("Creating bridge_file failed: {}", e)))?;

    tx.execute(
        "CREATE TABLE IF NOT EXISTS bridge_entry (
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
        )",
        &[],
    )
    .await
    .map_err(|e| BridgeError::Database(format!("Creating bridge_entry failed: {}", e)))?;

    Ok(())
}

/// Insert one bridge_file row
async fn insert_file(tx: &Transaction<'_>, file: &BridgeParsedAssignment) -> Result<(), BridgeError> {
    let published = to_naive_utc(file.published)?;
    tx.execute(
        "INSERT INTO bridge_file (sha, header, published)
         VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        &[&file.file_sha, &file.header, &published],
    )
    .await
    .map_err(|e| BridgeError::Database(format!("Insert into bridge_file failed: {}", e)))?;

    Ok(())
}

/// Insert multiple bridge_entry rows
async fn insert_lines(
    tx: &Transaction<'_>,
    file_sha: &str,
    lines: &[BridgeLineEntry],
    millis: i64,
) -> Result<(), BridgeError> {
    let published = to_naive_utc(millis)?;

    for entry in lines {
        let method = entry.distribution_method.clone();
        let transport = entry.transport.clone();
        let ip = entry.ip.clone();
        let block = entry.blocklist.clone();
        let distributed = entry.distributed;
        let state = entry.state.clone();
        let bandwidth = entry.bandwidth.clone();
        let ratio = entry.ratio;

        tx.execute(
            "INSERT INTO bridge_entry (
                sha, fingerprint, method, file_sha,
                transport, ip, block, distributed,
                state, bandwidth, ratio, published
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12
            ) ON CONFLICT DO NOTHING",
            &[
                &entry.sha,
                &entry.fingerprint,
                &method,
                &file_sha,
                &transport,
                &ip,
                &block,
                &distributed,
                &state,
                &bandwidth,
                &ratio,
                &published,
            ],
        )
        .await
        .map_err(|e| BridgeError::Database(format!("Insert into bridge_entry failed: {}", e)))?;
    }

    Ok(())
}

/// Convert i64 timestamp in millis to UTC NaiveDateTime.
fn to_naive_utc(ms: i64) -> Result<NaiveDateTime, BridgeError> {
    // Convert milliseconds to DateTime<Utc>
    let utc: DateTime<Utc> = {
        let secs = ms / 1000;
        let nsecs = ((ms % 1000) * 1_000_000) as u32;
        DateTime::<Utc>::from_timestamp(secs, nsecs)
            .ok_or_else(|| BridgeError::Export("Invalid timestamp conversion".into()))?
    };
    Ok(utc.naive_utc())
}

/// Struct wrapping config info for PostgreSQL export.
pub struct PostgresExporter {
    pub conn_str: String,
    pub truncate: bool,
}

/// Implements `Exporter` trait for PostgreSQL backend.
impl Exporter for PostgresExporter {
    fn export(&self, data: &[BridgeParsedAssignment]) -> Result<(), BridgeError> {
        let items = data.to_vec();
        let conn = self.conn_str.clone();
        let truncate = self.truncate;

        tokio::runtime::Runtime::new()
            .map_err(|e| BridgeError::Database(format!("Tokio runtime init failed: {}", e)))?
            .block_on(async move {
                write_to_postgres(items, &conn, truncate).await
            })
    }
}
