use sqlx::SqlitePool;

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!("SELECT value FROM settings WHERE key = ?", key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.value))
}

pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO settings (key, value) VALUES (?, ?)
           ON CONFLICT (key) DO UPDATE SET value = excluded.value"#,
        key,
        value,
    )
    .execute(pool)
    .await?;
    Ok(())
}
