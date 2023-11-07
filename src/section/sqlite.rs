use rusqlite::{Connection, Result};

use crate::section::embedding::from_byte_array;

#[test]
fn test_read_database() -> Result<()> {
    let conn = Connection::open("database.db")?;
    let mut stmt = conn.prepare("SELECT text, embedding FROM embeddings2")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, [u8; 3072]>(1)?,
        ))
    })?;

    for row in rows {
        let (text, embedding) = row?;
        dbg!(text);
        dbg!(&embedding[0..10]);
        dbg!(from_byte_array(embedding));
    }

    Ok(())
}
