# 26 — Evento realmente normalizado

**What to build:** Estender `NormalizedEvent` com campos estruturados opcionais (`event_id`, provedor/canal, host, usuário, IP de origem/destino, processo, ação, resultado, tipo de fonte). O EVTX preenche o que existir; `data` guarda o registro original. Versionar o schema do protocolo e do banco.

**Blocked by:** F1-16.

**Status:** IMPLEMENTED / PENDING-HUMAN (ADR-0025, 2026-10-27). Esquema do protocolo 4; campos estruturais opcionais; migration 0012; filtro `campo:valor`. Testado em rustc 1.97.1.

**Nota (2026-11-14) — DOC-01.** O pin atual é **1.99.1**; `just verify` de 2026-11-14 reexecutou os testes deste ticket no 1.99.1 (`EVD-001`) e o e2e confirmou os campos estruturais e o filtro sobre o EVTX real (`EVD-002`). A menção a "1.97.1" acima é o registro histórico da rodada de 2026-10-27.

**Origem:** RF-048, DEM-FONTE-02, DEM-TL-01

- [x] EVTX real preenche `event_id`, canal, host e usuário (e2e: "estrutura: eventos trazem host, conta e tipo de fonte").
- [x] Schema versionado (3 → 4); itens da versão 3 continuam legíveis e o app recalcula os campos do registro (testes no protocolo e no serviço).
- [x] Teste de compatibilidade do envelope: a forma da v3 sem campo novo continua serializando igual (`structured_fields_are_omitted_when_empty_and_present_when_set`). Importador JSONL: F1-27, adiado.
