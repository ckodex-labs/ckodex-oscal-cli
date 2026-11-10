use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::error::{AppError, Result, io_error};

/// Maximum permitted size (64 MiB) when reading OSCAL, SBOM, UTCP, or CAS files into memory.
pub const MAX_OSCAL_FILE_BYTES: u64 = 64 * 1024 * 1024;

static ATOMIC_TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Read a regular file into memory with strict pre-check and TOCTOU byte-cap enforcement.
///
/// Rejects directories, device nodes (`/dev/zero`), FIFOs/pipes, sockets, and files
/// exceeding `max_bytes`.
pub fn read_bounded_regular_file(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    let file = File::open(path).map_err(|e| io_error(path, e))?;
    let meta = file.metadata().map_err(|e| io_error(path, e))?;

    if !meta.file_type().is_file() {
        return Err(AppError::InvalidArgument(format!(
            "Refusing to read non-regular file: {}",
            path.display()
        )));
    }

    if meta.len() > max_bytes {
        return Err(AppError::InvalidArgument(format!(
            "File '{}' ({} bytes) exceeds maximum permitted size of {} bytes",
            path.display(),
            meta.len(),
            max_bytes
        )));
    }

    let initial_cap = usize::try_from(meta.len().min(max_bytes))
        .unwrap_or(0)
        .min(8 * 1024 * 1024);
    let mut buf = Vec::with_capacity(initial_cap);
    let read_limit = max_bytes.saturating_add(1);
    file.take(read_limit)
        .read_to_end(&mut buf)
        .map_err(|e| io_error(path, e))?;

    if (buf.len() as u64) > max_bytes {
        return Err(AppError::InvalidArgument(format!(
            "File '{}' grew past maximum permitted size of {} bytes during read",
            path.display(),
            max_bytes
        )));
    }

    Ok(buf)
}

/// Read a UTF-8 regular file into memory with strict pre-check and TOCTOU byte-cap enforcement.
pub fn read_bounded_utf8_file(path: &Path, max_bytes: u64) -> Result<String> {
    let bytes = read_bounded_regular_file(path, max_bytes)?;
    String::from_utf8(bytes).map_err(|e| {
        io_error(
            path,
            std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        )
    })
}

/// Write `data` crash-safely to `path` via a unique sibling temporary file, `sync_all()`,
/// atomic `rename()`, and best-effort parent directory `sync_all()`.
pub fn atomic_write_bytes(path: &Path, data: &[u8]) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty())
        .ok_or_else(|| {
            AppError::InvalidArgument(format!(
                "Cannot atomically write to path without file name: {}",
                path.display()
            ))
        })?;

    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    fs::create_dir_all(parent).map_err(|e| io_error(parent, e))?;

    let pid = std::process::id();
    let seq = ATOMIC_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_path = parent.join(format!(".{file_name}.mizan-tmp.{pid}.{seq}"));

    let write_res = (|| -> std::io::Result<()> {
        let mut tmp_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)?;
        tmp_file.write_all(data)?;
        tmp_file.sync_all()?;
        Ok(())
    })();

    if let Err(e) = write_res {
        let _ = fs::remove_file(&tmp_path);
        return Err(io_error(&tmp_path, e));
    }

    if let Err(e) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(io_error(path, e));
    }

    if let Ok(dir_handle) = File::open(parent) {
        let _ = dir_handle.sync_all();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_write_bytes_and_bounded_read_roundtrip() {
        let tmp_dir = std::env::temp_dir().join(format!(
            "mizan-io-safe-test-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let nested_file = tmp_dir.join("sub/dir/artifact.json");

        atomic_write_bytes(&nested_file, b"{\"v\":1}")
            .expect("initial atomic write should succeed");
        assert_eq!(
            read_bounded_utf8_file(&nested_file, 1024).expect("bounded utf8 read should succeed"),
            "{\"v\":1}"
        );

        // Overwrite existing file atomically
        atomic_write_bytes(&nested_file, b"{\"v\":2,\"updated\":true}")
            .expect("overwrite atomic write should succeed");
        assert_eq!(
            read_bounded_utf8_file(&nested_file, 1024).expect("bounded utf8 read should succeed"),
            "{\"v\":2,\"updated\":true}"
        );

        // Ensure no leftover .mizan-tmp files remain in parent directory
        let leftovers: Vec<_> = fs::read_dir(nested_file.parent().unwrap())
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains(".mizan-tmp."))
            .collect();
        assert!(leftovers.is_empty(), "temporary files must be cleaned up");

        let _ = fs::remove_dir_all(tmp_dir);
    }

    #[test]
    fn test_read_bounded_regular_file_rejects_oversized_and_non_regular() {
        let tmp_dir = std::env::temp_dir().join(format!(
            "mizan-io-bounds-test-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&tmp_dir).unwrap();

        // 1. Reject directory
        let dir_err = read_bounded_regular_file(&tmp_dir, 1024)
            .expect_err("reading a directory must fail closed");
        assert!(
            dir_err.to_string().contains("non-regular")
                || matches!(dir_err, AppError::Io { .. } | AppError::InvalidArgument(_))
        );

        // 2. Reject oversized file
        let big_file = tmp_dir.join("oversized.bin");
        atomic_write_bytes(&big_file, &[b'x'; 2048]).unwrap();
        let size_err = read_bounded_regular_file(&big_file, 1024)
            .expect_err("file exceeding max_bytes must be rejected");
        assert!(size_err.to_string().contains("exceeds maximum"));

        // Exact boundary succeeds
        let exact_ok = read_bounded_regular_file(&big_file, 2048)
            .expect("file at exact max_bytes boundary must succeed");
        assert_eq!(exact_ok.len(), 2048);

        // 3. Reject character device on Unix (/dev/null or /dev/zero)
        #[cfg(unix)]
        {
            let dev_zero = Path::new("/dev/zero");
            if dev_zero.exists() {
                let dev_err = read_bounded_regular_file(dev_zero, 1024)
                    .expect_err("/dev/zero character device must be rejected");
                assert!(dev_err.to_string().contains("non-regular"));
            }
        }

        let _ = fs::remove_dir_all(tmp_dir);
    }
}
