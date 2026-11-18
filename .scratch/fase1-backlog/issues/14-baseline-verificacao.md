# 14 — Baseline de verificação e CI mínimo

**What to build:** Executar `fmt/check/clippy/nextest` no toolchain pinado (Windows + Linux), registrar `EVD-*` com versões efetivas e abrir defeito para tudo que falhar. Promover F1-01..F1-05 a `VERIFIED` somente com evidência executada. CI mínimo rodando `just verify` e `just supply-chain`.

**Blocked by:** F1-13.

**Status:** PLANNED (ADR-0018)

**Status vigente (2026-11-04, espelho de `docs/rtm.md`; Gap origem: INC-03):** PARCIAL — detalhe na linha F1-14 da RTM, que é a autoridade. A linha `PLANNED` acima é histórica.

**Origem:** DOC-06, AGENTS §12

- [ ] Relatório de gates com versões efetivas de rustc/cargo; nenhum PASS sem execução.
- [ ] Defeitos abertos para cada falha e ligados a ticket.
- [ ] CI executa `just verify` e `just supply-chain` em cada mudança.
