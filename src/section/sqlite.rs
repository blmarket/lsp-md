use super::embedding::{from_byte_array, to_byte_array};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::convert::TryInto;

struct EmbeddingCache {
    pool: Pool<SqliteConnectionManager>,
}

impl EmbeddingCache {
    fn new(filename: &str) -> Self {
        EmbeddingCache {
            pool: Pool::new(SqliteConnectionManager::file(filename)).unwrap(),
        }
    }
    
    fn memory() -> Self {
        EmbeddingCache {
            pool: Pool::new(SqliteConnectionManager::memory()).unwrap(),
        }
    }

    #[cfg(test)]
    fn enumerate(&self) -> anyhow::Result<Vec<(String, [f32; 768])>> {
        let conn = self.pool.get().unwrap();
        let mut stmt =
            conn.prepare("SELECT text, embedding FROM embeddings2")?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                from_byte_array(row.get::<_, [u8; 3072]>(1)?),
            ))
        })?;

        Ok(rows.into_iter().map(|v| v.unwrap()).collect())
    }

    fn get(&self, text: &str) -> Option<[f32; 768]> {
        let conn = self.pool.get().ok()?;
        let mut stmt = conn
            .prepare("SELECT embedding FROM embeddings2 WHERE text = ?1")
            .ok()?;
        let embedding: Vec<u8> =
            stmt.query_row([text], |row| row.get(0)).ok()?;
        Some(from_byte_array(embedding.try_into().unwrap()))
    }

    fn insert(
        &self,
        text: &str,
        embedding: &[f32; 768],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "INSERT INTO embeddings2 (text, embedding) VALUES (?1, ?2)",
        )?;
        stmt.execute(params![text, to_byte_array(embedding)])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_database() -> Result<(), Box<dyn std::error::Error>> {
        let cache = EmbeddingCache::new("database.db");

        for row in cache.enumerate()? {
            dbg!(row);
        }

        Ok(())
    }

    #[test]
    fn test_query() -> Result<(), Box<dyn std::error::Error>> {
        let cache = EmbeddingCache::memory();
        cache.pool.get().unwrap().execute(
            "CREATE TABLE embeddings2 (
                text TEXT PRIMARY KEY,
                embedding BLOB NOT NULL
            )",
            [],
        )?;

        let text = "hello world";
        assert_eq!(None, cache.get(text));

        let embedding = [1.0; 768];
        cache.insert(text, &embedding)?;

        let retrieved_embedding = cache.get(text).expect("should be some");
        assert_eq!(retrieved_embedding, embedding);

        Ok(())
    }
}
