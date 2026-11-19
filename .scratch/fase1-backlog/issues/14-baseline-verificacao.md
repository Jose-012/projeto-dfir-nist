# 14 — Baseline de verificação e CI mínimo

**What to build:** Executar `fmt/check/clippy/nextest` no toolchain pinado (Windows + Linux), registrar `EVD-*` com versões efetivas e abrir defeito para tudo que falhar. Promover F1-01..F1-05 a `VERIFIED` somente com evidência executada. CI mínimo rodando `just verify` e `just supply-chain`.

**Blocked by:** F1-13.

**Status:** PLANNED (ADR-0018)

**Status vigente (2026-11-19, espelho de `docs/rtm.md`; Gap origem: INC-03):** VERIFIED: `EVD-052` cobre o baseline local e `EVD-054` cobre o CI em Linux x86_64; Windows permanece separado em F1-22/WVR-001. A linha `PLANNED` acima é histórica.

**Origem:** DOC-06, AGENTS §12

- [x] Relatório de gates com versões efetivas de rustc/cargo; nenhum PASS sem execução (`EVD-052`, `EVD-054`).
- [x] Falhas registradas e ligadas a este ticket, com correção rastreável; a falha inicial de setup do CI está preservada em `EVD-054` e foi corrigida em `7acf795`.
- [x] CI executa `just verify` e `just supply-chain` em cada mudança na main e em pull requests (`EVD-054`, execução 44567609855).
