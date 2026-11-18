//! Tipo de falha compartilhado pelos módulos do engine.

/// Falha de processamento de um request: `(código estável, mensagem)`.
///
/// O código vira o campo `code` de uma `ErrorMessage` do protocolo; a
/// mensagem é sanitizada e truncada por `ErrorMessage::new`.
pub type EngineFailure = (&'static str, String);
