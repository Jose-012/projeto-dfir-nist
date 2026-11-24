<!-- doc-type: reference | status: current -->
# Índice de ADRs (Architecture Decision Records)

Este índice lista os 31 ADRs do projeto DFIR/NIST, seus status, relações de supersessão e links relativos.

> **Autoridade:** `docs/DOCUMENT_AUTHORITY.md` §2. As ADRs são decisões de arquitetura/escopo (nível 3). Não são "requisitos explícitos da demanda" (nível 1). Mudanças usam `supersedes`/`superseded-by`; IDs nunca são reutilizados.

---

## Tabela Mestra

| ADR | Título | Status | Supersedes | Superseded-by | Link |
|-----|--------|--------|------------|---------------|------|
| **ADR-0001** | Fundação da Fase 0 | ATIVA — 2026-08-17 | — | — | [ADR-0001-fase0.md](ADR-0001-fase0.md) |
| **ADR-0002** | Rejeição de Hayabusa e Chainsaw como componentes do produto | ACEITA — 2026-08-17 | — | — | [ADR-0002-rejeicao-hayabusa-chainsaw.md](ADR-0002-rejeicao-hayabusa-chainsaw.md) |
| **ADR-0003** | Separação entre RBAC e visibilidade de dados | ACEITA — 2026-08-17 | — | — | [ADR-0003-rbac-visibilidade-dados.md](ADR-0003-rbac-visibilidade-dados.md) |
| **ADR-0004** | Repositories finos com SQLx | ATIVA — 2026-08-17 | — | — | [ADR-0004-repositories-finos-sqlx.md](ADR-0004-repositories-finos-sqlx.md) |
| **ADR-0005** | Stack híbrida por classes de runtime | PARTIALLY SUPERSEDED — 2026-08-17 | — | ADR-0017 (parcial) | [ADR-0005-stack-hibrida-runtime-build-tooling.md](ADR-0005-stack-hibrida-runtime-build-tooling.md) |
| **ADR-0006** | Engine fino sobre `evtx` e protocolo compartilhado | ATIVA — 2026-08-18 | — | — | [ADR-0006-engine-evtx-e-protocolo.md](ADR-0006-engine-evtx-e-protocolo.md) |
| **ADR-0007** | Simplificação para ferramenta forense local | ACEITA — 2026-08-17 | — | — | [ADR-0007-local-forensic-scope-simplification.md](ADR-0007-local-forensic-scope-simplification.md) |
| **ADR-0008** | Modelo analítico comum e versionamento de resultados (F1-01) | ACEITA — 2026-08-18 | — | — | [ADR-0008-modelo-analitico-comum.md](ADR-0008-modelo-analitico-comum.md) |
| **ADR-0009** | Substituição de Bun por Node.js/npm na toolchain de build/test do frontend | SUPERSEDED BY ADR-0017 — 2026-09-22 | — | ADR-0017 | [ADR-0009-substituicao-bun-por-node-npm.md](ADR-0009-substituicao-bun-por-node-npm.md) |
| **ADR-0010** | YARA-X e ruleset curado (F1-02) | ACEITA — 2026-08-18 | — | — | [ADR-0010-yara-x-ruleset-curado.md](ADR-0010-yara-x-ruleset-curado.md) |
| **ADR-0011** | NIST Current/Target Profile e Gap Analysis reproduzível (F1-03) | ACEITA — 2026-08-27 | — | — | [ADR-0011-nist-profile-gap.md](ADR-0011-nist-profile-gap.md) |
| **ADR-0012** | ATT&CK mapping, caminho crítico e matriz de impacto (F1-04) | ACEITA — 2026-08-28 | — | — | [ADR-0012-attack-caminho-critico-impacto.md](ADR-0012-attack-caminho-critico-impacto.md) |
| **ADR-0013** | IOCs e recomendações operacionais (F1-05) | ACEITA — 2026-08-28 | — | — | [ADR-0013-ioc-recomendacoes.md](ADR-0013-ioc-recomendacoes.md) |
| **ADR-0014** | Default de `data_root` por plataforma | ACEITA — 2026-08-28 | — | — | [ADR-0014-data-root-por-plataforma.md](ADR-0014-data-root-por-plataforma.md) |
| **ADR-0015** | `engine_path` resolvido a partir do executável do `dfir-app` | ACEITA — 2026-09-21 | — | — | [ADR-0015-engine-path-relativo-ao-executavel.md](ADR-0015-engine-path-relativo-ao-executavel.md) |
| **ADR-0016** | Engine em módulos e `NormalizedEvent` tipado no `dfir-protocol` | ACEITA — 2026-09-21 | — | — | [ADR-0016-engine-em-modulos-e-normalized-event.md](ADR-0016-engine-em-modulos-e-normalized-event.md) |
| **ADR-0017** | Interface nativa (eframe/egui/egui_extras) substitui o par Axum + `frontend/` TypeScript | ACEITA — 2026-09-22 | ADR-0005 (parcial), ADR-0009 (toolchain) | — | [ADR-0017-interface-nativa-eframe-egui.md](ADR-0017-interface-nativa-eframe-egui.md) |
| **ADR-0018** | Realinhamento à demanda original (SENAI 12412): poda de escopo e adições necessárias | ACEITA — 2026-10-02 | ADR-0017 (classificação), RF-012/025/026/032/041/045/046 (parcial) | ADR-0030 (parcial) | [ADR-0018-realinhamento-demanda-original.md](ADR-0018-realinhamento-demanda-original.md) |
| **ADR-0019** | CLI headless fino sobre a camada de serviço (sem TUI) | ACEITA — 2026-10-03 | — | — | [ADR-0019-cli-headless-fino.md](ADR-0019-cli-headless-fino.md) |
| **ADR-0020** | Sem limite de saída e sem truncamento silencioso | ACEITA — 2026-10-04 | — | — | [ADR-0020-sem-limite-de-saida-e-truncamento.md](ADR-0020-sem-limite-de-saida-e-truncamento.md) |
| **ADR-0021** | Timeline no banco, guarda de espaço em disco e avisos do parser | ACEITA — 2026-10-11 | — | — | [ADR-0021-timeline-no-banco-guarda-de-disco-e-avisos.md](ADR-0021-timeline-no-banco-guarda-de-disco-e-avisos.md) |
| **ADR-0022** | Testes de GUI com `egui_kittest` e observabilidade (logs em arquivo, spans, diagnóstico) | ACEITA — 2026-10-11 | — | — | [ADR-0022-testes-de-gui-com-kittest-e-observabilidade.md](ADR-0022-testes-de-gui-com-kittest-e-observabilidade.md) |
| **ADR-0023** | Tipos `FINDING`, `ROOT_CAUSE` e `PLAN_ITEM`, migration 0010 e plano de conclusão do protótipo | ACEITA — 2026-10-13 | — | — | [ADR-0023-tipos-finding-root-cause-plan-item-e-migration-0010.md](ADR-0023-tipos-finding-root-cause-plan-item-e-migration-0010.md) |
| **ADR-0024** | Contenção do engine (F1-22) e terminologia de "sandbox" | ACEITA — 2026-10-27 | — | — | [ADR-0024-contencao-do-engine-e-terminologia-de-sandbox.md](ADR-0024-contencao-do-engine-e-terminologia-de-sandbox.md) |
| **ADR-0025** | Evento normalizado (F1-26), detecção por regras com confirmação (F1-28) e escopo EVTX (F1-27) | ACEITA — 2026-10-27 | — | — | [ADR-0025-eventos-normalizados-deteccao-por-regras-e-escopo-evtx.md](ADR-0025-eventos-normalizados-deteccao-por-regras-e-escopo-evtx.md) |
| **ADR-0026** | Cópia verificada do banco antes de migration (RF-032) | ACEITA — 2026-10-27 | — | — | [ADR-0026-copia-verificada-do-banco-antes-de-migration.md](ADR-0026-copia-verificada-do-banco-antes-de-migration.md) |
| **ADR-0027** | Harness de agente e remoção do Aider | ACEITA — 2026-11-04 | — | — | [ADR-0027-harness-de-agente-e-remocao-do-aider.md](ADR-0027-harness-de-agente-e-remocao-do-aider.md) |
| **ADR-0028** | SBOM fora do escopo | ACEITA — 2026-11-09 | RF-024, RF-031, seção 41.1, F.12 item 2 (spec 6.4.1, parcialmente) | — | [ADR-0028-sbom-fora-do-escopo.md](ADR-0028-sbom-fora-do-escopo.md) |
| **ADR-0029** | Reavaliação da interface e manutenção de eframe/egui | ACEITA — 2026-11-18 | — | — | [ADR-0029-manter-eframe-egui.md](ADR-0029-manter-eframe-egui.md) |
| **ADR-0030** | Reintrodução delimitada de melhorias de inspeção, integridade e operação | ACEITA — 2026-11-20 | ADR-0018 D-2 (parcial: assinatura, âncora, SVG, breadcrumbs, cancelamento) | ADR-0031 (somente recovery) | [ADR-0030-melhorias-inspecao-integridade-e-operacao.md](ADR-0030-melhorias-inspecao-integridade-e-operacao.md) |
| **ADR-0031** | Propriedade de jobs por lock exclusivo e recuperação multiinstância | ACEITA — 2026-11-24 | ADR-0020 item 6; ADR-0030 D-4 (somente recovery) | — | [ADR-0031-ownership-de-jobs-por-lock.md](ADR-0031-ownership-de-jobs-por-lock.md) |

---

## Resumo de Relações

### Supersede (substitui parcial ou totalmente)

- **ADR-0005** → partially superseded by **ADR-0017** (classe "interface web rica" removida; Production Runtime e Engineering/QA Tooling permanecem)
- **ADR-0009** → superseded by **ADR-0017** (toolchain Node/npm/frontend removida junto com `frontend/`)
- **ADR-0017** → supersedes **ADR-0005** (parcial) e **ADR-0009** (toolchain)
- **ADR-0018** → supersedes classificação de **ADR-0017** e RFs 012/025/026/032/041/045/046 (parcial)
- **ADR-0028** → supersedes parcialmente RF-024, RF-031, seção 41.1, F.12 item 2 da spec 6.4.1
- **ADR-0030** → supersedes parcialmente **ADR-0018** D-2 (apenas 5 itens listados)
- **ADR-0031** → supersedes **ADR-0020** item 6 e **ADR-0030** D-4 somente quanto à recuperação multiinstância

### Superseded-by (substituído por)

- **ADR-0005** → superseded-by **ADR-0017** (parcial)
- **ADR-0009** → superseded-by **ADR-0017**
- **ADR-0018** → superseded-by **ADR-0030** (parcial)
- **ADR-0020** → superseded-by **ADR-0031** (somente item 6)
- **ADR-0030** → superseded-by **ADR-0031** (somente recovery)

---

## Notas de Navegação

- **Links relativos** válidos a partir de `docs/adrs/`.
- **Status** reflete a metadata e os avisos explícitos de supersessão no topo; o campo histórico `## Status` é preservado mesmo quando diverge.
- **Supersedes/Superseded-by** vêm de metadata e notas explícitas, sem inferir novas relações.
- **Histórico preservado**: ADRs com status `partially-superseded` ou `superseded` permanecem no índice; não são movidos nem arquivados (ADR-0018 D-2, ADR-0028).
- **Evidências**: vinculadas apenas quando já existem nos próprios ADRs (campos `Evidência de execução`, `Evidência desta reavaliação`, `Verificação`). Nenhuma execução retroativa nem novo `PASS` criado.

---

## Contagem

- **Total:** 31 ADRs (ADR-0001 a ADR-0031)
- **Sem metadata de supersessão:** 29
- **Partially Superseded:** 1 (ADR-0005)
- **Superseded:** 1 (ADR-0009)

---

*Gerado em 2026-11-24 como parte da Prioridade 4 do `docs/FINAL_ANALYSIS_CHECKLIST.md`.*