use std::io;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("storage error: {0}")]
    Storage(#[from] io::Error),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("engine error: {0}")]
    Engine(String),
}

impl AppError {
    /// Mensagem segura para exibir na interface. Antes desta migração (v6.4.5)
    /// esta mesma lógica decidia o par (status HTTP, texto) de `IntoResponse`;
    /// a interface nativa não tem status HTTP, então só o texto permanece —
    /// os mesmos textos genéricos usados antes (nunca detalhes internos de
    /// storage/banco) continuam genéricos aqui, pela mesma razão original
    /// (não vazar caminhos/erros internos do SO ou do SQLite para a tela).
    pub fn user_message(&self) -> String {
        match self {
            Self::Config(message) => message.clone(),
            Self::NotFound(message) => message.clone(),
            Self::Storage(_) => "storage operation failed".into(),
            // A camada de persistência sinaliza entrada inválida do usuário
            // (IOC malformado, id de técnica ATT&CK fora do formato, profile
            // sem entradas...) com `sqlx::Error::Protocol(<motivo>)`. O motivo
            // é texto nosso, sem detalhes internos: mostrar à pessoa em vez de
            // reduzir a um "database operation failed" opaco que esconde o que
            // corrigir (mesma regra de antes, ver `error.rs` histórico).
            Self::Database(sqlx::Error::Protocol(reason)) => reason.clone(),
            Self::Database(_) => "database operation failed".into(),
            Self::Engine(_) => "analysis engine failed".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn persistence_validation_errors_surface_their_reason() {
        let error = AppError::Database(sqlx::Error::Protocol("invalid IP address: nope".into()));
        assert_eq!(error.user_message(), "invalid IP address: nope");
    }

    #[test]
    fn other_database_errors_stay_opaque() {
        let error = AppError::Database(sqlx::Error::RowNotFound);
        assert_eq!(error.user_message(), "database operation failed");
    }

    #[test]
    fn engine_errors_show_a_generic_headline() {
        let error = AppError::Engine("engine process exited with signal 9".into());
        assert_eq!(error.user_message(), "analysis engine failed");
    }
}
