use crate::config;
use rusqlite::{Connection, Result};

fn connection() -> Result<Connection> {
    let (_, _, db_path) = config::path();
    Connection::open(db_path)
}

pub fn init() -> Result<()> {
    let conn = connection()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS rss (
             id INTEGER PRIMARY KEY,
             uuid TEXT NOT NULL UNIQUE,
             config TEXT NOT NULL
             )",
        [],
    )?;

    Ok(())
}

pub fn delete(uuid: &str) -> Result<()> {
    let conn = connection()?;
    conn.execute("DELETE FROM rss WHERE uuid=?", [uuid])?;
    Ok(())
}

pub fn insert(uuid: &str, config: &str) -> Result<()> {
    let conn = connection()?;

    conn.execute(
        "INSERT INTO rss (uuid, config) VALUES (?, ?)",
        [uuid, config],
    )?;
    Ok(())
}

pub fn update(uuid: &str, config: &str) -> Result<()> {
    let conn = connection()?;

    conn.execute("UPDATE rss SET config=? WHERE uuid=?", [config, uuid])?;

    Ok(())
}

#[allow(dead_code)]
pub fn select(uuid: &str) -> Result<Option<String>> {
    let conn = connection()?;

    let mut stmt = conn.prepare(&format!("SELECT config FROM rss WHERE uuid='{}'", uuid))?;
    let mut rows = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;

    if let Some(Ok(row)) = rows.next() {
        return Ok(Some(row));
    }
    Ok(None)
}

pub fn select_all() -> Result<Vec<(String, String)>> {
    let conn = connection()?;

    let mut stmt = conn.prepare("SELECT uuid, config FROM rss")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    Ok(rows.flatten().collect())
}

pub fn is_exist(uuid: &str) -> Result<bool> {
    let conn = connection()?;
    let cnt = conn.query_row::<i64, _, _>(
        &format!("SELECT 1 FROM rss WHERE uuid='{}'", uuid),
        [],
        |r| r.get(0),
    )?;
    Ok(cnt == 1)
}
