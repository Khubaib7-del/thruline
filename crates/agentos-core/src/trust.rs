//! Trust-on-first-use for project memory (security finding 1). The decisions
//! file is injected into agents' prompts, so an edit that bypassed agentos —
//! a `git pull` of a poisoned repo, a script — must not reach an agent
//! unreviewed. Fingerprints live OUTSIDE the repo (in the user profile), so
//! whoever tampers with the repo cannot also bless their own tampering.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustStatus {
    /// Fingerprint matches — content only changed through agentos.
    Trusted,
    /// No fingerprint recorded for this project on this machine yet.
    Untrusted,
    /// Content changed outside agentos since it was last approved.
    Changed,
}

/// `AGENTOS_TRUST_DB` overrides the location (used by tests); default is
/// `~/.agentos/trust.json` — deliberately not inside any repo.
pub fn default_db_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("AGENTOS_TRUST_DB") {
        return Some(PathBuf::from(p));
    }
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))?;
    Some(PathBuf::from(home).join(".agentos").join("trust.json"))
}

pub fn project_key(state_root: &Path) -> String {
    let canon = state_root
        .canonicalize()
        .unwrap_or_else(|_| state_root.to_path_buf());
    canon.to_string_lossy().to_lowercase()
}

pub fn hash_file(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    Some(format!("{:x}", Sha256::digest(&bytes)))
}

pub fn status(db: &Path, state_root: &Path, file: &Path) -> TrustStatus {
    let Some(current) = hash_file(file) else {
        return TrustStatus::Untrusted;
    };
    match load(db).get(&project_key(state_root)) {
        Some(approved) if *approved == current => TrustStatus::Trusted,
        Some(_) => TrustStatus::Changed,
        None => TrustStatus::Untrusted,
    }
}

pub fn approve(db: &Path, state_root: &Path, file: &Path) -> Result<()> {
    let hash = hash_file(file).context("hashing decisions file for trust approval")?;
    if let Some(dir) = db.parent() {
        fs::create_dir_all(dir)?;
    }
    // Concurrent writers (parallel agents/sessions) would lose updates in a
    // plain read-modify-write; a lock file serializes them, and temp+rename
    // keeps the db uncorrupted even on a crash mid-write.
    let _lock = FileLock::acquire(&db.with_extension("lock"));
    let mut map = load(db);
    map.insert(project_key(state_root), hash);
    let tmp = db.with_extension("tmp");
    fs::write(&tmp, serde_json::to_string_pretty(&map)?)?;
    fs::rename(&tmp, db).with_context(|| format!("writing trust db {}", db.display()))?;
    Ok(())
}

struct FileLock(PathBuf);

impl FileLock {
    /// Best-effort lock: retry ~500ms, then proceed unlocked rather than
    /// hang the user's tooling on a stale lock from a crashed process.
    fn acquire(path: &Path) -> Option<Self> {
        for _ in 0..100 {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
            {
                Ok(_) => return Some(Self(path.to_path_buf())),
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(5)),
            }
        }
        None
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn load(db: &Path) -> HashMap<String, String> {
    fs::read_to_string(db)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
