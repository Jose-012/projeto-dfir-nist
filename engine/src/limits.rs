//! Limites de execução do engine que não vêm no `Request`.
//!
//! Os limites que o chamador controla (`max_items`, `max_output_bytes`) chegam
//! no próprio request e são validados por `dfir_protocol::validate`. Por
//! padrão o app os envia **sem limite prático** (ADR-0020); só valem se o
//! operador pedir um teto explícito; os de
//! tamanho de linha e profundidade JSON ficam centralizados em `dfir-protocol`
//! (`MAX_LINE_BYTES`, `MAX_JSON_DEPTH`) porque app e engine precisam aplicar
//! exatamente os mesmos valores. Aqui ficam só os limites internos do engine.

/// Quantos erros de registro individuais (`PARSER_RECORD_ERROR`) o parser EVTX
/// tolera antes de abortar o job com `PARSER_RECORD_ERROR_LIMIT`.
pub const MAX_RECORD_ERRORS: usize = 64;

/// Itens por mensagem `RESULT` parcial. O batch também é fechado antes se a
/// linha JSONL ultrapassar `dfir_protocol::MAX_LINE_BYTES`.
pub const RESULT_BATCH_SIZE: usize = 128;
