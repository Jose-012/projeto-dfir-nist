<!-- doc-type: decision | status: superseded -->
> **Status:** SUPERSEDED BY ADR-0007 (2026-08-20).
> The following document is retained as historical design rationale and is not an active runtime requirement.

# ADR-0001 — Fundação da Fase 0

## Decisão

Adotar monólito modular de implantação com `dfir-app` e `dfir-engine` em processos separados, ambos em Rust; SQLite/WAL para persistência; JSONL em stdin/stdout para IPC do engine; storage local separado; autenticação Argon2id e RBAC por papel + acesso ao incidente.

## Justificativa

Preserva as fronteiras arquiteturais e a defesa em profundidade descritas na especificação, sem criar microserviços ou dependências externas desnecessárias.
