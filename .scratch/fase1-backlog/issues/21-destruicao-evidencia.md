# 21 — Destruição de evidência: ordem segura e retomada

**What to build:** Hoje o arquivo é apagado antes da transação que registra `EVIDENCE_DESTROYED`. Inverter: transação no banco primeiro (`DESTROY_REQUESTED` + custódia), depois remoção física e confirmação; falha do apagamento é registrada e permite nova tentativa. Limpar parciais de `quarantine/` na partida.

**Blocked by:** F1-16.

**Status:** VERIFIED (2026-11-15, `EVD-010`) — ordem segura (registrar antes de apagar) e retomada idempotente em `destroy_evidence_core` com 6 testes: os 4 existentes + `destroy_records_destruction_before_touching_the_file` (falha de banco não apaga nada) e `destroy_removes_detection_candidates_with_the_evidence` (purga completa incl. candidatos, custódia/auditoria preservadas). Pendente residual: limpeza de parciais em `quarantine/` na partida (fora do escopo desta pista).

**Origem:** AUD-02, AUD-21

- [x] Falha de remoção depois do registro nunca deixa banco `ACTIVE` sem registro (teste `destroy_never_leaves_active_record_after_file_removal_failure`).
- [ ] Parcial em `quarantine/` é limpo.
