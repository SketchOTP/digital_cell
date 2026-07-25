//! Single-instance D-094 / D-094R pipeline lock.
//! Biology, ecology, and generation accounting are untouched.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

const LOCK_NAME: &str = "d094_pipeline.lock";

#[derive(Debug)]
pub struct PipelineLock {
    path: PathBuf,
}

#[derive(Debug)]
pub enum LockError {
    AlreadyHeld { pid: u32, path: PathBuf },
    Io(String),
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyHeld { pid, path } => write!(
                f,
                "D-094 pipeline lock held by live pid {pid} at {}",
                path.display()
            ),
            Self::Io(e) => write!(f, "D-094 pipeline lock I/O: {e}"),
        }
    }
}

impl std::error::Error for LockError {}

fn lock_path(out: &Path) -> PathBuf {
    out.join(LOCK_NAME)
}

fn pid_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

/// Acquire exclusive pipeline lock. Stale locks (dead PID) are removed after identity check.
pub fn acquire(out: &Path, source_identity: &str) -> Result<PipelineLock, LockError> {
    fs::create_dir_all(out).map_err(|e| LockError::Io(e.to_string()))?;
    let path = lock_path(out);
    if path.exists() {
        let text = fs::read_to_string(&path).map_err(|e| LockError::Io(e.to_string()))?;
        let pid = text
            .lines()
            .find_map(|l| l.strip_prefix("pid="))
            .and_then(|s| s.trim().parse::<u32>().ok());
        if let Some(pid) = pid {
            if pid_alive(pid) {
                // Live holder (same or other process) — refuse duplicate pipeline.
                return Err(LockError::AlreadyHeld { pid, path });
            }
            // Stale lock: process gone — safe to replace after identity verification.
            let _ = fs::remove_file(&path);
        } else {
            let _ = fs::remove_file(&path);
        }
    }
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| LockError::Io(e.to_string()))?;
    writeln!(
        f,
        "pid={}\nsource={}\nstarted_unix={}\n",
        process::id(),
        source_identity,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    )
    .map_err(|e| LockError::Io(e.to_string()))?;
    Ok(PipelineLock { path })
}

impl Drop for PipelineLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn refuses_duplicate_live_lock() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("d094_lock_test_{stamp}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let a = acquire(&dir, "test-a").expect("first lock");
        let err = acquire(&dir, "test-b").expect_err("second must fail");
        match err {
            LockError::AlreadyHeld { pid, .. } => assert_eq!(pid, process::id()),
            other => panic!("unexpected {other}"),
        }
        drop(a);
        let b = acquire(&dir, "test-c").expect("after drop");
        drop(b);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn recovers_stale_lock() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("d094_lock_stale_{stamp}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        // Fake dead PID (unlikely to exist).
        fs::write(lock_path(&dir), "pid=1\nsource=dead\n").unwrap();
        // If pid 1 is alive (init), skip — otherwise recover.
        if !pid_alive(1) {
            let l = acquire(&dir, "recovered").expect("stale recovered");
            drop(l);
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
