use blinker_core_common::{BlinkerError, Result};
use std::fs::File;
use std::io::{Read};
use std::path::{Path, PathBuf};
use rusqlite::{Connection, params};
use crate::{LibraryItem, LibraryQuery, LibraryStore, AddOutcome};

const SCHEMA_SQL: &str = include_str!("../../../sql/001_initial_schema.sql");

pub struct LibraryDatabase {
    conn: Connection,
}

impl LibraryDatabase {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| BlinkerError::Database(format!("open db: {}", e)))?;
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| BlinkerError::Database(format!("pragma: {}", e)))?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn get_schema_version(&self) -> Result<i64> {
        let mut stmt = match self.conn.prepare("SELECT MAX(version) FROM schema_version") {
            Ok(s) => s,
            Err(_) => return Ok(0),
        };
        let v: Option<i64> = stmt
            .query_row([], |row| row.get(0))
            .ok()
            .unwrap_or(None);
        Ok(v.unwrap_or(0))
    }

    pub fn migrate(&self) -> Result<()> {
        let version = self.get_schema_version()?;
        if version < 1 {
            self.conn
                .execute_batch(SCHEMA_SQL)
                .map_err(|e| BlinkerError::Database(format!("migrate: {}", e)))?;
        }
        Ok(())
    }

    fn now_secs() -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
    }

    fn file_hash(path: &Path) -> Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = blake3::Hasher::new();
        let mut buf = [0u8; 64 * 1024];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
        }
        Ok(hasher.finalize().to_hex().to_string())
    }

    fn infer_file_type(path: &Path) -> String {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "unknown".into())
    }
}

impl LibraryStore for LibraryDatabase {
    fn add_or_update_path(&self, path: &Path) -> Result<AddOutcome> {
        let meta = std::fs::metadata(path).map_err(|e| BlinkerError::Io(e))?;
        if !meta.is_file() {
            return Err(BlinkerError::Parsing(format!("not a file: {}", path.display())));
        }
        let file_size = meta.len();
        let file_hash = Self::file_hash(path)?;
        let file_type = Self::infer_file_type(path);
        let title = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled").to_string();
        let author: Option<String> = None;
        let now = Self::now_secs();
        // Determine if an entry exists for this file_path
        let existing: Option<(String, String)> = self.conn
            .query_row(
                "SELECT id, file_hash FROM library_item WHERE file_path = ?1",
                params![path.to_string_lossy()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        let outcome = if let Some((existing_id, existing_hash)) = existing {
            if existing_hash == file_hash {
                // Unchanged metadata update: touch indexed_at
                self.conn.execute(
                    "UPDATE library_item SET indexed_at = ?2 WHERE id = ?1",
                    params![existing_id, now],
                ).map_err(|e| BlinkerError::Database(format!("touch indexed_at: {}", e)))?;
                AddOutcome::Unchanged { id: existing_id }
            } else {
                self.conn
                    .execute(
                        "UPDATE library_item SET file_hash=?2, file_type=?3, file_size=?4, title=?5, author=?6, modified_at=?7, indexed_at=?7 WHERE id=?1",
                        params![existing_id, file_hash, file_type, file_size as i64, title, author, now],
                    )
                    .map_err(|e| BlinkerError::Database(format!("update item: {}", e)))?;
                AddOutcome::Updated { id: existing_id }
            }
        } else {
            // New insert; ID uses content hash for PoC
            let id = file_hash.clone();
            self.conn
                .execute(
                    "INSERT INTO library_item (
                        id, file_path, file_hash, file_type, file_size,
                        title, author, publisher, subject, language, page_count,
                        created_at, modified_at, indexed_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL, NULL, NULL, ?8, ?8, ?8)",
                    params![
                        id,
                        path.to_string_lossy(),
                        file_hash,
                        file_type,
                        file_size as i64,
                        title,
                        author,
                        now,
                    ],
                )
                .map_err(|e| BlinkerError::Database(format!("insert item: {}", e)))?;
            AddOutcome::Created { id }
        };

        Ok(outcome)
    }

    fn get_item(&self, id: &str) -> Result<Option<LibraryItem>> {
        let mut stmt = self.conn
            .prepare("SELECT id, file_path, file_hash, file_type, file_size, title, author FROM library_item WHERE id = ?1")
            .map_err(|e| BlinkerError::Database(format!("prepare get: {}", e)))?;
        let row = stmt.query_row(params![id], |row| {
            let id: String = row.get(0)?;
            let file_path: String = row.get(1)?;
            let file_hash: String = row.get(2)?;
            let file_type: String = row.get(3)?;
            let file_size: i64 = row.get(4)?;
            let title: String = row.get(5)?;
            let author: Option<String> = row.get(6)?;
            let metadata = blinker_core_common::types::Metadata {
                title,
                author,
                ..Default::default()
            };
            Ok(LibraryItem {
                id,
                file_path: PathBuf::from(file_path),
                file_hash,
                file_type,
                file_size: file_size as u64,
                metadata,
                tags: vec![],
            })
        });

        match row {
            Ok(item) => Ok(Some(item)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BlinkerError::Database(format!("get item: {}", e))),
        }
    }

    fn query(&self, query: &LibraryQuery) -> Result<Vec<LibraryItem>> {
        // Minimal PoC: simple SELECT with LIKE filters and optional type filter
        let mut sql = String::from("SELECT id, file_path, file_hash, file_type, file_size, title, author FROM library_item");
        let mut clauses: Vec<String> = vec![];
        let mut params_box: Vec<(String, String)> = vec![];

        if let Some(text) = &query.text {
            clauses.push("(title LIKE :text OR ifnull(author,'') LIKE :text)".into());
            params_box.push((":text".into(), format!("%{}%", text)));
        }
        if let Some(types) = &query.file_types {
            if !types.is_empty() {
                let placeholders: Vec<String> = (0..types.len()).map(|i| format!(":t{}", i)).collect();
                clauses.push(format!("file_type IN ({})", placeholders.join(",")));
                for (i, t) in types.iter().enumerate() {
                    params_box.push((format!(":t{}", i), t.clone()));
                }
            }
        }
        if !clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&clauses.join(" AND "));
        }
        sql.push_str(" ORDER BY title COLLATE NOCASE ASC");
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        } else {
            sql.push_str(" LIMIT 100");
        }

        let mut stmt = self.conn
            .prepare(&sql)
            .map_err(|e| BlinkerError::Database(format!("prepare query: {}", e)))?;

        let mut rows = stmt.query_named(
            params_box.iter().map(|(k,v)| (k.as_str(), &v as &dyn rusqlite::ToSql))
        ).map_err(|e| BlinkerError::Database(format!("run query: {}", e)))?;

        let mut out = vec![];
        while let Some(row) = rows.next().map_err(|e| BlinkerError::Database(format!("row: {}", e)))? {
            let id: String = row.get(0)?;
            let file_path: String = row.get(1)?;
            let file_hash: String = row.get(2)?;
            let file_type: String = row.get(3)?;
            let file_size: i64 = row.get(4)?;
            let title: String = row.get(5)?;
            let author: Option<String> = row.get(6)?;
            let metadata = blinker_core_common::types::Metadata { title, author, ..Default::default() };
            out.push(LibraryItem { id, file_path: PathBuf::from(file_path), file_hash, file_type, file_size: file_size as u64, metadata, tags: vec![] });
        }
        Ok(out)
    }
}
