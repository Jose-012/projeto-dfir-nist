<!-- doc-type: backlog | status: current -->
# Fase 1 — Backlog do MVP DFIR (realinhado à demanda original)

**Status:** REBASELINED em 2026-10-02 (ADR-0018). Substitui o backlog F1-01..F1-12 de 2026-08-24.
**Escopo:** somente requisitos classificados como MVP na especificação canônica v6.5.1. Fase 4/5 não bloqueiam este backlog.
**Fonte de verdade da demanda:** `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md`; rastreabilidade em `docs/DEMAND_TRACE.md`.

## 1. Objetivo

Levar a ferramenta local/offline ao ponto em que um analista, a partir de evidência preservada, produz os seis requisitos de entrega da demanda: relatório forense, vulnerabilidades exploradas, maturidade NIST CSF 2.0, recomendações, plano de hardening e plano de criptografia e proteção de dados.

A base existente cobre preservação, custódia, engine separado, JSONL, EVTX e timeline (bloco A da demanda: investigar). O bloco B (transformar a investigação em plano de fortalecimento) não tinha modelo de dados; o backlog abaixo o constrói e corrige os defeitos encontrados pela auditoria de 2026-10-01/02.

## 2. Estado de partida (honesto)

- Os gates Rust rodaram de verdade em 2026-09-21 (rustc 1.97: `fmt`, `clippy -D warnings`, `test`, `cargo deny`) e em 2026-09-22 (rustc 1.91.1, crate de aplicação: 93/93 testes). As mudanças de 2026-09-26 (egui 0.36.2, `reject_symlink`, fechamento de gap) **não foram compiladas** por seus autores; em 2026-10-02 a primeira compilação delas revelou o lock dessincronizado e um erro de empréstimo (E0505) em `nist_profiles.rs`, ambos corrigidos em cópia de trabalho e pendentes de confirmação com rustc 1.97.1 (F1-13).
- F1-01..F1-05: `IMPLEMENTED / PENDING-HUMAN`. Nenhum está `VERIFIED` (F1-14).
- Defeitos conhecidos (AUD-xx) estão distribuídos nos tickets abaixo; cada ticket cita sua origem.

## 3. Ordem de execução

| Onda | Ticket | Entrega principal | Bloqueado por |
|---|---|---|---|
| 0 | F1-13 | Lock regenerado + compilação corrigida | Nenhum |
| 0 | F1-14 | Baseline de verificação e CI mínimo | F1-13 |
| 0 | F1-15 | Supply chain da árvore de UI | F1-13 |
| 0 | F1-23 | Erros acionáveis; auditoria sem `let _ =` | Nenhum |
| 1 | F1-16 | Fixtures EVTX e testes de integração | F1-13 |
| 1 | F1-17 | Erros de parsing persistidos (RF-006) | F1-16 |
| 1 | F1-26 | Evento realmente normalizado | F1-16 |
| 1 | F1-18 | Timeline persistida, ordenada e filtrável | F1-26 |
| 1 | F1-19 | Vigentes sem janela, YARA idempotente, vínculo evidência↔incidente | F1-16 |
| 1 | F1-20 | YARA `wide` e limites | F1-16 |
| 1 | F1-21 | Destruição de evidência: ordem segura | F1-16 |
| 1 | F1-22 | Isolamento do engine | F1-16 |
| 1 | F1-10 | Robustez reduzida: recovery, shutdown, espaço, backup de migration | F1-16 |
| 1 | F1-24 | Custódia e auditoria visíveis | F1-16 |
| 1 | F1-33 | Correções de estado/UX da GUI | F1-18 |
| 2 | F1-27 | Importador estruturado CSV/JSONL | F1-26 |
| 2 | F1-25 | Finding, causa-raiz e lacunas de conclusão | F1-19 |
| 2 | F1-28 | Detecção determinística + extração de IOC | F1-26, F1-25 |
| 2 | F1-30 | Visão NIST por Function | F1-19 |
| 2 | F1-31 | Proveniência, cobertura e lacunas | F1-19 |
| 2 | F1-29 | Item de plano | F1-25 |
| 2 | F1-06 | Plano de hardening (sem Ansible) | F1-29, F1-25 |
| 3 | F1-07 | Entregáveis: um documento por entregável | F1-25, F1-29, F1-30, F1-06, F1-32 |
| 3 | F1-32 | Redação de dados pessoais | F1-07 |
| 3 | F1-11 | Dataset BEC e testes de segurança | F1-27, F1-28, F1-29, F1-07 |
| 4 | F1-12 | Release candidate e reconciliação | demais tickets MVP |
| 0 | F1-34 | CLI headless fina e teste ponta a ponta (ADR-0019) | Nenhum — **já implementado**, pendente de verificação no toolchain pinado |
| 0 | F1-35 | Observabilidade e testes de GUI com `egui_kittest` (ADR-0022) | Nenhum — **já implementado**, pendente de verificação no toolchain pinado |
| — | F1-08 | `.dfirpkg` | **ADIADO — Fase 4** |
| — | F1-09 | Atualização offline assinada | **ADIADO — Fase 4** |

Tickets F1-01..F1-05 e F1-34 permanecem como entregues (`IMPLEMENTED / PENDING-HUMAN`) e entram em F1-14 para verificação.

**Caminho mínimo para um MVP defensável:** F1-13 → F1-16 → F1-26 → F1-18 → F1-27 → F1-28 → F1-25 → F1-29 → F1-07. Esse caminho produz, de ponta a ponta, o conteúdo dos seis requisitos de entrega a partir de evidência. Os demais tickets são suporte e correção.

**Plano de conclusão do protótipo (ADR-0023, 2026-10-13):** por decisão do operador o projeto é protótipo, e formatos além de EVTX ficam fora desta fase. A ordem de execução passa a ser: migration 0010 e tipos (F1-25/F1-29, feito) → F1-25 (feito em 2026-10-14) → F1-29 (feito em 2026-10-14; absorve F1-06) → F1-30 (feito em 2026-10-15) → F1-07 (feito em 2026-10-16, parcial: sem redação e sem lacunas de proveniência) → F1-31/F1-32 mínimos (feitos em 2026-10-18) → dataset e fechamento (F1-11 feito em 2026-10-23, reduzido; F1-12 `PARCIAL`: documentos reconciliados, RF-032 automático e F1-26..F1-28 pendentes). F1-26, F1-27 e F1-28 seguem `PLANNED` e fora desse caminho; a especificação continua os classificando como MVP.

> **Nota (2026-11-04) — Gap origem: INC-11.** O trecho acima, e o caminho mínimo da linha 57 que cita F1-27, estão superados. Estado vigente (atualização de 2026-10-27, mais abaixo neste arquivo): F1-26 e F1-28 `IMPLEMENTED / PENDING-HUMAN`; F1-27 `DEFERRED` (o escopo do protótipo é EVTX; ADR-0025). A autoridade de estado de ticket é `docs/rtm.md`.

## 4. Cobertura dos RFs

- Fundação já implementada (aguardando verificação em F1-14): `RF-001` a `RF-007`, `RF-027` a `RF-029`, `RF-031`, `RF-033` a `RF-036`, `RF-038`.
- Fase 1 original (F1-01..F1-05): `RF-008` a `RF-011`, `RF-018` a `RF-020`, `RF-022`, `RF-039`.
- Novos (ADR-0018): `RF-048` (F1-26), `RF-049` (F1-27), `RF-050` (F1-28), `RF-051` (F1-25), `RF-052` (F1-29, F1-06), `RF-053` (F1-30), `RF-054` (F1-31), `RF-055` (F1-32).
- Entregáveis: `RF-013`, `RF-040`, `RF-046` (F1-07, F1-18).
- Operacionais reduzidos: `RF-032`, `RF-042`, `RF-043` (F1-10); `RF-030`, `RF-047` (F1-12, F1-13).
- Substituído: `RF-041` (retry/DLQ) → falha inspecionável (F1-10, F1-23).

## 5. Fora do caminho crítico (Fase 4)

`RF-012` (Ansible), `RF-021` (Sigma), `RF-023` (Velociraptor), `RF-025` (`.dfirpkg`), `RF-026` (atualização offline), `RF-045` (benchmark). Melhorias descartadas do MVP sem ticket: assinatura de relatório, âncora extra de custódia, `cargo vet`, Job Object/limite de memória no Windows, grafo SVG do caminho crítico, breadcrumbs, cancelamento de job.

> **Nota (2026-11-04) — Gap origem: INC-11.** "Job Object/limite de memória no Windows" saiu do MVP pela ADR-0018 D-2 e depois foi implementado: ADR-0024 (complementa a ADR-0018) e F1-22 (Job Object compilado para Windows; verificação em Windows `PENDING-HUMAN`). Ver a atualização de 2026-10-27 mais abaixo.

## 6. Definição de pronto

A Fase 1 estará concluída quando todos os tickets da tabela (exceto `ADIADO`) estiverem `ACCEPTED`, os `RF-*` MVP tiverem rastreabilidade na RTM, `docs/DEMAND_TRACE.md` não tiver item `PLANEJADO` sem justificativa e o release candidate satisfizer os gates de aceitação da especificação.

## Atualização 2026-10-27 — F1-22, F1-26, F1-28 implementados; F1-27 adiado

Decisões em `docs/adrs/ADR-0024` (contenção do engine), `ADR-0025` (evento normalizado, detecção, escopo EVTX) e `ADR-0026` (cópia verificada antes de migration). Estado dos tickets desta rodada:

| Ticket | Estado | O que mudou |
|---|---|---|
| F1-22 | `IMPLEMENTED / PENDING-HUMAN` | `env_clear` + ambiente mínimo, arquivo único imposto, `kill_on_drop`, Landlock + seccomp + `no_new_privs` (Linux, provados com o engine real), Job Object (Windows, só compilado). Token restrito/AppContainer: não feito |
| F1-26 | `IMPLEMENTED / PENDING-HUMAN` | campos estruturais opcionais no evento, esquema 4, migration 0012, filtro `campo:valor` na timeline e na CLI |
| F1-27 | `DEFERRED` | importador CSV/JSONL adiado por decisão do operador: o escopo do protótipo é EVTX |
| F1-28 | `IMPLEMENTED / PENDING-HUMAN` | 19 regras, candidatos pendentes, confirmar (mapeamento + achado + IOCs) ou rejeitar com motivo; Entregável 1 §4.3 |
| F1-10 | `IMPLEMENTED / PENDING-HUMAN` | RF-032 automático (ADR-0026) |

Dependências: F1-11 listava F1-27 como bloqueador; com F1-27 adiado, o dataset BEC continua sendo o cenário sintético registrado à mão sobre um EVTX real, e agora roda também as regras de detecção (candidatos ficam pendentes na demonstração).

## 7. Plano remanescente (concluído o planejamento em 2026-11-15)

Ponto de partida medido (2026-11-14/15): `just verify` **410/410** (`EVD-005`), e2e **110/110** (`EVD-006`), `cargo audit` 0 vulnerabilidades e `cargo deny --workspace check` verde (`EVD-007`), doc-gate `PASS` (`EVD-008`), janela real da GUI aberta (`EVD-009`). Waivers `WVR-001`..`WVR-004` aprovados até **2026-12-31**. O que resta está organizado nas pistas abaixo; a ordem entre pistas é livre, mas **A** e **B** são as únicas que dependem de infraestrutura externa ao container de desenvolvimento.

| Pista | Escopo | Tickets / artefatos | Critério de saída |
|---|---|---|---|
| **A. Verificação externa** (operador) | Windows e aarch64 reais; incidente real | `WVR-001`, `WVR-002`, F1-22, F1-12 | `dfir-app engine-smoke` + `just e2e <evtx>` em Windows com o aviso `SANDBOX_NOT_FULLY_APPLIED`; build+e2e em aarch64; um caso real analisado com `EVD-*` registrado. Fecha os waivers; se não ocorrer até 2026-12-31, renovar com nova aprovação |
| **B. CI automático** (F1-14) | Pipeline no toolchain pinado 1.100.0 | `WVR-003`, F1-14 | `just verify` + e2e com fixture em CI; `WVR-003` passa a `satisfied` |
| **C. Fixtures EVTX sanitizadas** (F1-16) | `.evtx` mínimo, determinístico e versionado com SHA-256 | F1-16; desbloqueia F1-17..F1-22 na verificação | **CONCLUÍDA 2026-11-15 (`EVD-011`)**: `tests/fixtures/senai-demo.evtx` determinística, e2e 110/110 sem arquivo externo; `docs/OPERATION.md` §5 atualizado |
| **D. Lacunas funcionais parciais** | Vínculo evidência↔incidente (F1-19), limites YARA (F1-20), ordem/retomada da destruição (F1-21), auditoria de integridade (F1-23), custódia/auditoria visíveis (F1-24), AUD-32 (F1-33) | F1-19, F1-20, F1-21, F1-23, F1-24, F1-33 | **CONCLUÍDA 2026-11-15 (`EVD-010`)**: cada ticket com teste próprio (nextest 424/424, suíte `gui::` 62/62) e `EVD-010` registrado |
| **E. Supply chain recorrente** | Procedimento de `AGENTS.md` §8 a cada rodada; reavaliar `WVR-004` quando o `sqlx` admitir `libsqlite3-sys >= 0.39.0`; revisar `RUSTSEC-2025-0141` a cada upgrade do `yara-x` | `deny.toml`, `.cargo/audit.toml`, `docs/SUPPLY_CHAIN.md` | exceções vivas apenas com justificativa + data de revisão; `WVR-004` fechado ou renovado até 2026-12-31 |
| **F. Fechamento do release candidate** | Promover `VERIFIED`/`ACCEPTED` o que tiver evidência; declarar RC | F1-12, `docs/RELEASE_GATE.md` | nenhuma linha de RF MVP em `PENDING`/`PLANNED`/`PARCIAL` sem waiver formal |

Fora deste plano por decisão registrada: F1-08/F1-09 (Fase 4, ADR-0018), F1-27 (ADR-0025) e os itens listados em `docs/FASE1_BACKLOG.md` §5.

