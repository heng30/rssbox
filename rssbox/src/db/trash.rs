use crate::config;
use rusqlite::{Connection, Result};

fn connection() -> Result<Connection> {
    let (_, _, db_path) = config::path();
    Connection::open(db_path)
}

pub fn init() -> Result<()> {
    let conn = connection()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS trash (
             id INTEGER PRIMARY KEY,
             md5 TEXT NOT NULL UNIQUE
             )",
        [],
    )?;

    Ok(())
}

pub fn insert(md5: &str) -> Result<()> {
    let conn = connection()?;

    conn.execute("INSERT INTO trash (md5) VALUES (?)", [md5])?;
    Ok(())
}

pub fn delete_all() -> Result<()> {
    let conn = connection()?;
    conn.execute("DELETE FROM trash", [])?;
    Ok(())
}

pub fn is_exist(md5: &str) -> Result<bool> {
    let conn = connection()?;
    let cnt = conn.query_row::<i64, _, _>(
        &format!("SELECT 1 FROM trash WHERE md5='{}'", md5),
        [],
        |r| r.get(0),
    )?;
    Ok(cnt == 1)
}

pub fn row_count() -> Result<i32> {
    let conn = connection()?;
    let cnt = conn.query_row::<i32, _, _>("SELECT COUNT(*) FROM trash", [], |r| r.get(0))?;
    Ok(cnt)
}
