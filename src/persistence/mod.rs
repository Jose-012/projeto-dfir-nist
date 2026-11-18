pub mod analytical_results;
pub mod attack_impact;
pub mod audit;
pub mod backup;
pub mod db;
pub mod detection;
pub mod findings;
pub mod ioc_recommendations;
pub mod nist_functions;
pub mod nist_profiles;
pub mod plan_items;
pub mod source_gaps;

/// Rejeição de validação de entrada em um repository, no canal que a camada de
/// serviço já traduz para mensagem acionável (`sqlx::Error::Protocol` é tratado
/// como rejeição, não como falha de banco). Vivendo aqui, cada repository usa a
/// mesma conversão em vez de repetir uma cópia própria.
pub(crate) fn validation_error(reason: impl Into<String>) -> sqlx::Error {
    sqlx::Error::Protocol(reason.into())
}

/// Executa `ROLLBACK` e **registra** a falha (F1-23): um rollback que falha em
/// silêncio deixa a conexão em estado incerto sem rastro. Nunca engolir com
/// `let _ =`.
pub async fn rollback_loudly(connection: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>) {
    if let Err(error) = sqlx::query("ROLLBACK").execute(&mut **connection).await {
        tracing::warn!(%error, "rollback failed; connection left to the pool");
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn rollback_loudly_runs_against_a_live_connection() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory pool");
        sqlx::query("CREATE TABLE t (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .expect("schema");
        let mut connection = pool.acquire().await.expect("connection");
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut *connection)
            .await
            .expect("begin");
        super::rollback_loudly(&mut connection).await;
        // Depois do rollback a conexão aceita novos comandos.
        sqlx::query("INSERT INTO t (id) VALUES (1)")
            .execute(&mut *connection)
            .await
            .expect("usable after rollback");
    }

    #[tokio::test]
    async fn rollback_loudly_on_a_connection_without_transaction_does_not_panic() {
        // F1-23: chamar rollback fora de transação devolve erro do SQLite; o
        // helper precisa engolir esse erro com log (não pode virar panic nem
        // erro propagado) e deixar a conexão utilizável.
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory pool");
        sqlx::query("CREATE TABLE t (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .expect("schema");
        let mut connection = pool.acquire().await.expect("connection");
        super::rollback_loudly(&mut connection).await;
        sqlx::query("INSERT INTO t (id) VALUES (1)")
            .execute(&mut *connection)
            .await
            .expect("usable after no-op rollback");
    }
}
