# 11 — Dataset de demonstração BEC e testes de segurança (benchmark movido para a Fase 4)

**What to build:** Dataset determinístico e sanitizado do cenário BEC (spec §42) que percorre evidência → eventos → candidatos → Finding → plano → entregáveis, usando EVTX **e** CSV/JSONL (F1-27). Testes executáveis de injection/traversal (hoje só descritos em `tests/*/fase0.md`). `just perf`/RF-045 foram para a Fase 4.

**Blocked by:** F1-27, F1-28, F1-25, F1-29, F1-07.

**Status:** IMPLEMENTED / PENDING-HUMAN, reduzido (ADR-0023, Etapa 7, 2026-10-23): só EVTX; CSV/JSONL não existem (F1-27)

**Origem:** RF-016, RF-017, RF-046; RF-045 movido para Fase 4

- [x] Dataset BEC reproduzível com proveniência e sem dado pessoal real. (`dfir-app demo`, `service/demo.rs`, versão `bec-demo-1`; sintético e rotulado DEMO; o EVTX é fornecido pelo operador, não distribuído)
- [x] Roteiro de demo ponta a ponta offline documentado. (`docs/DEMO.md`; a execução com a rede cortada não foi medida)
- [ ] Testes automatizados de injection/traversal (caminho, nome de arquivo, template, envelope JSONL, mapeamento CSV). Parcial: valem os dos documentos (Etapa 5) e da proveniência (Etapa 6); envelope JSONL e mapeamento CSV não têm teste porque não há importador.
- [x] Sem metas de benchmark neste ticket.
