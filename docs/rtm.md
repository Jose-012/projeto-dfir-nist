<!-- doc-type: status | status: current -->
# RTM - ferramenta forense local

> **Autoridade única de estado de ticket: este documento.** `docs/FASE1_STATUS.md` guarda somente o resumo cronológico por rodada e não deve duplicar a matriz. (Decisão original de 2026-11-04; simplificação de 2026-11-18, Gap origem: INC-03/INC-13.)

## Baseline Fase 0 — capacidades já aceitas

A implementação abaixo pertence à base já concluída e não redefine os `RF-*` da especificação canônica. O rastreamento normativo dos requisitos do MVP usa os IDs da seção 39 da especificação.

| Capacidade | Implementação | Validação | Estado |
|---|---|---|---|
| Arquitetura modular | `src/`, `engine/`, `protocol/` | build/test/gates | ACCEPTED |
| SQLite/migrations | `src/persistence/db.rs`, `migrations/` | migração + testes | ACCEPTED |
| Audit chain | `src/persistence/audit.rs`, `src/persistence/jcs.rs` | testes de integridade/tamper e vetores fixos v1/v2 | ACCEPTED |
| Storage/staging | `src/infrastructure/storage.rs` | testes adversariais | ACCEPTED |
| Engine separado | `src/application/engine.rs`, `engine/` | subprocesso real | ACCEPTED |
| JSONL | `protocol/`, `src/application/engine.rs` | protocolo/smoke | ACCEPTED |
| Evidência + hash + custódia | `src/service/mod.rs` (`import_evidence_with_provenance`), migrations 0004/0005 | `EVD-010`/`EVD-012`: custódia visível, importação, SHA-256 e `audit-verify`; RF-003 com preservação byte a byte | ACCEPTED |
| EVTX + timeline | `engine/src/main.rs`, `src/service/mod.rs` (`job_results`), `src/gui/evidence.rs` | `EVD-011`/`EVD-012`: fixture EVTX sanitizada versionada, e2e 110/110, timeline persistida e filtrável | ACCEPTED |
| Descarte governado local | `src/service/mod.rs` (`destroy_evidence`) + audit/custody | `EVD-010`/`EVD-011`/`EVD-012`: ordem registro→arquivo→jobs, retomada, purga e preservação da custódia | ACCEPTED |

> Estas três linhas ficaram `STALE` em 2026-10-02 porque a evidência apontava para código removido ou comportamento reescrito pela ADR-0017. Voltaram a `ACCEPTED` em 2026-11-18 somente após as evidências executadas exigidas por esta regra: F1-14/`EVD-012`, F1-16/`EVD-011` e F1-21/`EVD-010`.

## RTM Fase 1 — backlog do MVP (rebaselined em 2026-10-02, ADR-0018)

> Vista por ticket; ver também a vista por RF (seção "RTM Fase 1 — vista por RF", abaixo). Gap origem: INC-15 (2026-11-04).

| Ticket | RFs canônicos | Entrega | Estado |
|---|---|---|---|
| F1-01 | Enabler (sem RF próprio) | Modelo analítico comum/provenance/versionamento | IMPLEMENTED / PENDING-HUMAN |
| F1-02 | RF-008 | YARA-X e resultados estruturados | IMPLEMENTED / PENDING-HUMAN |
| F1-03 | RF-009, RF-010 | Current/Target Profile + Gap reproduzível | IMPLEMENTED / PENDING-HUMAN (E0505 corrigido em 2026-10-02) |
| F1-04 | RF-011, RF-018, RF-020 | ATT&CK + caminho crítico + impacto | IMPLEMENTED / PENDING-HUMAN |
| F1-05 | RF-019, RF-022 | IOCs + recomendações | IMPLEMENTED / PENDING-HUMAN |
| F1-06 | RF-052 (RF-012 → Fase 4) | Plano de hardening (item de plano, sem Ansible) | IMPLEMENTED / PENDING-HUMAN (absorvido por F1-29, domínio `OS_HARDENING`; a saída no Entregável 4 é gerada por F1-07) |
| F1-07 | RF-013, RF-040, RF-046 | Entregáveis: um documento por entregável | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-08 | RF-025 | `.dfirpkg` | DEFERRED — Fase 4 |
| F1-09 | RF-026 | Atualização offline assinada | DEFERRED — Fase 4 |
| F1-10 | RF-032, RF-042, RF-043 (RF-041 substituído) | Robustez reduzida | IMPLEMENTED / PENDING-HUMAN (ADR-0026: cópia verificada do banco antes de migration, automática e testada; AUD-03a e recovery na partida já existiam) |
| F1-11 | RF-016, RF-017, RF-046 (RF-045 → Fase 4) | Dataset BEC e testes de segurança | IMPLEMENTED / PENDING-HUMAN (reduzido: ADR-0023, Etapa 7; só EVTX, sem CSV/JSONL; testado em rustc 1.97.1 em 2026-10-27) |
| F1-12 | RF-014, RF-030, RF-032, RF-047 + RFs MVP restantes | Release candidate e reconciliação | PARCIAL (documentos reconciliados em 2026-10-27; RF-032 automático; F1-26 e F1-28 implementados; RF-049 adiado por decisão do operador; **pendentes:** CI no toolchain pinado (F1-14), ~~`cargo audit` (F1-15)~~ [executado em 2026-10-29, ver a linha F1-15; nota de 2026-11-04, Gap origem: INC-03], execução em Windows e em incidente real) |
| F1-13 | RF-047 | Lock regenerado e compilação corrigida | IMPLEMENTED / PENDING-HUMAN |
| F1-14 | (gates) | Baseline de verificação e CI | PARCIAL (2026-11-18: `just verify` com 446 passed/1 skipped em **1.100.0**, `EVD-023`; e2e 110/110 em `EVD-023`; CI automático não implementado e `WAIVED` por `WVR-003` até 2026-12-31) |
| F1-15 | (supply chain) | Supply chain da árvore de UI | PARCIAL (2026-11-18: `cargo audit` e `cargo deny --workspace check` verdes em `EVD-020`; uma vulnerabilidade aceita por `WVR-004` e duas exceções informativas documentadas; manual, sem CI (`WVR-003`); `cargo vet` dispensado no MVP; exceções informativas da UI removidas do grafo com eframe 0.37.0 sem accesskit) |
| F1-16 | RF-005, RF-006 | Fixtures EVTX e testes de integração | VERIFIED (2026-11-15, `EVD-011`): fixture EVTX sanitizada versionada `tests/fixtures/senai-demo.evtx` (determinística) e e2e 110/110 sem arquivo externo |
| F1-17 | RF-006 | Erros de parsing persistidos | IMPLEMENTED / PENDING-HUMAN (ADR-0021; `EVD-021`: warnings persistidos com JSON inválido ou contagem negativa/divergente falham em vez de virar lista vazia) |
| F1-18 | RF-007, RF-046 | Timeline persistida e filtrável | IMPLEMENTED / PENDING-HUMAN (ADR-0020/0021/0025; filtros estruturados `campo:valor` implementados) |
| F1-19 | RF-039 | Vigentes sem janela, YARA idempotente, vínculo | VERIFIED (2026-11-15, `EVD-010`): vínculo evidência↔incidente idempotente com auditoria `EVIDENCE_LINKED_TO_INCIDENT`, vigente sem janela, YARA idempotente; GUI + CLI (`link-evidence`, `incident-evidence`) com testes próprios |
| F1-20 | RF-008 | YARA `wide` e limites | VERIFIED (2026-11-15, `EVD-010`): `wide ascii` provado em UTF-16LE, `scan_timeout_for` por tamanho com teto, `MAX_MATCHES_PER_PATTERN` sem descarte silencioso, sem truncamento de varredura; testes em `engine/src/yara_scan.rs` |
| F1-21 | RF-004 | Destruição de evidência: ordem segura | VERIFIED (2026-11-15, `EVD-010`): ordem registro→arquivo→jobs com retomada em reexecução; testes cobrindo falha de banco (nada apagado), falha de I/O (registro mantido) e purga completa incl. candidatos de detecção, custódia preservada |
| F1-22 | RF-033, RF-037 | Isolamento do engine | IMPLEMENTED / PENDING-HUMAN (ADR-0024: ambiente reduzido, arquivo único, `kill_on_drop`, Landlock + seccomp + `no_new_privs` no Linux, **provados com o engine real**; Job Object no Windows **só compilado, nunca executado**; confinamento de arquivo/rede no Windows não feito) |
| F1-23 | RF-006 | Erros acionáveis | VERIFIED (2026-11-15, `EVD-010`): nenhum `let _ =` em código de produção fora de testes; `rollback_loudly`, `discard_temp_file/dir`, `mark_job_failed_loudly` com `tracing::warn!` e testes próprios; sends de canal logados; `EVD-023`: `payload`/`provenance` analíticos inválidos produzem diagnóstico específico em vez de `null` |
| F1-24 | RF-004, RF-029 | Custódia e auditoria visíveis | VERIFIED (2026-11-15, `EVD-010`): painel de custódia no detalhe da evidência e abas Evidências/Auditoria em Incidentes na GUI; CLI `custody` e `audit`; testes `egui_kittest` 62/62 na suíte `gui::`; `EVD-022`: metadata inválido em `list_custody`/`list_events` agora falha em vez de virar `null` |
| F1-25 | RF-051 | Finding, causa-raiz e lacunas | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-26 | RF-048 | Evento normalizado | IMPLEMENTED / PENDING-HUMAN (ADR-0025; esquema 4, migration 0012, campos estruturais e filtro; testado em rustc 1.97.1 em 2026-10-27) |
| F1-27 | RF-049 | Importador CSV/JSONL | DEFERRED — decisão do operador (ADR-0025): escopo do protótipo é EVTX |
| F1-28 | RF-050 | Detecção determinística + IOC | IMPLEMENTED / PENDING-HUMAN (ADR-0025; 19 regras, candidatos com confirmação/rejeição motivada, entregável 1 §4.3; `EVD-020`: `event_refs`/IOCs com JSON inválido falham antes de qualquer escrita analítica; verificado no `security.evtx` real e em eventos sintéticos, **não em incidente real**) |
| F1-29 | RF-052 | Item de plano | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-30 | RF-053 | NIST por Function | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-31 | RF-054 | Proveniência, cobertura, lacunas | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-32 | RF-055 | Redação de dados pessoais | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-33 | RF-046 | Estado da GUI/UX | VERIFIED (2026-11-15, `EVD-010`): AUD-32 corrigido — YARA não sobrescreve a timeline (só análise EVTX recarrega eventos); teste próprio em `src/gui/evidence.rs` |
| F1-34 | Enabler (sem RF próprio; ADR-0019) | CLI headless fina e teste ponta a ponta | IMPLEMENTED / PENDING-HUMAN |
| F1-35 | Enabler (sem RF próprio; ADR-0022) | Observabilidade e testes de GUI | IMPLEMENTED / PENDING-HUMAN |

Backlog detalhado: `docs/FASE1_BACKLOG.md`. Tickets locais: `.scratch/fase1-backlog/issues/`. Rastreabilidade demanda → RF → ticket: `docs/DEMAND_TRACE.md`.

## RTM Fase 1 — vista por RF (13 RFs MVP sem linha na vista por ticket)

> Gap origem: INC-15 (2026-11-04). Cobre `RF-001`, `RF-002`, `RF-003`, `RF-015`, `RF-027`, `RF-028`, `RF-034`, `RF-035`, `RF-036`, `RF-038` e `RF-044` (mais `RF-024` e `RF-031`, que voltaram à tabela da spec 6.5.1 em 2026-11-09, ADR-0028); `RF-015` e `RF-044` não tinham rastro em RTM, DEMAND_TRACE, backlog nem tickets. As demais RFs MVP seguem pela vista por ticket (coluna "RFs canônicos"). Mapeamento feito por leitura do código e dos nomes dos testes (todo arquivo e teste citado existe); o conteúdo exato de cada asserção não foi reauditado. Desde 2026-11-14 existem `EVD-001` a `EVD-004` em `docs/EVIDENCE_REGISTER.md`; onde a execução cobre a RF, a coluna EVD cita o ID. O estado continua `IMPLEMENTED / PENDING-HUMAN` porque Windows, aarch64 e incidente real não foram executados.

| RF | Implementação | Teste | EVD | Estado |
|---|---|---|---|---|
| RF-001 — Upload (UUID e Evidence) | `src/service/mod.rs` (`import_evidence_with_provenance`, `import_evidence_classified`); `src/infrastructure/storage.rs` | `src/service/provenance.rs::import_with_provenance_stores_and_returns_what_was_declared`; `src/service/demo.rs::the_evidence_is_classified_demo_and_the_seed_is_audited`; e2e: `scripts/e2e_headless.py` ("import: comando concluiu") | `EVD-002` | IMPLEMENTED / PENDING-HUMAN |
| RF-002 — Integridade (SHA-256 antes da análise) | `src/service/mod.rs` (`analyze`; falha de integridade) | `src/service/mod.rs::integrity_failure_is_recorded_in_audit_and_custody`; `src/service/mod.rs::integrity_failure_that_cannot_be_recorded_says_so`; e2e: SHA-256 da evidência igual ao do arquivo | `EVD-002` | IMPLEMENTED / PENDING-HUMAN |
| RF-003 — Preservação (original não alterado) | `src/infrastructure/storage.rs`; `src/service/mod.rs` (`import_evidence_classified`) | `src/service/provenance.rs::import_leaves_the_source_file_byte_for_byte_untouched` (2026-11-18: bytes, mtime e SHA-256 do original idênticos antes/depois da importação; cópia preservada com os mesmos bytes); e2e confere o hash gravado | `EVD-012` | IMPLEMENTED / PENDING-HUMAN |
| RF-015 — Auditoria | `src/persistence/audit.rs` | `src/persistence/audit.rs::custody_chain_verification_detects_tampering`; `src/persistence/audit.rs::concurrent_appends_preserve_one_auditable_chain`; `src/persistence/audit.rs::tampered_event_count_is_detected`; `src/persistence/audit.rs::tampering_with_audit_head_is_detected`; e2e: `audit-verify` íntegro | `EVD-001`, `EVD-002` | IMPLEMENTED / PENDING-HUMAN |
| RF-024 — Licenças | `LICENSE`; `LICENSES_THIRD_PARTY.md` (gerado de `cargo metadata --locked`, 659 pacotes); `scripts/validate_fase0.py` (guarda do texto da licença e das crates diretas no lock) | `[não no contexto]` — não há teste Rust; a guarda é estática; `cargo deny --workspace check` verifica as licenças da árvore | `EVD-001`, `EVD-003` | IMPLEMENTED / PENDING-HUMAN |
| RF-027 — Protocolo do engine (subprocesso + JSONL) | `src/application/process.rs`; `src/application/engine.rs`; `engine/src/output.rs` | `src/application/engine.rs::bounded_jsonl_reader_accepts_exact_limit_but_rejects_overflow`; `engine/src/output.rs::result_batch_is_one_jsonl_line_and_matches_the_sized_length`; `engine/src/output.rs::protocol_lines_end_with_a_newline`; ausência de endpoint TCP/Unix: `src/main.rs::no_network_listener_exists_in_app_or_engine_source` (2026-11-18, guarda estática sobre `src/`, `engine/src/`, `protocol/src/`); runtime: `engine/tests/sandbox.rs` prova que socket TCP/Unix é EPERM sob seccomp | `EVD-001`, `EVD-002`, `EVD-012` | IMPLEMENTED / PENDING-HUMAN |
| RF-028 — Engine separado por processo | `engine/Cargo.toml` (sem `sqlx`); `engine/src/limits.rs`; `engine/src/runner.rs`; `src/application/process.rs` | `engine/src/runner.rs::yara_item_limit_is_enforced_before_anything_is_emitted`; `engine/src/output.rs::result_batch_over_max_output_bytes_is_rejected_without_writing`; `src/application/process.rs::the_limit_covers_the_mapped_input_file_without_overflowing`; "engine não acessa SQLite": fato do manifesto, teste `[não no contexto]` | `EVD-001`, `EVD-002` | IMPLEMENTED / PENDING-HUMAN |
| RF-031 — Supply chain | `Cargo.lock`; `justfile` (`supply-chain`, `audit`, `licenses`); `deny.toml` | `[não no contexto]` — são gates de comando, não testes; `cargo audit` e `cargo deny --workspace check` rodaram em 2026-11-14 | `EVD-003` | IMPLEMENTED / PENDING-HUMAN |
| RF-034 — Fontes de evidência | `src/service/provenance.rs`; `src/service/mod.rs` (`import_evidence_with_provenance`) | `src/service/provenance.rs::import_with_provenance_stores_and_returns_what_was_declared`; `src/service/provenance.rs::invalid_provenance_is_refused_before_anything_is_copied`; `src/service/mod.rs::analyze_loads_the_evidence_row_before_validating_the_kind`; aviso explícito de análise do formato não implementada: `[não no contexto]` | `EVD-001`, `EVD-002` | IMPLEMENTED / PENDING-HUMAN |
| RF-035 — Contrato JSONL | `protocol/src/lib.rs` | `protocol/src/lib.rs::rejects_excessive_json_depth_without_parsing`; `protocol/src/lib.rs::rejects_traversal_and_oversized_lines`; `protocol/src/lib.rs::accepts_known_analysis_kinds_and_rejects_unknown_ones`; `protocol/src/lib.rs::errors_are_sanitized_and_bounded`; versão, tipo e `request_id`/`job_id`: `[não no contexto]` (sem teste nomeado) | `EVD-001`, `EVD-002` | IMPLEMENTED / PENDING-HUMAN |
| RF-036 — Isolamento fechado | `engine/src/sandbox.rs`; `src/config.rs` (`DFIR_ENGINE_SANDBOX`); `src/application/process.rs` | `engine/tests/sandbox.rs::required_policy_fails_closed_and_analyses_nothing_when_a_layer_cannot_be_applied`; `engine/tests/sandbox.rs::an_invalid_policy_value_is_treated_as_required_not_as_off`; `engine/tests/sandbox.rs::with_the_sandbox_off_the_same_probes_succeed_so_the_test_is_not_vacuous`; `src/config.rs::the_sandbox_policy_accepts_only_known_names_and_rejects_typos`; e2e: sandbox `required` sobre o EVTX real | `EVD-001`, `EVD-002` | IMPLEMENTED / PENDING-HUMAN |
| RF-038 — Auditoria operacional local | `src/persistence/audit.rs`; `src/service/mod.rs`; `src/service/deliverables/mod.rs` (`generate`) | `src/service/mod.rs::integrity_failure_is_recorded_in_audit_and_custody`; `src/service/demo.rs::the_evidence_is_classified_demo_and_the_seed_is_audited`; `src/service/deliverables/mod.rs::generate_reads_the_database_writes_the_files_and_audits_without_paths` | `EVD-001`, `EVD-002` | IMPLEMENTED / PENDING-HUMAN |
| RF-044 — Validação (entradas e envelope JSONL) | `protocol/src/lib.rs`; `src/application/engine.rs`; `src/gui/timeline.rs`; `src/service/provenance.rs` | `protocol/src/lib.rs::rejects_excessive_json_depth_without_parsing`; `protocol/src/lib.rs::rejects_traversal_and_oversized_lines`; `src/application/engine.rs::validates_logical_request_and_rejects_traversal`; `src/gui/timeline.rs::an_invalid_filter_shows_an_error_beside_the_box_and_keeps_the_timeline`; `src/service/provenance.rs::invalid_provenance_is_refused_before_anything_is_copied` | `EVD-001`, `EVD-002` | IMPLEMENTED / PENDING-HUMAN |

## Decisão de escopo

Autenticação, RBAC, sessões e server-mode não são requisitos da demanda original e não fazem parte do runtime atual. Ver `docs/adrs/ADR-0007-local-forensic-scope-simplification.md`.


## Atualização 2026-08-24
- `RF-007` foi corrigido para rastrear a implementação real da timeline EVTX.
- A evidência de compilação anterior que dizia `PASS` foi invalidada como `STALE` após a reprodução do `just check` no Windows em 2026-08-24.
- ~~`RF-011` cobre descarte local explícito sem apagar audit/custody.~~ **Corrigido em 2026-10-02:** `RF-011` é *ATT&CK*. O descarte governado local não tem RF próprio na seção 39; é capacidade de baseline ligada a `RF-004` (custódia) e é tratada em F1-21.

## Atualização 2026-08-25 — F1-01
- F1-01 (Enabler, sem RF próprio) implementado: `migrations/0006_analytical_results.sql` + `src/persistence/analytical_results.rs`. A coluna "Estado inicial" da tabela acima ("READY") não é reescrita retroativamente; o estado corrente é rastreado em `docs/FASE1_STATUS.md`.
- F1-02 (RF-008), F1-03 (RF-009/RF-010), F1-04 (RF-011/RF-018/RF-020) e F1-05 (RF-019/RF-022) deixam de estar bloqueados por F1-01, mas permanecem `PLANNED` — nenhum deles foi iniciado nesta rodada.
- Estado de implementação (`IMPLEMENTED` vs `VERIFIED`/`ACCEPTED`) e evidência de execução: `docs/FASE1_STATUS.md` e `docs/adrs/ADR-0008-modelo-analitico-comum.md`. `cargo fmt`/`check`/`clippy`/`nextest` não foram executados neste ambiente (sem `cargo`/`rustc` instalados) — a suíte de testes escrita em `src/persistence/analytical_results.rs` nunca foi compilada.

## Atualização 2026-08-25 — F1-02
- F1-02 (RF-008) implementado: `engine/rules/dfir_mvp_ruleset.yar` (ruleset curado, 4 regras originais), `engine/src/yara_scan.rs`, roteamento por `analysis_kind` em `engine/src/main.rs`, persistência dos matches em `analytical_results` (F1-01) dentro da transação de conclusão do job em `src/web/mod.rs`, novo `GET /api/evidence/{id}/analytical-results`.
- Mudança de contrato: `dfir_protocol::Request` ganhou o campo obrigatório `analysis_kind` (`SCHEMA_VERSION` "2"→"3"); `analysis_jobs` ganhou a coluna `kind` (migration `0007`, default `EVTX_TIMELINE` para linhas já existentes).
- Um primeiro erro de build real foi reportado e corrigido nesta rodada: o lint `SqlSafeStr` do sqlx 0.9 rejeitou as queries de leitura de F1-01 que usavam `format!()` para montar SQL — corrigido trocando por strings literais com `LIMIT` como bind parameter (ver `docs/FASE1_STATUS.md`).
- Estado: `IMPLEMENTED / PENDING-HUMAN` — nada neste ticket foi compilado neste ambiente (sem `cargo`/`rustc`); depende de `cargo generate-lockfile`/`cargo build` local para resolver `yara-x`/`sha2` no `Cargo.lock` antes de qualquer gate `--locked` funcionar. Detalhe completo: `docs/adrs/ADR-0010-yara-x-ruleset-curado.md`.

## Atualização 2026-08-27 — correções de build reportadas (Windows + rust-analyzer)
- Bug real de Windows corrigido em `scripts/verify_frontend.py` (invocava shim `.bin` diretamente; agora roteia por `node`). Confirmado por execução real no Windows do operador: `npm run build`/`npm test`/`npm run typecheck` passam. Ver adendo em `docs/adrs/ADR-0009-substituicao-bun-por-node-npm.md`.
- Avisos `dead_code` (rust-analyzer) corrigidos: `#![allow(dead_code)]` documentado em `analytical_results.rs` (API de F1-01 à frente de F1-03/04/05); `RULESET_LICENSE`/`RULESET_SOURCE_ORIGIN` de `yara_scan.rs` passaram a ser usadas de verdade no item emitido e no provenance persistido, em vez de suprimidas.

## Atualização 2026-08-27 — F1-03
- F1-03 (RF-009/RF-010) implementado: `migrations/0008_nist_profiles.sql` (`nist_profiles`+`nist_profile_entries`, append-only) + `src/persistence/nist_profiles.rs` (`compute_gaps` — função pura e determinística — e `run_gap_analysis`, que persiste cada gap como `AnalyticalResultType::NistGap` via F1-01).
- Pré-requisito descoberto durante a implementação: não existia CRUD de `incidents` na API. Adicionado (`POST/GET /api/incidents`, `GET /api/incidents/{id}`) nesta mesma rodada — dependência real de F1-03, não escopo inflado.
- Novas rotas: `POST/GET /api/incidents/{id}/nist/profiles`, `POST/GET /api/incidents/{id}/nist/gap-analysis`.
- Decisão de não embutir o catálogo oficial de subcategorias do CSF 2.0 (risco de transcrição incorreta em conteúdo de conformidade real): `subcategory_id` validado só na forma estrutural; descrição fornecida pelo analista. Decisão completa: `docs/adrs/ADR-0011-nist-profile-gap.md`.
- Estado: `IMPLEMENTED / PENDING-HUMAN` — 12 testes escritos (7 puros sobre `compute_gaps`, sem banco; 5 de integração), nenhum compilado neste ambiente.

## Atualização 2026-08-28 — F1-04
- F1-04 (RF-011/RF-018/RF-020) implementado: novo módulo `src/persistence/attack_impact.rs`, sem migration nova — ATT&CK mapping, estágio de caminho crítico e avaliação de impacto reusam `analytical_results` (F1-01) diretamente, já que são julgamentos analíticos, não dados de entrada como os profiles de F1-03.
- Mesma decisão de F1-03 (ADR-0011) aplicada ao MITRE ATT&CK: as 14 táticas são hardcoded (estrutura estável); o catálogo de ~600+ técnicas não é embutido, só a forma (`T####`/`T####.###`) é validada.
- "Relações não suportadas por evidência são marcadas como incerteza, não como fato" (requisito explícito do ticket) implementado como validação: confidence HIGH/CONFIRMED sem uma `basis` de evidência não-vazia é rejeitado, não apenas desencorajado.
- Caminho crítico usa `RelationType::SequenceNext` (já reservado em F1-01) entre estágios — novo helper genérico `analytical_results::insert_relation` adicionado para permitir ligar estágios depois de criados (o parâmetro `related` do insert original exige que o alvo já exista).
- Novas rotas: `POST/GET /api/incidents/{id}/attack-mappings`, `GET/POST /api/incidents/{id}/critical-path` (+`/stages`, `/links`), `GET/POST /api/incidents/{id}/impact-assessment`.
- Decisão completa e limitações assumidas: `docs/adrs/ADR-0012-attack-caminho-critico-impacto.md`.
- Estado: `IMPLEMENTED / PENDING-HUMAN` — 15 testes novos em `attack_impact.rs` + 2 em `analytical_results.rs` (para `insert_relation`), nenhum compilado neste ambiente.

## Atualização 2026-08-28 — F1-05 (fecha o backlog original)
- F1-05 (RF-019/RF-022) implementado: novo módulo `src/persistence/ioc_recommendations.rs`, sem migration nova.
- IOCs: `record_ioc_sighting` reusa `supersedes` de F1-01 com semântica diferente das anteriores — cada nova ocorrência do mesmo indicador (mesmo incidente+tipo+valor normalizado) **acumula** no agregado em vez de substituí-lo, preservando toda a proveniência (`sightings[]`) e recalculando `first_seen`/`last_seen`. Validação de valor via `std::net::IpAddr` (sem dependência nova) para IP e checagens estruturais leves para os demais 8 tipos da seção 17.
- Recomendações: sem chave natural de deduplicação — `supersedes` é explícito e opcional (o analista decide se é revisão ou nova); `supporting_result_ids` obrigatório e validado, virando relações `DerivedFrom` (mecanismo de F1-01/04, sem nada novo).
- RF-022 ("exportação preserva a mesma origem usada na UI") implementado literalmente: `export_incident_findings` reusa a mesma função (`current_results_for_incident`) que as rotas de listagem usam.
- Novas rotas: `POST/GET /api/incidents/{id}/iocs`, `POST/GET /api/incidents/{id}/recommendations` (`?category=`), `GET /api/incidents/{id}/export`.
- Decisão completa: `docs/adrs/ADR-0013-ioc-recomendacoes.md`.
- Estado: `IMPLEMENTED / PENDING-HUMAN` — 13 testes novos, nenhum compilado neste ambiente. Com F1-05, todo o backlog `.scratch/fase1-backlog/issues/` originalmente fornecido está implementado (nenhum ainda `VERIFIED`/`ACCEPTED` — gates Rust reais pendentes do operador).

## Atualização 2026-09-21 — interface e contrato de API

- RF-019 e RF-022 (F1-05), além de RF-009/RF-010 (F1-03) e RF-011/RF-018/RF-020 (F1-04), passam a ter **interface**: `frontend/src/incident-ui.ts` (DOM) e `frontend/src/incident-model.ts` (validação/apresentação, coberto por `frontend/src/incident-model.test.ts`). RF-022 continua garantido pela origem única: a aba Exportação e as abas de listagem consomem as mesmas rotas, e `GET /api/incidents/{id}/export` reutiliza `current_results_for_incident`.
- Contrato das rotas: `docs/API.md`. Decisões: ADR-0013 (adendo do teto de sightings), ADR-0015 (`engine_path`), ADR-0016 (engine em módulos e `NormalizedEvent`).

## Atualização 2026-09-22 — interface nativa (ADR-0017)

- Pedido do operador (decisão de arquitetura, **não** requisito da demanda — corrigido pela ADR-0018): substituir a interface (Axum + `frontend/` TypeScript) por uma interface nativa em Rust (eframe/egui/egui_extras). Por `AGENTS.md` §1 (ordem de precedência), este requisito supera o contrato de frontend do Apêndice F.3.1 e as ADR-0005/0009, que ficam superseded — ver ADR-0017.
- `src/web/`, `templates/`, `frontend/` removidos por inteiro. A lógica de negócio dos handlers (a mesma que atendia RF-019/RF-022/RF-009/RF-010/RF-011/RF-018/RF-020 acima) foi portada 1:1 para `src/service/` (`mod.rs`, `nist.rs`, `findings.rs`), sem HTTP. `docs/API.md` fica arquivado em `docs/archive/API_HTTP_v1_superseded.md` — não há mais rota alguma, a UI chama `src/service/` diretamente.
- Nenhum RF muda de status por esta atualização: RF-019/RF-022/RF-009/RF-010/RF-011/RF-018/RF-020 continuam com a mesma cobertura funcional, agora servida por `src/gui/` em vez de `frontend/`. A única mudança de comportamento é a importação de evidência, que passa de upload multipart para leitura de um caminho local escolhido via diálogo nativo de arquivo (`rfd`).
- Estado: `IMPLEMENTED / PENDING-HUMAN` — ver `docs/FASE1_STATUS.md` para a evidência de execução real desta sessão (ambiente sem o toolchain 1.97.1 pinado) e os gates que o operador ainda precisa reproduzir com o toolchain correto.

## Atualização 2026-10-02 — auditoria e realinhamento à demanda original (ADR-0018)

- A demanda original foi versionada em `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md` e rastreada em `docs/DEMAND_TRACE.md`.
- Especificação v6.5.0: `.dfirpkg` (`RF-025`), atualização offline (`RF-026`), Ansible (`RF-012`) e benchmark (`RF-045`) → Fase 4; `RF-041` substituído; perfis executivo/técnico removidos de `RF-046`; novos `RF-048`..`RF-055`. IDs anteriores não são reutilizados.
- Backlog reescrito (F1-13..F1-33 novos; F1-08/F1-09 adiados); linhas de baseline com evidência da era HTTP marcadas `STALE`.
- Evidência real desta data: `cargo check --locked` falhava por lock dessincronizado; a regeneração do lock em cópia de trabalho alterou apenas `eframe`/`egui`/`egui_extras` e dependências; a primeira compilação das mudanças de 2026-09-26 (crate de aplicação com a GUI trocada por um stub; egui 0.36.2 não compila em rustc 1.91) revelou o erro E0505 em `nist_profiles.rs` (fechamento de gap), corrigido; depois da correção, `cargo test -p dfir-app -p dfir-protocol` passou (95 + 10 testes) e, ao final de 2026-10-02, com todas as correções de código e a GUI compilada, app 104, engine 35 e protocolo 10 testes passaram em rustc 1.91.1 (ver `docs/FASE1_STATUS.md` para limites). Detalhes e limites do que foi executado: `docs/FASE1_STATUS.md`.

## Atualização 2026-10-13 — Etapa 1 do plano de conclusão do protótipo (ADR-0023)

- Migration `0010` recria `analytical_results` aceitando `FINDING`, `ROOT_CAUSE` (F1-25, RF-051) e `PLAN_ITEM` (F1-29, RF-052). Nenhum RF muda de estado: só a fundação de dados existe; Finding, causa-raiz e item de plano (modelo, serviço e GUI) começam nas Etapas 2 e 3.
- F1-26, F1-27 e F1-28 não entram no plano de conclusão do protótipo; continuam `PLANNED`. F1-06 é absorvido por F1-29 e F1-07.
- RF-032 (cópia e verificação do banco antes de migration) não está implementado: nenhuma ocorrência de backup em `src/`. A 0010 reescreve uma tabela forense, então a pendência pesa mais; até lá, cópia manual do banco antes de abrir a build nova (ADR-0023).
- Evidência: a migration foi executada em SQLite 3.45.1 sobre 0001–0009; os testes Rust foram executados depois, em rustc 1.91.1 (ver a atualização de 2026-10-14). Detalhe em `docs/FASE1_STATUS.md`.

## Atualização 2026-10-14 — Etapa 2 do plano de conclusão do protótipo (ADR-0023)

- **F1-25 (RF-051) passa a `IMPLEMENTED / PENDING-HUMAN`:** `persistence/findings.rs`, `service/findings.rs` (`create_finding`, `create_root_cause`, `findings_view`) e a aba "Achados" (`gui/findings.rs`). DEM-ID-02 e DEM-CR-01 passam a `COBERTO`; DEM-ENT-01 e DEM-ENT-02 seguem `PLANEJADO` (o documento depende de F1-07).
- Evidência: `cargo test` 154 ok (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` limpo e `cargo fmt --check` limpo, em rustc 1.91.1 com cópias locais descartáveis das crates do egui (ver `docs/FASE1_STATUS.md`). Nada é `VERIFIED` no 1.97.1 pinado.
- Limites conhecidos: a interface não expõe `supporting_result_ids` (IOCs/YARA como suporte de um achado) nem a revisão de um achado (`supersedes`); o serviço e a persistência já os aceitam.

## Atualização 2026-10-14 — Etapa 3 do plano de conclusão do protótipo (ADR-0023)

- **F1-29 (RF-052) passa a `IMPLEMENTED / PENDING-HUMAN`:** `persistence/plan_items.rs`, `service/plan.rs` (`create_plan_item`, `plan_view`) e a aba "Plano" (`gui/plan.rs`). **F1-06** é o domínio `OS_HARDENING` da mesma entidade (visão por domínio; sem Ansible, sem artefato executável) e fica `IMPLEMENTED / PENDING-HUMAN` quanto ao modelo; a saída no Entregável 4 depende de F1-07.
- DEM-PLAN-01 a DEM-PLAN-07, RB-004 e RB-005 passam a `COBERTO`. DEM-ENT-04, DEM-ENT-05 e DEM-ENT-06 seguem como estavam (o documento depende de F1-07).
- Evidência: `cargo test` 171 ok (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos, em rustc 1.91.1 com cópias locais descartáveis das crates do egui (ver `docs/FASE1_STATUS.md`). Nada é `VERIFIED` no 1.97.1 pinado.
- Limites conhecidos: a interface não oferece a revisão de um item (`supersedes`); o serviço e a persistência já a aceitam. Item sem achado de suporte é aceito e marcado como recomendação geral, como pede o ticket (o plano de execução da ADR-0023 previa exigir achado ou recomendação).

## Atualização 2026-10-15 — Etapa 4 do plano de conclusão do protótipo (ADR-0023)

- **F1-30 (RF-053) passa a `IMPLEMENTED / PENDING-HUMAN`:** `persistence/nist_functions.rs` (agregação pura por Function), `service/nist.rs` (`function_view`) e a seção "Visão por Function" no topo da aba NIST. DEM-NIST-01 a DEM-NIST-06 passam a `COBERTO`; DEM-ENT-03 segue `PLANEJADO` (o documento depende de F1-07).
- A visão traz **só contagens** (status do Current, subcategorias do Target, gaps por prioridade). Não há média, percentual nem nota: o NIST CSF 2.0 não define score oficial.
- **Tier:** é do profile inteiro, informado pelo analista, e não existe por Function. A visão o mostra uma vez por profile (Current e Target), com a justificativa. O texto do ticket ("Tier descritivo" em cada Function) não é atendido ao pé da letra, porque o dado não existe nessa granularidade e inventá-lo seria exatamente o que a especificação veta.
- **Gap desatualizado:** a visão compara os ids de profile gravados na proveniência de cada gap vigente com os profiles vigentes e avisa quando divergem (`STALE`).
- Evidência: `cargo test` 180 ok (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos, em rustc 1.91.1 com cópias locais descartáveis das crates do egui. Nada é `VERIFIED` no 1.97.1 pinado.

## Atualização 2026-10-16 — Etapa 5 do plano de conclusão do protótipo (ADR-0023)

- **F1-07 (RF-013) passa a `IMPLEMENTED / PENDING-HUMAN`, parcial:** `service/deliverables/` gera os cinco documentos em Markdown determinístico e o `manifest.json`; a aba "Entregáveis" pré-visualiza e grava; `dfir-app deliverables` faz o mesmo por linha de comando. Faltam a opção de redação (F1-32) e as lacunas de proveniência e retenção (F1-31), que entram na Etapa 6.
- DEM-ENT-02 a DEM-ENT-06 passam a `COBERTO`. **DEM-ENT-01 fica `PARCIAL`:** o Entregável 1 sai com custódia, linha do tempo, vetores, causa e lacunas das conclusões, mas sem as lacunas de proveniência e retenção (F1-31).
- Evidência: `cargo test` 200 ok (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos, em rustc 1.91.1 com cópias locais descartáveis das crates do egui. Nada é `VERIFIED` no 1.97.1 pinado.
- Limites conhecidos: a linha do tempo bruta não vai inteira para os documentos (resumo por evidência e marcos do caminho crítico; a completa fica no banco); a interface não aciona o seletor de pasta em teste (a geração é exercida pelo serviço, pela CLI e pelo comando da aba).

## Atualização 2026-10-18 — Etapa 6 do plano de conclusão do protótipo (ADR-0023)

- **F1-31 (RF-054) passa a `IMPLEMENTED / PENDING-HUMAN`:** a importação grava a proveniência que o operador declara (origem, quem forneceu, método, ferramenta e versão, data da coleta, autorização); o que não é informado fica como `NOT_PROVIDED` ou ausente, e a data de **importação** deixa de ser gravada como data de **coleta**. A aba de evidências diz o que é analisado e o que é só preservado. A aba "Lacunas" registra fonte esperada, período e motivo (`SOURCE_GAP`, migration 0011), e o Entregável 1 as lista, junto com a proveniência não informada.
- **F1-32 (RF-055) passa a `IMPLEMENTED / PENDING-HUMAN`:** flag "contém dado pessoal" por evidência (migration 0011) e máscara de dados pessoais nos cinco documentos, **ligada por padrão**. Exportar sem ela exige confirmação explícita na interface (`--no-redaction` na CLI) e a escolha vai para a auditoria.
- **F1-07 deixa de ser parcial.** DEM-ENT-01 passa a `COBERTO`; RB-001, RB-002 e RB-006 passam a `COBERTO`.
- Evidência: `cargo test` 230 ok (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos, em rustc 1.91.1 com cópias locais descartáveis das crates do egui. Nada é `VERIFIED` no 1.97.1 pinado.
- Limites conhecidos: a máscara reconhece IPv4, IPv6, e-mails e contas `DOMÍNIO\usuário` por padrão de texto, mais o campo "fornecida por" e o responsável da custódia; **nomes de pessoas em texto livre não são detectados**. A interface não oferece a revisão de uma lacuna (o serviço aceita).

## Atualização 2026-10-23 — Etapa 7 do plano de conclusão do protótipo (ADR-0023)

- **F1-11 (RF-017, RF-016 em parte) passa a `IMPLEMENTED / PENDING-HUMAN`, reduzido:** `service/demo.rs` e `dfir-app demo` registram o cenário BEC sintético sobre um EVTX real informado pelo operador (não distribuído), rotulado DEMO (classe da evidência, nome do incidente, bases e auditoria). Roteiro em `docs/DEMO.md`. RF-016: valem os testes de texto hostil, pasta de saída e ids dos documentos (Etapa 5) e a validação da proveniência (Etapa 6); **não há** testes de envelope JSONL nem de mapeamento CSV, porque não há importador (F1-27).
- **F1-12 passa a `PARCIAL`:** documentos vivos reconciliados; `docs/OPERATION.md` cobre RF-030 (proteção do volume, comandos de referência não executados) e a parte documental de RF-032. **RF-032 automático (cópia e verificação do banco antes de migration) segue sem implementação.** O critério "nenhuma linha em `PLANNED` para RF MVP" não é atendido: F1-26, F1-27 e F1-28 seguem `PLANNED`.
- **Defeitos reais corrigidos:** `analyze` falhava em toda evidência desde a Etapa 6 (`ColumnNotFound("source")`); evidência descartada sumia dos documentos.
- Evidência: `cargo test -p dfir-app` 240 ok (1 ignorado), clippy do aplicativo e `rustfmt --check` limpos, e2e 79/79 com engine e EVTX reais, em rustc 1.91.1 com cópias locais descartáveis (ver `docs/FASE1_STATUS.md`). Nada é `VERIFIED` no 1.97.1 pinado.

## Atualização 2026-11-04 — autoridade de estado de ticket e INC-03 (parcial)

- **Autoridade:** a RTM é a autoridade de estado de ticket (decisão do operador, 2026-11-04). `docs/FASE1_STATUS.md` passa a ser log cronológico e sua matriz do topo vai para `docs/archive/` (ação pendente). `docs/FASE0_STATUS.md` fica como matriz de capacidades da Fase 0. `docs/DOCUMENT_AUTHORITY.md` reflete isso.
- **Linha F1-12 corrigida:** a lista de pendentes citava `cargo audit` (F1-15) como não executado; a linha F1-15 registra a execução de 2026-10-29 e foi mantida.
- **`cargo vet` dispensado no MVP** (ADR-0018 D-2; confirmado pelo operador em 2026-11-04): exigência removida de `AGENTS.md` §8, `CONVENTIONS.md`, `docs/SUPPLY_CHAIN.md` e `PROMPT_NOVA_CONVERSA.md` (gap INC-04).
- **INC-03 parcialmente resolvido.** Falta: (a) a visão RF→código→teste→evidência, item P2c do cluster B; (b) reconciliar o texto de status dos tickets F1-12, F1-14 e F1-15, que ainda divergem desta tabela; (c) mover a matriz do topo de `docs/FASE1_STATUS.md` para `docs/archive/`.

## Atualização 2026-11-04 (2) — cluster B dos gaps P1

- **INC-15:** vista por RF (RF→código→teste→EVD→estado) acrescentada para os 11 RFs MVP que não tinham linha, incluindo `RF-015` e `RF-044`. Todo arquivo e teste citado existe. A coluna EVD é `[PENDING-HUMAN]` (registro vazio); `RF-003` está sem teste (`[não no contexto]`).
- **INC-13:** `docs/FASE1_STATUS.md` passou a espelhar a matriz desta RTM e a guardar um resumo por rodada; o conteúdo anterior está em `docs/archive/FASE1_STATUS_historico_2026-10.md` (com a contagem do engine corrigida para 83, gap REL-02).
- **INC-03 fechado:** os tickets F1-12, F1-14 e F1-15 ganharam linha de status vigente espelhada desta tabela; `scripts/validate_fase0.py` passou a comparar o status de cada ticket com esta RTM.
- **EVI-05 e REM-02:** notas datadas nos 15 tickets com nível de verificação em rustc 1.91.1 ou sem `cargo`, e em F1-02 e F1-05 (código removido pela ADR-0017).
- **EVI-01:** `docs/DEFINITION_OF_READY.md` não tem mais item marcado como concluído sem `EVD-*` ou `WVR-*`; todos viraram `[PENDING-HUMAN]` com pergunta objetiva.
- **AUS-04, VAZ-04, AUS-05, INC-08, INC-09, AUS-07, EVI-02, AUS-08:** `LICENSE` com o texto da Apache-2.0; `LICENSES_THIRD_PARTY.md` regerado do lock (licenças `[PENDING-HUMAN]`); `docs/SUPPLY_CHAIN.md` com estado vigente; `docs/API.md` com o contrato de `src/service/*`; `README.md` com os 16 subcomandos; nota do `.evtx` do operador em `tests/fixtures/README.md`; relatórios de rodada com `doc-type` e nota de correção.
- **EVI-04:** `scripts/validate_fase0.py` e `scripts/doc_gate.py` ampliados (pareamento de spec por versão; coerência entre documentos). Lacunas ainda abertas saem como `WARNING`: INC-06, INC-10, INC-12 e INC-16.

## Atualização 2026-11-09 — spec 6.5.1, SBOM fora do escopo e toolchain

- **Spec 6.5.1** (`docs/PROJECT_SPEC_v6_5_1.md` e a cópia raiz; a 6.5.0 está em `docs/archive/`): `RF-024` e `RF-031` reescritos e de volta à tabela; seção 41 recuperada (título, marcador e índice); F.8.1 e item 2 da F.12 restaurados; `D04`, `D12` e `D13` marcados no índice; `<data>`, `<ajuste-de-escopo>` e seção 53 atualizados. Fecha INC-06, INC-16, AUS-02 e VAZ-01.
- **ADR-0028:** SBOM fora do escopo (decisão do operador). `cyclonedx` deixou de ser gate.
- **ADR-0027 atualizada:** o harness de agente consta só como ferramenta que foi usada, não obrigatória; ele não impede commit nem histórico de conversa.
- **AUS-04 fechado:** `LICENSE` conferido byte a byte contra o texto anexado pelo operador; `LICENSES_THIRD_PARTY.md` com as licenças de `cargo metadata --locked --format-version 1` (cargo 1.99.1, 672 pacotes, todos com licença declarada). A execução foi do assistente e não está registrada como `EVD-*`.
- **Rust:** a stable mais recente (1.99.1) está fixada em `rust-toolchain.toml` e `toolchains.toml`; os validadores leem o pin. Nada foi compilado nem testado com a 1.99.1: `[PENDING-HUMAN]` (`just verify`, `cargo audit`, `cargo deny --workspace check`).
- **Lacunas de teste:** `RF-003` (arquivo de origem inalterado após a importação) e `RF-027` (nenhum endpoint de rede) são relevantes ao projeto e devem virar teste; a escrita e a execução em Rust ficam com o operador.
- **Ainda aberto (documentação):** perguntas do `docs/DEFINITION_OF_READY.md`, decisão sobre as 3 exceções informativas do `deny.toml`, registro da fixture EVTX sanitizada (F1-16), entradas do `CHANGELOG.md`, `docs/NATIVE_UI_ARCHITECTURE.md` (INC-10) e `PROMPT_NOVA_CONVERSA.md` (INC-12).

## Atualização 2026-11-14 — registro de problemas de execução local (BUILD-01..DOC-01)

- **BUILD-01:** `just e2e`/`just demo` usam `cargo build -p dfir-app -p dfir-engine`; o e2e foi executado com engine real e EVTX real, **110/110** com o `security.evtx` de referência. O script deixou de estourar `IndexError` quando o arquivo não gera candidatos e os testes de filtro usam os Event IDs presentes, não um arquivo específico; o truncamento do EVTX agora é menor que o arquivo de fato. `EVD-002`.
- **GUI-01:** o seletor nativo roda em thread própria (`src/gui/file_dialog.rs`), a UI não congela e há três caminhos de importação (seletor, caminho digitado, arrastar-e-soltar) com a mesma validação; testes cobrem caminho vazio/pasta, múltiplos arquivos e importação concorrente. Dependências Linux documentadas em `docs/OPERATION.md` §3D.
- **GFX-01:** o adaptador `wgpu` escolhido é registrado na inicialização (`adapter`, `backend`, `device_type`, `driver`); os erros de `wgpu_hal` continuam visíveis como diagnóstico, com a linha do adaptador para interpretá-los.
- **SEC-01:** `evtx 0.12.3 → 0.15.1` resolve `RUSTSEC-2026-0431`; `encoding`/`RUSTSEC-2021-0153` saiu do grafo. Resultado funcional idêntico: 2 261 eventos, 6 candidatos, sem avisos do parser.
- **SEC-02:** `RUSTSEC-2026-0459` (`libsqlite3-sys 0.37.0`) fica como exceção temporária justificada (sem `sqlx` publicado que admita `>=0.39.0`), com alcance, controles compensatórios e revisão até 2026-12-31. **Não é `VERIFIED`; é risco aceito e documentado.**
- **TOOL-01:** `just supply-chain` executa `cargo audit` e `cargo deny --workspace check` e ambos passam; a retirada de `async-executor 1.14.0` foi corrigida. `EVD-003`.
- **DOC-01:** `rust-version` e manifests em 1.99; `docs/TOOLCHAINS.md`, `README.md`, `PROMPT_NOVA_CONVERSA.md`, `LICENSES_THIRD_PARTY.md` e `docs/SUPPLY_CHAIN.md` atualizados. Menções a 1.97.1 em ADRs e logs históricos foram preservadas como registro do que foi executado na época.
- **Evidências:** `EVD-001` (`just verify`), `EVD-002` (e2e 110/110) e `EVD-003` (supply chain), em `docs/EVIDENCE_REGISTER.md`. Windows, Linux aarch64 e incidente real continuam **não** executados.

### Atualização 2026-11-14 (tarde) — upgrade `eframe`/`egui` 0.37.0 + `cargo update` geral

- **Item 1 do plano do operador (atualizar dependências, começando pelo `eframe` mais recente):** `eframe`/`egui`/`egui_extras`/`egui_kittest` 0.36.2 → 0.37.0, `rfd` → 0.18.0. `eframe` com `default-features = false` (sem `accesskit`): a cadeia `zbus`/`zvariant`/`atspi`/`ashpd`/`ttf-parser` saiu do grafo de build, resolvendo por remoção as exceções informativas RUSTSEC-2026-0370/0371/0372 e RUSTSEC-2026-0192 (removidas do `deny.toml`).
- `cargo update --workspace`: 0 pendências (máximas compatíveis com rustc 1.99); minors `jiff`, `toml`, `uuid` aplicados (`defmt` fora do grafo); 52 pacotes restantes bloqueados por `rust-version > 1.99`.
- Testes de GUI adaptados ao `egui_kittest 0.37` (`allow_missing_glyphs`, nomes acessíveis via `labelled_by`/`from_label`); binário sem mudança de código.
- **Evidências:** `EVD-005` (`just verify` PASS, nextest 410/410), `EVD-006` (e2e 110/110), `EVD-007` (supply chain: audit 0 vulnerabilidades, deny ok). Windows, aarch64 e janela real da GUI continuam não executados.

### Atualização 2026-11-14 (noite) — itens p2–p6 do plano do operador: waivers formais, Definition of Ready, hardware e políticas operacionais

- **p2 — waivers formais** (aprovados pelo operador em 2026-11-14T22:00:30Z, registrados em `docs/EVIDENCE_REGISTER.md` no formato F.14): `WVR-001` (Runtime/Windows: Job Object só compilado; saída = `engine-smoke` + e2e em Windows; válido até 2026-12-31), `WVR-002` (Runtime/aarch64), `WVR-003` (CI automático, F1-14; baseline manual `just verify` = `EVD-005`).
- **p3 — `docs/DEFINITION_OF_READY.md`:** todos os itens passaram a `[x]` citando `EVD-001`..`EVD-009` e `WVR-001`..`WVR-004` (regra EVI-01 do `validate_fase0.py` satisfeita).
- **p4 — SEC-02 formalizado:** `WVR-004` cobre `RUSTSEC-2026-0459` (`libsqlite3-sys 0.37.0`), com ponteiro cruzado em `deny.toml` e `.cargo/audit.toml`; revisão até 2026-12-31.
- **p5 — hardware/verificação real da GUI:** `EVD-009` — janela real aberta no Xorg `:0` do host por 25 s; log registrou o adaptador wgpu escolhido (Mesa Intel HD Graphics 2000, backend Gl, 2 candidatos) — cobre GUI-01 (janela abre e não trava) e GFX-01 (adaptador no log, erros EGL interpretáveis).
- **p6 — políticas operacionais** documentadas em `docs/OPERATION.md`: §3.1.1 retenção de `backups/` (app nunca apaga; manter as duas últimas; limpeza manual após `backup-verify`) e §5 distribuição de `.evtx` (pacote não distribui amostras; `security.evtx` é fornecido pelo operador com SHA-256 registrado; F1-16 de fixtures sanitizadas continua PLANNED).

### Atualização 2026-11-15 — pistas C e D do plano remanescente concluídas (`EVD-010`, `EVD-011`)

- **Pista C (F1-16):** fixture EVTX sanitizada e determinística versionada em `tests/fixtures/senai-demo.evtx` (SHA-256 `256a655a7e259908e1dfd23b914abf27b7ecb8fd98c0b3f73033336f6ed448be`); `just e2e tests/fixtures/senai-demo.evtx` roda sem arquivo externo e passou **110/110** (`EVD-011`); `docs/OPERATION.md` §5 atualizado.
- **Pista D:** F1-19 vínculo evidência↔incidente idempotente com auditoria `EVIDENCE_LINKED_TO_INCIDENT` (GUI + CLI `link-evidence`/`incident-evidence`; vigentes sem janela; YARA idempotente); F1-20 limites YARA formalizados com testes (`wide ascii` casando UTF-16LE, `scan_timeout_for` por tamanho com teto de 6 h, `MAX_MATCHES_PER_PATTERN` sem descarte silencioso, varredura sem truncamento); F1-21 ordem da destruição registro→arquivo→jobs com retomada, testes cobrindo falha de banco (nada apagado), falha de I/O (registro mantido) e purga completa incl. candidatos de detecção com custódia preservada; F1-23 nenhum `let _ =` em código de produção (`rollback_loudly`, `discard_temp_file/dir`, `mark_job_failed_loudly`, sends de canal e abort de stderr logados; `NotFound` tratado como esperado); F1-24 custódia e auditoria visíveis na GUI (painel no detalhe da evidência, abas Evidências/Auditoria em Incidentes) e CLI (`custody`, `audit`); F1-33/AUD-32 corrigido (análise YARA não sobrescreve a timeline).
- **Verificação executada:** `cargo fmt --all -- --check` ok; `cargo clippy --workspace --locked --all-targets --all-features -- -D warnings` limpo; `cargo nextest run --workspace --locked` **424/424** (1 ignorado), incluindo os testes novos de cada ticket (`EVD-010`); e2e **110/110** (`EVD-011`). Waivers `WVR-001`..`WVR-004` inalterados (Windows, aarch64, CI e SEC-02 continuam pendentes de infraestrutura externa).

### Atualização 2026-11-18 — upgrade do pin para Rust 1.100.0 e lacunas RF-003/RF-027 fechadas (`EVD-012`)

- **Pin:** `rust-toolchain.toml` e `toolchains.toml` de 1.99.1 para **1.100.0** (stable lançada em 2026-11-12); `rust-version = "1.100"` nos três `Cargo.toml`. `cargo metadata --locked` ok; `cargo update --dry-run` não força nenhum bump (só `yara-x-parser 1.22.1 → 1.23.0` disponível, não aplicado para manter o lock estável nesta rodada). `docs/TOOLCHAINS.md`, `docs/SUPPLY_CHAIN.md` (linha do pin) e `docs/FASE1_BACKLOG.md` (pista B) atualizados; menções a 1.99.1 em logs históricos preservadas como registro.
- **Código:** 1 fix de lint novo do clippy 1.100 (`needless_bool` em `src/persistence/audit.rs`: `Ok(actual_count == verified_count)`).
- **Lacunas de teste fechadas:** RF-003 → `src/service/provenance.rs::import_leaves_the_source_file_byte_for_byte_untouched` (bytes, mtime e SHA-256 do original idênticos antes/depois da importação; cópia preservada com os mesmos bytes); RF-027 → `src/main.rs::no_network_listener_exists_in_app_or_engine_source` (guarda estática recursiva sobre `src/`, `engine/src/`, `protocol/src/`; o engine contido já prova em runtime que socket TCP/Unix é EPERM sob seccomp em `engine/tests/sandbox.rs`).
- **Supply chain:** o bump revelou `RUSTSEC-2026-0543` (chrono 0.4.45 "unmaintained", informativo, sem CVE) no `cargo deny`; exceção justificada adicionada em `deny.toml` e `.cargo/audit.toml` (uso só de timestamps locais/parsing RFC 3339 interno; rever até 2026-12-31; avaliar migrar para jiff). WVR-004 inalterado.
- **Verificação executada (`EVD-012`):** em rustc/cargo 1.100.0, Linux x86_64: `cargo fmt --check` ok; `cargo check --workspace --locked --all-targets` ok; `cargo clippy --workspace --locked --all-targets --all-features -- -D warnings` limpo; `cargo nextest run --workspace --locked` **426/426** (1 ignorado); `just e2e tests/fixtures/senai-demo.evtx` **110/110** com engine real e sandbox `required`; `cargo deny --workspace check` e `cargo audit` verdes (2 exceções justificadas); `validate_fase0.py` 0 erros/0 avisos; `doc_gate.py` PASS. Windows, aarch64 e incidente real continuam pendentes (`WVR-001`/`WVR-002`, até 2026-12-31).
