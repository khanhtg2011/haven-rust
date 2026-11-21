use crate::error::{HavenError, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn create_pool<P: AsRef<Path>>(path: P) -> Result<DbPool> {
    let manager = SqliteConnectionManager::file(path);
    let pool = Pool::builder()
        .max_size(16)
        .build(manager)
        .map_err(HavenError::Pool)?;

    let conn = pool.get().map_err(HavenError::Pool)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA foreign_keys = ON;",
    )
    .map_err(HavenError::Database)?;

    super::schema::initialize_schema(&conn)?;

    Ok(pool)
}

pub fn create_in_memory_pool() -> Result<DbPool> {
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::builder()
        .max_size(1)
        .build(manager)
        .map_err(HavenError::Pool)?;

    let conn = pool.get().map_err(HavenError::Pool)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(HavenError::Database)?;
    super::schema::initialize_schema(&conn)?;

    Ok(pool)
}
