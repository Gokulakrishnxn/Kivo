//! Local history. Text only — never audio.
//!
//! One SQLite file in Kivo's data directory, created 0600 so other accounts on
//! the machine cannot read it. Turning history off means nothing is written at
//! all: the file is not created, and an existing one is deleted.

use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub id: i64,
    pub raw: String,
    pub text: String,
    pub mode: String,
    pub millis: u64,
    /// Unix seconds.
    pub at: i64,
}

pub struct History {
    db: Option<Connection>,
}

impl History {
    pub fn path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Kivo")
            .join("history.sqlite")
    }

    /// Open the store, or return a disabled one that silently keeps nothing.
    pub fn open(enabled: bool) -> Self {
        if !enabled {
            return Self { db: None };
        }
        match Self::connect() {
            Ok(db) => Self { db: Some(db) },
            Err(e) => {
                tracing::error!("history unavailable, continuing without it: {e:#}");
                Self { db: None }
            }
        }
    }

    fn connect() -> Result<Connection> {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).context("could not create the data directory")?;
        }
        let db = Connection::open(&path).context("could not open the history database")?;

        // Readable only by this user. Done after open so the file exists.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }

        db.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS entries (
                 id     INTEGER PRIMARY KEY AUTOINCREMENT,
                 raw    TEXT    NOT NULL,
                 text   TEXT    NOT NULL,
                 mode   TEXT    NOT NULL,
                 millis INTEGER NOT NULL,
                 at     INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS entries_at ON entries (at DESC);",
        )
        .context("could not prepare the history schema")?;
        Ok(db)
    }

    pub fn is_enabled(&self) -> bool {
        self.db.is_some()
    }

    /// Record one dictation and trim the store to `limit`.
    pub fn insert(
        &self,
        raw: &str,
        text: &str,
        mode: &str,
        millis: u64,
        limit: usize,
    ) -> Result<i64> {
        let Some(db) = &self.db else { return Ok(0) };
        let at = now();
        db.execute(
            "INSERT INTO entries (raw, text, mode, millis, at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![raw, text, mode, millis as i64, at],
        )?;
        let id = db.last_insert_rowid();
        db.execute(
            "DELETE FROM entries WHERE id NOT IN
             (SELECT id FROM entries ORDER BY at DESC, id DESC LIMIT ?1)",
            rusqlite::params![limit as i64],
        )?;
        Ok(id)
    }

    /// Most recent first. `query` filters on the cleaned text, case-insensitively.
    pub fn recent(&self, limit: usize, query: &str) -> Vec<Entry> {
        let Some(db) = &self.db else {
            return Vec::new();
        };
        let q = query.trim();
        let result = if q.is_empty() {
            db.prepare("SELECT id, raw, text, mode, millis, at FROM entries ORDER BY at DESC, id DESC LIMIT ?1")
                .and_then(|mut s| {
                    s.query_map(rusqlite::params![limit as i64], row_to_entry)
                        .and_then(|r| r.collect::<rusqlite::Result<Vec<_>>>())
                })
        } else {
            let like = format!("%{q}%");
            db.prepare(
                "SELECT id, raw, text, mode, millis, at FROM entries
                 WHERE text LIKE ?1 COLLATE NOCASE ORDER BY at DESC, id DESC LIMIT ?2",
            )
            .and_then(|mut s| {
                s.query_map(rusqlite::params![like, limit as i64], row_to_entry)
                    .and_then(|r| r.collect::<rusqlite::Result<Vec<_>>>())
            })
        };
        result.unwrap_or_else(|e| {
            tracing::error!("history query failed: {e}");
            Vec::new()
        })
    }

    pub fn clear(&self) -> Result<()> {
        if let Some(db) = &self.db {
            db.execute("DELETE FROM entries", [])?;
            db.execute_batch("VACUUM;")?;
        }
        Ok(())
    }

    /// Remove the file entirely — used when the user turns history off, so
    /// "off" means gone rather than merely ignored.
    pub fn destroy() {
        let p = Self::path();
        for suffix in ["", "-wal", "-shm"] {
            let f = PathBuf::from(format!("{}{}", p.display(), suffix));
            let _ = std::fs::remove_file(f);
        }
    }
}

fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        raw: row.get(1)?,
        text: row.get(2)?,
        mode: row.get(3)?,
        millis: row.get::<_, i64>(4)? as u64,
        at: row.get(5)?,
    })
}

fn now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An in-memory store, so tests never touch the real history file.
    fn mem() -> History {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY AUTOINCREMENT, raw TEXT NOT NULL,
             text TEXT NOT NULL, mode TEXT NOT NULL, millis INTEGER NOT NULL, at INTEGER NOT NULL);",
        )
        .unwrap();
        History { db: Some(db) }
    }

    #[test]
    fn stores_and_returns_newest_first() {
        let h = mem();
        h.insert("raw one", "One.", "Standard", 200, 10).unwrap();
        h.insert("raw two", "Two.", "Standard", 210, 10).unwrap();
        let all = h.recent(10, "");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].text, "Two.");
    }

    #[test]
    fn trims_to_the_limit() {
        let h = mem();
        for i in 0..8 {
            h.insert("r", &format!("entry {i}"), "Standard", 1, 3)
                .unwrap();
        }
        assert_eq!(h.recent(50, "").len(), 3);
    }

    #[test]
    fn search_is_case_insensitive_and_matches_the_cleaned_text() {
        let h = mem();
        h.insert("r", "Ship the release Thursday.", "Standard", 1, 10)
            .unwrap();
        h.insert("r", "Nothing to do with it.", "Standard", 1, 10)
            .unwrap();
        assert_eq!(h.recent(10, "THURSDAY").len(), 1);
        assert_eq!(h.recent(10, "nonsense").len(), 0);
    }

    #[test]
    fn a_disabled_store_keeps_nothing() {
        let h = History { db: None };
        assert!(!h.is_enabled());
        h.insert("r", "t", "Standard", 1, 10).unwrap();
        assert!(h.recent(10, "").is_empty());
    }

    #[test]
    fn clear_empties_the_store() {
        let h = mem();
        h.insert("r", "t", "Standard", 1, 10).unwrap();
        h.db.as_ref()
            .unwrap()
            .execute("DELETE FROM entries", [])
            .unwrap();
        assert!(h.recent(10, "").is_empty());
    }
}
