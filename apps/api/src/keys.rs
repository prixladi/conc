//! API key storage backed by SQLite.
//!
//! Keys are random 256-bit tokens. Only their SHA-256 hash is stored, the
//! plaintext is shown once when the key is created. Keys never expire, they
//! can only be revoked.

use std::{
    fs,
    io::Read,
    os::unix::fs::OpenOptionsExt,
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};

const KEY_PREFIX: &str = "conc_";
/// Number of characters of the plaintext key kept for identification in listings.
const DISPLAY_PREFIX_LEN: usize = KEY_PREFIX.len() + 8;

/// Schema migrations, applied in order. `PRAGMA user_version` holds the number applied.
const MIGRATIONS: &[&str] = &[r#"
    CREATE TABLE api_keys (
        id           INTEGER PRIMARY KEY AUTOINCREMENT,
        name         TEXT    NOT NULL,
        prefix       TEXT    NOT NULL,
        key_hash     TEXT    NOT NULL UNIQUE,
        created_at   INTEGER NOT NULL,
        last_used_at INTEGER,
        revoked_at   INTEGER
    );
    -- A name identifies one active key, revoked keys keep their name for auditing.
    CREATE UNIQUE INDEX api_keys_active_name ON api_keys(name) WHERE revoked_at IS NULL;
"#];

#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("an active key named '{0}' already exists")]
    DuplicateName(String),
    #[error("key name must not be empty")]
    EmptyName,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiKey {
    pub id: i64,
    pub name: String,
    pub prefix: String,
    pub created_at: u64,
    pub last_used_at: Option<u64>,
    pub revoked_at: Option<u64>,
}

/// The identity attached to an authenticated request.
#[derive(Debug, Clone)]
pub struct Principal {
    pub key_id: i64,
    pub key_name: String,
}

pub struct KeyStore {
    conn: Mutex<Connection>,
}

impl KeyStore {
    pub fn open(path: &Path) -> Result<Self, KeyStoreError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        // Create the file owner-only before SQLite opens it.
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(path)?;

        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Creates a new key and returns its metadata together with the plaintext token.
    pub fn create(&self, name: &str) -> Result<(ApiKey, String), KeyStoreError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(KeyStoreError::EmptyName);
        }

        let token = generate_token()?;
        let prefix = token[..DISPLAY_PREFIX_LEN].to_string();
        let now = now();

        let conn = self.conn.lock().unwrap();
        let inserted = conn.execute(
            "INSERT INTO api_keys (name, prefix, key_hash, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![name, prefix, hash(&token), now as i64],
        );
        match inserted {
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                return Err(KeyStoreError::DuplicateName(name.to_string()))
            }
            other => other?,
        };

        let key = ApiKey {
            id: conn.last_insert_rowid(),
            name: name.to_string(),
            prefix,
            created_at: now,
            last_used_at: None,
            revoked_at: None,
        };
        Ok((key, token))
    }

    pub fn list(&self, include_revoked: bool) -> Result<Vec<ApiKey>, KeyStoreError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, prefix, created_at, last_used_at, revoked_at FROM api_keys
             WHERE ?1 OR revoked_at IS NULL ORDER BY id",
        )?;
        let keys = stmt
            .query_map([include_revoked], |row| {
                Ok(ApiKey {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    prefix: row.get(2)?,
                    created_at: row.get::<_, i64>(3)? as u64,
                    last_used_at: row.get::<_, Option<i64>>(4)?.map(|v| v as u64),
                    revoked_at: row.get::<_, Option<i64>>(5)?.map(|v| v as u64),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(keys)
    }

    /// Revokes the active key with the given name. Returns false if there is none.
    pub fn revoke(&self, name: &str) -> Result<bool, KeyStoreError> {
        let conn = self.conn.lock().unwrap();
        let changed = conn.execute(
            "UPDATE api_keys SET revoked_at = ?1 WHERE name = ?2 AND revoked_at IS NULL",
            params![now() as i64, name],
        )?;
        Ok(changed > 0)
    }

    /// Looks up an active key by its plaintext token and records its usage.
    pub fn authenticate(&self, token: &str) -> Result<Option<Principal>, KeyStoreError> {
        if !token.starts_with(KEY_PREFIX) {
            return Ok(None);
        }
        let conn = self.conn.lock().unwrap();
        let principal = conn
            .query_row(
                "UPDATE api_keys SET last_used_at = ?1
                 WHERE key_hash = ?2 AND revoked_at IS NULL
                 RETURNING id, name",
                params![now() as i64, hash(token)],
                |row| {
                    Ok(Principal {
                        key_id: row.get(0)?,
                        key_name: row.get(1)?,
                    })
                },
            )
            .optional()?;
        Ok(principal)
    }
}

fn migrate(conn: &Connection) -> Result<(), KeyStoreError> {
    let applied: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(applied as usize) {
        conn.execute_batch(&format!(
            "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
            i + 1
        ))?;
    }
    Ok(())
}

fn generate_token() -> std::io::Result<String> {
    let mut bytes = [0u8; 32];
    fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(format!("{KEY_PREFIX}{}", hex::encode(bytes)))
}

fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> KeyStore {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        KeyStore {
            conn: Mutex::new(conn),
        }
    }

    #[test]
    fn create_authenticate_revoke() {
        let s = store();
        let (key, token) = s.create("laptop").unwrap();
        assert!(token.starts_with("conc_") && token.len() == 5 + 64);
        assert!(token.starts_with(&key.prefix));

        let p = s.authenticate(&token).unwrap().unwrap();
        assert_eq!(p.key_name, "laptop");
        assert!(s.list(false).unwrap()[0].last_used_at.is_some());

        assert!(s.authenticate("conc_nope").unwrap().is_none());
        assert!(s.authenticate("garbage").unwrap().is_none());

        assert!(s.revoke("laptop").unwrap());
        assert!(!s.revoke("laptop").unwrap());
        assert!(s.authenticate(&token).unwrap().is_none());
        assert!(s.list(false).unwrap().is_empty());
        assert_eq!(s.list(true).unwrap().len(), 1);
    }

    #[test]
    fn names_are_unique_among_active_keys() {
        let s = store();
        s.create("ci").unwrap();
        assert!(matches!(
            s.create("ci"),
            Err(KeyStoreError::DuplicateName(_))
        ));
        s.revoke("ci").unwrap();
        s.create("ci").unwrap();
        assert!(matches!(s.create("  "), Err(KeyStoreError::EmptyName)));
    }

    #[test]
    fn migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
    }
}
