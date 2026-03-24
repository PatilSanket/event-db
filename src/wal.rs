/// Write Ahead Log (WAL) implementation for crash recovery.
///
/// Every write operation is durably logged to the WAL file *before* it reaches
/// the LSM tree or B-tree. On startup, unflushed WAL entries are replayed to
/// restore the database to a consistent state.
///
/// Record format (on disk):
/// ```text
/// [u32 payload_length][u32 crc32][payload bytes...]
/// ```
use crate::error::DatabaseError;
use crate::types::*;

use bincode;
use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

// ── WAL entry ──────────────────────────────────────────────────────────────

/// A single entry in the WAL file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    /// Monotonically increasing sequence number.
    pub sequence: SequenceNumber,
    /// The operation that was performed (Put / Delete).
    pub operation: Operation,
    /// Wall-clock timestamp (millis since UNIX epoch).
    pub timestamp: Timestamp,
}

// ── Header constants ───────────────────────────────────────────────────────

const HEADER_SIZE: usize = 8; // 4-byte length + 4-byte CRC

// ── WriteAheadLog ──────────────────────────────────────────────────────────

pub struct WriteAheadLog {
    /// Path to the WAL file on disk.
    path: PathBuf,
    /// Mutex-protected file handle for concurrent async appends.
    writer: Mutex<BufWriter<File>>,
    /// Whether to call `fsync` after every append.
    sync_on_write: bool,
}

impl WriteAheadLog {
    // ── Construction ───────────────────────────────────────────────────

    /// Open (or create) the WAL file at `path`.
    pub fn open(path: PathBuf, sync_on_write: bool) -> DatabaseResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            path,
            writer: Mutex::new(BufWriter::new(file)),
            sync_on_write,
        })
    }

    // ── Append ─────────────────────────────────────────────────────────

    /// Durably append an operation to the WAL.
    ///
    /// The record is length-prefixed and CRC-protected so that recovery can
    /// detect and skip incomplete/corrupt trailing records.
    pub async fn append(&self, operation: &Operation, sequence: u64) -> DatabaseResult<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let entry = WalEntry {
            sequence,
            operation: operation.clone(),
            timestamp,
        };

        let payload = bincode::serialize(&entry)
            .map_err(|e| DatabaseError::WalError(format!("serialization failed: {e}")))?;

        let crc = compute_crc(&payload);
        let length = payload.len() as u32;

        let mut writer = self.writer.lock().await;
        writer.write_all(&length.to_le_bytes())?;
        writer.write_all(&crc.to_le_bytes())?;
        writer.write_all(&payload)?;
        writer.flush()?;

        if self.sync_on_write {
            // Force the OS to flush its internal buffers to disk.
            writer.get_ref().sync_all()?;
        }

        Ok(())
    }

    // ── Recovery ───────────────────────────────────────────────────────

    /// Read the WAL file and return all valid entries.
    ///
    /// Records with a CRC mismatch or truncated records at the tail of the
    /// file are silently skipped (they indicate an incomplete write before a
    /// crash).
    pub fn recover(path: &Path) -> DatabaseResult<Vec<WalEntry>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(path)?;
        let file_len = file.metadata()?.len();
        if file_len == 0 {
            return Ok(Vec::new());
        }

        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut header_buf = [0u8; HEADER_SIZE];

        loop {
            // Try to read the 8-byte header; EOF here is not an error.
            match reader.read_exact(&mut header_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(DatabaseError::Io(e)),
            }

            let length = u32::from_le_bytes(header_buf[0..4].try_into().unwrap()) as usize;
            let expected_crc = u32::from_le_bytes(header_buf[4..8].try_into().unwrap());

            // Try to read the payload.
            let mut payload = vec![0u8; length];
            match reader.read_exact(&mut payload) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    // Truncated record at end of file — stop recovery.
                    break;
                }
                Err(e) => return Err(DatabaseError::Io(e)),
            }

            // Validate CRC.
            let actual_crc = compute_crc(&payload);
            if actual_crc != expected_crc {
                // Corrupt record — stop recovery here; all subsequent records
                // are considered unreliable.
                break;
            }

            // Deserialize.
            match bincode::deserialize::<WalEntry>(&payload) {
                Ok(entry) => entries.push(entry),
                Err(_) => break, // deserialization failure — stop.
            }
        }

        Ok(entries)
    }

    // ── Truncate ───────────────────────────────────────────────────────

    /// Truncate the WAL file (typically called after a successful checkpoint /
    /// memtable flush).
    pub async fn truncate(&self) -> DatabaseResult<()> {
        let mut writer = self.writer.lock().await;

        // Flush any buffered data, then truncate and re-open.
        writer.flush()?;
        drop(writer);

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)?;
        file.sync_all()?;

        // Re-open in append mode.
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        *self.writer.lock().await = BufWriter::new(file);
        Ok(())
    }

    // ── Sync ───────────────────────────────────────────────────────────

    /// Force-sync the WAL file to disk without truncating.
    pub async fn sync(&self) -> DatabaseResult<()> {
        let writer = self.writer.lock().await;
        writer.get_ref().sync_all()?;
        Ok(())
    }

    // ── Accessors ──────────────────────────────────────────────────────

    /// Return the path of the WAL file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn compute_crc(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_append_and_recover() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("test.wal");

        // Append two entries.
        {
            let wal = WriteAheadLog::open(wal_path.clone(), true).unwrap();
            wal.append(
                &Operation::Put {
                    key: b"k1".to_vec(),
                    value: b"v1".to_vec(),
                },
                1,
            )
            .await
            .unwrap();

            wal.append(&Operation::Delete { key: b"k2".to_vec() }, 2)
                .await
                .unwrap();
        }

        // Recover.
        let entries = WriteAheadLog::recover(&wal_path).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence, 1);
        assert_eq!(entries[1].sequence, 2);
        assert!(matches!(entries[0].operation, Operation::Put { .. }));
        assert!(matches!(entries[1].operation, Operation::Delete { .. }));
    }

    #[tokio::test]
    async fn test_truncate() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("test.wal");

        let wal = WriteAheadLog::open(wal_path.clone(), true).unwrap();
        wal.append(
            &Operation::Put {
                key: b"k".to_vec(),
                value: b"v".to_vec(),
            },
            1,
        )
        .await
        .unwrap();

        wal.truncate().await.unwrap();

        let entries = WriteAheadLog::recover(&wal_path).unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_recover_empty_file() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("empty.wal");
        fs::write(&wal_path, &[]).unwrap();

        let entries = WriteAheadLog::recover(&wal_path).unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_recover_nonexistent_file() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("nope.wal");

        let entries = WriteAheadLog::recover(&wal_path).unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_recover_truncated_record() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("trunc.wal");

        // Write a valid record then append garbage.
        {
            let wal = WriteAheadLog::open(wal_path.clone(), true).unwrap();
            wal.append(
                &Operation::Put {
                    key: b"k".to_vec(),
                    value: b"v".to_vec(),
                },
                1,
            )
            .await
            .unwrap();
        }

        // Append a partial header (simulate crash mid-write).
        {
            let mut f = OpenOptions::new().append(true).open(&wal_path).unwrap();
            f.write_all(&[0u8; 3]).unwrap(); // incomplete header
        }

        let entries = WriteAheadLog::recover(&wal_path).unwrap();
        assert_eq!(entries.len(), 1); // only the valid record
    }
}
