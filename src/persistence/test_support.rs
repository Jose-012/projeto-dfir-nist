//! Fixtures compartilhados somente entre testes dos repositories.

use sqlx::SqlitePool;

pub(crate) async fn seed_incident(pool: &SqlitePool, id: &str) {
    sqlx::query("INSERT INTO incidents(id,name) VALUES(?,?)")
        .bind(id)
        .bind(format!("incident {id}"))
        .execute(pool)
        .await
        .expect("incident insert");
}
