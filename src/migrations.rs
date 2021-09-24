use rusqlite::Connection;
use anyhow::Result;

pub fn migration1(conn: &Connection) -> Result<()> {
    conn.execute(
        "\
CREATE TABLE IF NOT EXISTS cats (
    id INTEGER NOT NULL PRIMARY KEY ASC,
    name TEXT NOT NULL,
    age INTEGER NOT NULL,
    breed TEXT)",
        [],
    )?;
    Ok(())
}