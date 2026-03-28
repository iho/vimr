use rusqlite::{Connection, Result, params};
use std::path::PathBuf;

pub struct History {
    conn: Connection,
}

#[derive(Debug)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub visit_count: i64,
    pub last_visited: i64,
}

impl History {
    pub fn open() -> Result<Self> {
        let db_path = Self::db_path();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                visit_count INTEGER NOT NULL DEFAULT 1,
                last_visited INTEGER NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS history_url ON history(url);
        ")?;
        Ok(History { conn })
    }

    fn db_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_default()
            .join("vimr")
            .join("history.db")
    }

    pub fn add(&self, url: &str, title: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.conn.execute(
            "INSERT INTO history (url, title, visit_count, last_visited) VALUES (?1, ?2, 1, ?3)
             ON CONFLICT(url) DO UPDATE SET visit_count = visit_count + 1, last_visited = ?3, title = ?2",
            params![url, title, now],
        )?;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<Vec<HistoryEntry>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT url, title, visit_count, last_visited FROM history
             WHERE url LIKE ?1 OR title LIKE ?1
             ORDER BY visit_count DESC, last_visited DESC
             LIMIT 20"
        )?;
        let entries = stmt.query_map(params![pattern], |row| {
            Ok(HistoryEntry {
                url: row.get(0)?,
                title: row.get(1)?,
                visit_count: row.get(2)?,
                last_visited: row.get(3)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(entries)
    }
}
