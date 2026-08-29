use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct System {
    pub id: String,
    pub name: String,
    pub extensions: String,
    pub retroarch_core: Option<String>,
    pub standalone_binary: Option<String>,
}

pub struct NewSystem {
    pub id: String,
    pub name: String,
    pub extensions: String,
    pub retroarch_core: Option<String>,
    pub standalone_binary: Option<String>,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<System>, sqlx::Error> {
    sqlx::query_as!(
        System,
        "SELECT id, name, extensions, retroarch_core, standalone_binary FROM systems ORDER BY name"
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<System>, sqlx::Error> {
    sqlx::query_as!(
        System,
        "SELECT id, name, extensions, retroarch_core, standalone_binary FROM systems WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, new: NewSystem) -> Result<System, sqlx::Error> {
    sqlx::query_as!(
        System,
        r#"INSERT INTO systems (id, name, extensions, retroarch_core, standalone_binary)
           VALUES (?, ?, ?, ?, ?)
           RETURNING id, name, extensions, retroarch_core, standalone_binary"#,
        new.id,
        new.name,
        new.extensions,
        new.retroarch_core,
        new.standalone_binary,
    )
    .fetch_one(pool)
    .await
}

pub async fn update(pool: &SqlitePool, id: &str, new: NewSystem) -> Result<System, sqlx::Error> {
    sqlx::query_as!(
        System,
        r#"UPDATE systems SET name = ?, extensions = ?, retroarch_core = ?, standalone_binary = ?
           WHERE id = ?
           RETURNING id, name, extensions, retroarch_core, standalone_binary"#,
        new.name,
        new.extensions,
        new.retroarch_core,
        new.standalone_binary,
        id,
    )
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM systems WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}
