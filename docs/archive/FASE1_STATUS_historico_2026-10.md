<!-- doc-type: status | status: historical -->
# Status da Fase 1 (histórico, até 2026-10-29)

> **Status: HISTÓRICO / SUPERSEDED.** Matriz e log cronológico originais de `docs/FASE1_STATUS.md` até 2026-10-29, movidos para cá em 2026-11-04 (Gap origem: INC-13). O estado vigente de ticket está em `docs/rtm.md` (autoridade); o espelho da matriz e o resumo por rodada estão em `docs/FASE1_STATUS.md`. Não use este arquivo como estado atual.

Este documento acompanha a execução do backlog em `docs/FASE1_BACKLOG.md` (tickets `.scratch/fase1-backlog/issues/`), no mesmo formato de `docs/FASE0_STATUS.md`. Não substitui a RTM (`docs/rtm.md`), que rastreia RF ↔ código ↔ teste; este documento rastreia ticket ↔ estado de implementação.

Estados possíveis (AGENTS.md §9): PLANNED → IMPLEMENTED → INTEGRATED → VERIFIED → ACCEPTED. `PENDING-HUMAN` indica que o código foi escrito e revisado manualmente, mas os gates Rust reais (`cargo fmt`/`check`/`clippy`/`nextest`) não puderam ser executados no ambiente de implementação por falta de `cargo`/`rustc` instalados.

## Matriz de capacidades

| Ticket | Entrega | Estado | Evidência |
|---|---|---|---|
| F1-01 | Modelo analítico comum e versionamento de resultados | IMPLEMENTED / PENDING-HUMAN | `migrations/0006_analytical_results.sql`, `src/persistence/analytical_results.rs`, `docs/adrs/ADR-0008-modelo-analitico-comum.md` |
| F1-02 | YARA-X (RF-008) | IMPLEMENTED / PENDING-HUMAN | `engine/rules/dfir_mvp_ruleset.yar`, `engine/src/yara_scan.rs`, `migrations/0007_analysis_job_kind.sql`, `docs/adrs/ADR-0010-yara-x-ruleset-curado.md` |
| F1-03 | Gap NIST (RF-009/RF-010) | IMPLEMENTED / PENDING-HUMAN | `migrations/0008_nist_profiles.sql`, `src/persistence/nist_profiles.rs`, `docs/adrs/ADR-0011-nist-profile-gap.md` |
| F1-04 | ATT&CK / caminho crítico / impacto (RF-011/018/020) | IMPLEMENTED / PENDING-HUMAN | `src/persistence/attack_impact.rs`, `docs/adrs/ADR-0012-attack-caminho-critico-impacto.md` |
| F1-05 | IOCs / recomendações (RF-019/022) | IMPLEMENTED / PENDING-HUMAN | `src/persistence/ioc_recommendations.rs`, `docs/adrs/ADR-0013-ioc-recomendacoes.md` |

Todo o backlog originalmente fornecido em `.scratch/fase1-backlog/issues/` está implementado nesta rodada. Nenhum ticket está `VERIFIED`/`ACCEPTED` — os gates Rust reais (`cargo fmt`/`check`/`clippy`/`nextest`) dependem do operador, que testa localmente e reporta erros (ver histórico de correções já aplicadas abaixo).

Demais tickets do backlog permanecem `PLANNED`. **O backlog foi reescrito em 2026-10-02 (ADR-0018): F1-13..F1-33 são novos; F1-08 e F1-09 foram adiados para a Fase 4.** Ver `docs/FASE1_BACKLOG.md` e a seção "Atualização 2026-10-02" abaixo.

## F1-01 — detalhe

Implementado nesta rodada:

- `migrations/0006_analytical_results.sql`: tabela `analytical_results` (append-only, triggers `BEFORE UPDATE`/`BEFORE DELETE`) e `analytical_result_relations` (`DERIVED_FROM`, `SEQUENCE_NEXT`).
- `src/persistence/analytical_results.rs`: tipos (`AnalyticalResultType`, `Confidence`, `RelationType`), validação de campos obrigatórios por tipo, `insert`/`insert_on_connection` (com evento de auditoria `ANALYTICAL_RESULT_RECORDED` na mesma transação), `get`, `current_for_correlation_key`, `history_for_correlation_key`, `list_by_evidence`, `list_by_incident`, `related_to`.
- `src/domain/` (vazio, órfão, sem `mod domain;`) removido.
- Suíte de testes (`#[cfg(test)] mod tests` no mesmo arquivo): 12 testes cobrindo canonicalização JCS, validação de campos obrigatórios, preservação de histórico via `supersedes`, comportamento append-only e relações entre resultados.
- Decisão completa, alternativas consideradas e escopo deliberadamente fora do ticket: `docs/adrs/ADR-0008-modelo-analitico-comum.md`.
- `docs/rtm.md` atualizado com a seção de rastreabilidade de F1-01.

Pendente (fora do alcance deste ambiente de implementação):

- `cargo fmt --all -- --check`
- `cargo check --locked --all-targets`
- `cargo clippy --locked --all-targets --all-features --locked -- -D warnings`
- `cargo nextest run --locked` (a suíte de testes acima nunca foi compilada nem executada)

## F1-02 — detalhe

Implementado nesta rodada:

- `engine/rules/dfir_mvp_ruleset.yar`: 4 regras originais e curadas (EICAR, PowerShell com comando codificado, palavras-chave de nota de resgate, comando conhecido do Mimikatz) — nenhuma regra externa importada.
- `engine/src/yara_scan.rs`: compila o ruleset embutido (`include_str!`), varre com `yara-x` 1.20.0, nunca inclui bytes casados (só offset/comprimento), SHA-256 do ruleset recalculado em runtime. 7 testes (compila, hash estável, match positivo EICAR, sem match, erro controlado para path inexistente, erro de ruleset inválido, matches Mimikatz/ransom note).
- `protocol/src/lib.rs`: novo campo obrigatório `Request.analysis_kind` (`EVTX_TIMELINE`/`YARA_SCAN`), `SCHEMA_VERSION` "2"→"3", validação e testes atualizados.
- `engine/src/main.rs`: `process_request` roteia por `analysis_kind`; lógica EVTX original extraída para `process_evtx_request`, sem mudança de comportamento.
- `migrations/0007_analysis_job_kind.sql`: coluna `kind` em `analysis_jobs` (default `EVTX_TIMELINE` para linhas existentes).
- `src/web/mod.rs`: handler `analyze` aceita `kind` opcional; para `YARA_SCAN`, persiste cada match em `analytical_results` (F1-01) na mesma transação que marca o job `COMPLETED`; novo `GET /api/evidence/{id}/analytical-results` (subconjunto seguro do modelo F1-01); helper `read_result_items` extraído e reutilizado por `job_results`.
- Decisão completa, alternativas e dívida técnica assumida (features default do `yara-x`, truncamento de `read_result_items`): `docs/adrs/ADR-0010-yara-x-ruleset-curado.md`.
- `docs/rtm.md` atualizado.

Erro de build real já reportado e corrigido nesta rodada (não hipotético):

- **`SqlSafeStr` (sqlx 0.9)**: as 5 queries de leitura de `analytical_results.rs` (`get`, `current_for_correlation_key`, `history_for_correlation_key`, `list_by_evidence`, `list_by_incident`) usavam `format!()` para montar SQL, o que o sqlx 0.9 rejeita por padrão (só aceita `&'static str` ou `AssertSqlSafe`). Corrigido trocando por strings SQL literais, com `LIMIT` como bind parameter.
- **Avisos `dead_code`** (rust-analyzer, ambiente Windows do operador) em `analytical_results.rs` (variantes/funções de F1-01 ainda sem consumidor de produção — F1-03/04/05) e em `engine/src/yara_scan.rs` (`RULESET_LICENSE`/`RULESET_SOURCE_ORIGIN` declaradas mas não usadas no item emitido). Corrigidos: `#![allow(dead_code)]` documentado no primeiro caso; uso real das duas constantes no item `YARA_MATCH` e no `provenance` persistido no segundo caso (reforça "origem", exigida pelo próprio ticket).
- **Bug real de Windows em `scripts/verify_frontend.py`**: invocava `node_modules/.bin/esbuild`/`.bin/vitest` diretamente (`WinError 193` — não são executáveis Win32 válidos sem shell). Corrigido: tudo roteado por `node <script>.mjs`; novo `frontend/verify-build.mjs`. Ver adendo em `docs/adrs/ADR-0009-substituicao-bun-por-node-npm.md`.
- **Confirmado por execução real no Windows do operador**: `npm run build`, `npm test` (8/8) e `npm run typecheck` passam sem alteração — a migração Bun→Node.js/npm funciona no ambiente de destino real.

Pendente (fora do alcance deste ambiente de implementação):

- **Passo obrigatório antes de qualquer gate `--locked`**: `Cargo.lock` não tem entradas para `yara-x`/`sha2`/dependências transitivas (`wasmtime`, `cranelift`, etc.); rodar `cargo build` uma vez sem `--locked` primeiro.
- `cargo fmt --all -- --check`
- `cargo check --locked --all-targets`
- `cargo clippy --locked --all-targets --all-features --locked -- -D warnings`
- `cargo nextest run --locked`

## F1-03 — detalhe

Implementado nesta rodada:

- `migrations/0008_nist_profiles.sql`: `nist_profiles` (versão/fonte/tier, append-only) + `nist_profile_entries` (uma por subcategoria).
- `src/persistence/nist_profiles.rs`: `compute_gaps` — função pura, sem banco, determinística — mais `insert_profile` (versiona automaticamente via `supersedes`) e `run_gap_analysis` (persiste cada gap como `AnalyticalResultType::NistGap` de F1-01, dentro de uma transação, encadeando `supersedes` com a gap anterior da mesma subcategoria quando existir).
- Descoberto como pré-requisito real (não estava planejado antes de começar): não existia CRUD de `incidents` na API. Adicionado `POST/GET /api/incidents`, `GET /api/incidents/{id}`.
- Novas rotas: `POST/GET /api/incidents/{id}/nist/profiles` (`?type=CURRENT|TARGET`), `POST/GET /api/incidents/{id}/nist/gap-analysis`.
- Decisão de não embutir o catálogo oficial do CSF 2.0 (risco de transcrição incorreta) — `subcategory_id` validado só estruturalmente. Decisão completa e algoritmo do gap: `docs/adrs/ADR-0011-nist-profile-gap.md`.
- `docs/rtm.md` atualizado.
- 12 testes: 7 puros sobre `compute_gaps`/`combined_confidence` (gap real, sem gap, entrada ausente, `NOT_APPLICABLE` nos dois lados, subcategoria só no current, determinismo, confiança combinada) + 5 de integração (profile vazio rejeitado, subcategory_id inválido rejeitado, versionamento com histórico preservado, gap analysis exige os dois profiles, persistência+resupersessão de gaps).

Pendente (fora do alcance deste ambiente de implementação):

- `cargo fmt --all -- --check`
- `cargo check --locked --all-targets`
- `cargo clippy --locked --all-targets --all-features --locked -- -D warnings`
- `cargo nextest run --locked`

## F1-04 — detalhe

Implementado nesta rodada:

- Novo módulo `src/persistence/attack_impact.rs` (sem migration — ATT&CK mapping, estágio de caminho crítico e avaliação de impacto reusam `analytical_results` de F1-01 diretamente, por serem julgamentos analíticos, não dados de entrada como os profiles de F1-03).
- `AttackTactic` (14 táticas oficiais, hardcoded) + `is_valid_technique_id` (valida só a forma `T####`/`T####.###`, mesma decisão de não embutir catálogo grande da ADR-0011).
- Validação "incerteza, não fato" (requisito explícito do ticket): confidence HIGH/CONFIRMED sem uma `basis` de evidência não-vazia é rejeitada (`require_evidence_for_high_confidence`); payload grava `evidence_status: GAP|DOCUMENTED` explicitamente.
- Caminho crítico: estágios são `CRITICAL_PATH_LINK` independentes, ligados por `RelationType::SequenceNext` — novo helper genérico `analytical_results::insert_relation` (2 testes novos em `analytical_results.rs`).
- `ImpactLevel` (NONE/LOW/MEDIUM/HIGH/CRITICAL) para disponibilidade/confidencialidade/integridade; `financial_impact: Option<&str>` nunca fabricado.
- Novas rotas: `POST/GET /api/incidents/{id}/attack-mappings`, `GET /api/incidents/{id}/critical-path`, `POST /api/incidents/{id}/critical-path/stages`, `POST /api/incidents/{id}/critical-path/links`, `GET/POST /api/incidents/{id}/impact-assessment`.
- Helper `current_results_for_incident` extraído em `web/mod.rs` (antes só usado por `get_nist_gap_analysis`, agora compartilhado também por ATT&CK/caminho crítico/impacto).
- Decisão completa e limitações assumidas (estágios não re-vinculam relações automaticamente ao serem substituídos; leitura do grafo é N+1): `docs/adrs/ADR-0012-attack-caminho-critico-impacto.md`.
- `docs/rtm.md` atualizado.
- 15 testes novos em `attack_impact.rs` + 2 em `analytical_results.rs`.

Pendente (fora do alcance deste ambiente de implementação):

- `cargo fmt --all -- --check`
- `cargo check --locked --all-targets`
- `cargo clippy --locked --all-targets --all-features --locked -- -D warnings`
- `cargo nextest run --locked`

## F1-05 — detalhe (fecha o backlog original)

Implementado nesta rodada:

- Novo módulo `src/persistence/ioc_recommendations.rs` (sem migration).
- `record_ioc_sighting`: duplicata do mesmo IOC (mesmo incidente+tipo+valor normalizado) **acumula** no agregado via `supersedes` em vez de substituir — cada versão do payload é um retrato completo dos `sightings` até aquele ponto, `first_seen`/`last_seen` recalculados. Validação de valor: `std::net::IpAddr` da std para IP (zero dependência nova), checagens estruturais leves para os outros 8 tipos da seção 17.
- `insert_recommendation`: `supersedes` explícito e opcional (sem chave natural de dedup para recomendações, diferente de IOCs/gaps); `supporting_result_ids` obrigatório e validado, virando relações `DerivedFrom`.
- RF-022 ("exportação preserva a mesma origem usada na UI"): `export_incident_findings` reusa literalmente a mesma função (`current_results_for_incident`) das rotas de listagem.
- Novas rotas: `POST/GET /api/incidents/{id}/iocs`, `POST/GET /api/incidents/{id}/recommendations` (`?category=`), `GET /api/incidents/{id}/export`.
- Decisão completa: `docs/adrs/ADR-0013-ioc-recomendacoes.md`.
- `docs/rtm.md` atualizado.
- 13 testes novos.

Pendente (fora do alcance deste ambiente de implementação):

- `cargo fmt --all -- --check`
- `cargo check --locked --all-targets`
- `cargo clippy --locked --all-targets --all-features --locked -- -D warnings`
- `cargo nextest run --locked`

**Com F1-05, todo o backlog de `.scratch/fase1-backlog/issues/` fornecido está implementado.** Nenhum ticket é `VERIFIED`/`ACCEPTED` — todos dependem dos gates Rust reais, que só o operador consegue rodar neste momento.

## Frontend / toolchain (fora do backlog de Fase 1, mas executado nesta rodada)

Bun foi removido da toolchain de build/test do frontend por não ser instalável neste ambiente (ver `docs/adrs/ADR-0009-substituicao-bun-por-node-npm.md` e o addendum em `docs/FASE0_STATUS.md`). Diferente de F1-01, esta mudança **foi** executada de ponta a ponta neste ambiente: `npm install`, `npm run typecheck`, `npm run build`, `npm test`, `scripts/verify_frontend.py` e `scripts/validate_fase0.py` todos rodaram com sucesso e evidência real (não PENDING).

## Atualização 2026-09-21 — primeira execução real dos gates, revisão de supply chain e interface

- **Gates executados** (sandbox Linux, rustc 1.97): `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` (engine 30, protocol 10, app 103), `cargo deny --workspace check` e os gates de frontend (`typecheck`, `vitest` 28, `dist` idêntico a um build limpo). `cargo nextest`, `cargo audit` e `cargo vet` não foram executados. Os estados da matriz acima **não** foram alterados: promover a `VERIFIED` continua dependendo da execução no ambiente do operador (Windows), onde ainda faltam `reject_symlink`/engine no Windows e um navegador real.
- **Supply chain do `yara-x`**: revisado e reduzido (`docs/SUPPLY_CHAIN.md`, ADR-0010 adendo). O gate de `cargo deny` não cobria o engine até esta revisão.
- **F1-05**: teto de `sightings[]` (ADR-0013 adendo).
- **Engine**: `engine_path` relativo ao executável (ADR-0015); engine em módulos e `NormalizedEvent` tipado (ADR-0016).
- **Interface**: F1-01..F1-05 agora têm UI (incidentes, NIST, ATT&CK, caminho crítico, impacto, IOCs, recomendações, exportação) — `frontend/README.md`, `tests/frontend/fase0.md`.
- **Contrato de API**: `docs/API.md`.
- **Erros latentes corrigidos** ao compilar pela primeira vez: ver `CHANGELOG.md`.

## Atualização 2026-09-22 — interface nativa (ADR-0017)

Pedido do operador (decisão de arquitetura; **não** requisito da demanda original — classificação corrigida pela ADR-0018): substituir a interface (Axum + `frontend/` TypeScript) por uma interface nativa em Rust (eframe/egui/egui_extras). Decisão completa: `docs/adrs/ADR-0017-interface-nativa-eframe-egui.md`. Detalhe de arquitetura: `docs/NATIVE_UI_ARCHITECTURE.md`.

Nenhuma linha da matriz de capacidades acima muda de ticket ou de cobertura funcional — F1-01..F1-05 continuam coberto por `src/persistence/`, agora orquestrado por `src/service/` (extraído de `web/mod.rs`, removido) em vez de por handlers Axum, e exposto por `src/gui/` em vez de `frontend/`.

Implementado nesta rodada:

- `src/web/`, `templates/`, `frontend/` removidos por inteiro. Dependências removidas: `axum`, `tower`, `tower-http`, `askama`.
- `src/service/mod.rs`, `src/service/nist.rs`, `src/service/findings.rs`: lógica de negócio dos handlers Axum portada 1:1 (mesmas transações `BEGIN IMMEDIATE`, auditoria, custódia, validação).
- `src/gui/`: `mod.rs`, `backend.rs`, `evidence.rs`, `incidents.rs`, `nist.rs`, `attack.rs`, `critical_path.rs`, `impact.rs`, `iocs.rs`, `recommendations.rs`, `export.rs` — telas equivalentes às de `frontend/src/`.
- `src/main.rs`: sem subcomando abre a GUI; `audit-verify`/`engine-smoke` continuam subcomandos de CLI.
- `src/config.rs`: `bind`/`DFIR_BIND`/`DFIR_BIND_ALLOW_NON_LOOPBACK` removidos.
- `src/cli.rs`: template Askama do `engine-smoke` trocado por `format!` direto.
- `docs/API.md` arquivado em `docs/archive/API_HTTP_v1_superseded.md`.
- `scripts/validate_fase0.py`, `scripts/doc_gate.py`: atualizados para não depender de `frontend/`/Askama.

Estado: `IMPLEMENTED / PENDING-HUMAN`.

**Gates executados nesta sessão** (ambiente sem o toolchain `1.97.1` fixado em `rust-toolchain.toml`; `rustc`/`cargo` `1.91.1` instalados via `apt` só para esta verificação, com `--ignore-rust-version` — a MSRV declarada de `sqlx 0.9` e de outras dependências transitivas já excedia 1.91 antes desta mudança):

- `cargo check -p dfir-app --ignore-rust-version`: **limpo, 0 erros, 0 avisos**.
- `cargo test -p dfir-app --bin dfir-app --ignore-rust-version`: **93/93 testes verdes** (inclui os testes novos de `error.rs`/`cli.rs` e todos os testes pré-existentes de `src/persistence/`, inalterados).
- `cargo fmt --all -- --check`: **limpo**.
- `cargo clippy -p dfir-app --all-targets --all-features --ignore-rust-version -- -D warnings`: **limpo** (após corrigir 3 ocorrências de `clippy::too_many_arguments` com `#[allow(...)]`, mesmo padrão já usado em duas outras funções do próprio `findings.rs`).

**Não executado / não fecha neste ambiente**:

- `cargo check --workspace --all-targets`: falha em `dfir-engine` → `yara-x 1.20.0` → `wasmtime-environ 45.0.3`, que usa a feature de biblioteca instável `debug_closure_helpers` (issue rust-lang/rust#117729), indisponível no rustc 1.91. **Confirmado pré-existente e alheio a esta mudança**: reproduzido o mesmo erro, sem nenhuma alteração desta sessão, rodando `cargo check -p dfir-engine` sobre uma cópia intocada do projeto antes da migração. `dfir-app`/`dfir-protocol` compilam normalmente dentro do mesmo `cargo check --workspace` até o ponto em que o resolvedor chega em `dfir-engine`.
- `cargo clippy`/`cargo check`/`cargo build` com `--workspace` (mesma causa acima) e com `--locked` (o `Cargo.lock` foi regenerado sem `--locked` nesta sessão, já que as dependências mudaram — o operador deve rodar `cargo generate-lockfile` ou revisar o lockfile resultante com o toolchain `1.97.1` antes de fixar `--locked`).
- `cargo nextest`, `cargo audit`, `cargo deny`: ferramentas não instaladas neste ambiente (mesmo estado já registrado para essas ferramentas em sessões anteriores deste projeto).
- A interface nunca foi renderizada/exercitada visualmente nesta sessão — este ambiente não tem display; `cargo check`/`test`/`clippy` verificam que o código compila e os testes de lógica passam, não que a janela abre ou que os widgets respondem a clique. O operador precisa abrir `dfir-app` de verdade antes de considerar a UI aceita.

O operador deve reproduzir os gates acima com o toolchain `1.97.1` real (`just fmt`, `just check`, `just test`, `just nextest`, `just clippy`, `just supply-chain`) antes de promover este ticket além de `PENDING-HUMAN`.

## Atualização 2026-10-02 — auditoria, realinhamento à demanda (ADR-0018) e primeira compilação das mudanças de 2026-09-26

Escopo: auditoria completa de documentação e código contra o texto **original** da demanda, correção da documentação, poda/adição de escopo (ADR-0018, spec v6.5.0) e um conjunto pequeno de correções de código com teste.

### O que foi executado (e em qual ambiente)

Ambiente: Ubuntu 24.04, `rustc`/`cargo` **1.91.1** via `apt`, sempre com `--ignore-rust-version` (o toolchain pinado 1.97.1 não está disponível). Nada abaixo é `VERIFIED` no sentido do AGENTS §10.

| Execução | Resultado |
|---|---|
| `cargo check --locked` com o `Cargo.lock` original | **Falhou**: lock precisa de atualização (manifesto em egui 0.36.2, lock em 0.33.3). AUD-01 confirmado por execução. |
| Regeneração do lock (`cargo metadata` sem `--locked`) | Alterou só `eframe`/`egui`/`egui_extras` (0.33.3 → 0.36.2) e a árvore de UI correspondente. `yara-x 1.20.0`, `evtx 0.12.3`, `rfd 0.17.2` inalterados. |
| Primeira compilação das mudanças de 2026-09-26 | **Erro E0505** em `src/persistence/nist_profiles.rs` (`previously_open_gaps`). Corrigido. |
| Primeira compilação da GUI com egui 0.36.2 | **2 erros** (`egui::SidePanel` não existe mais; `show_inside` depreciado). Portado para `egui::Panel`/`show`. |
| `cargo fmt --all -- --check` | Limpo, depois de aplicar `rustfmt` a `nist_profiles.rs` e `service/nist.rs`. |
| `cargo test -p dfir-app` (com a GUI) | **104 ok** (9 novos: 8 em `service::tests`, 1 em `gui::evidence`). |
| `cargo test -p dfir-engine -p dfir-protocol` | engine **35 ok** (4 novos de YARA/UTF-16), protocolo **10 ok**. |
| Controle negativo dos testes de UTF-16 | Com o ruleset **original**, `powershell_rule_matches_ascii_and_utf16_content` e `mimikatz_and_ransom_rules_match_utf16_content` **falham** (9 ok, 2 falham); com o ruleset novo passam. Prova que AUD-09 era real e que o teste o detecta. |
| `scripts/doc_gate.py`, `tests/security/fase0_config.py`, `tests/security/rejected_external_tools.py` | PASS |
| `scripts/validate_fase0.py` | 0 erros, 1 aviso (cargo não encontrado pelo `shutil.which` do script). Com o **lock original**, o script novo acusa 3 erros de dessincronização. |

**Limites a conhecer sobre a compilação:** `egui 0.36.2`, `epaint`, `emath` e `wasmtime 45` usam APIs do `std` que não existem no 1.91 (`VecDeque::pop_front_if`/`pop_back_if`, `array_windows`, `Vec::push_mut`, `fmt::from_fn`). Para compilar aqui usei **cópias locais descartáveis** dessas crates com substituições equivalentes, fora do projeto e fora do `Cargo.lock`. A conclusão vale para o código do projeto; **não valida essas crates de terceiros** no 1.97.1.

### Não executado

`cargo clippy`, `cargo nextest`, `cargo deny`, `cargo audit`, `cargo vet`, runtime no Windows e no Linux (`just runtime-test`), qualquer gate com o toolchain 1.97.1. A GUI nunca foi renderizada. Nenhum teste usa um EVTX real (não há fixture: F1-16). Não há teste dedicado para a correção AUD-03a (job marcado `FAILED` quando `begin_immediate` falha dentro de `analyze`), porque exige montar o `AppServices` com o engine; a mudança é pequena e foi compilada, mas não exercitada.

### Defeitos corrigidos nesta data

| ID | Mudança | Teste |
|---|---|---|
| AUD-01 | `Cargo.lock` regenerado; gate de coerência manifesto × lock | `validate_fase0.py` |
| AUD-34 | E0505 em `previously_open_gaps` | compilação + suíte do app |
| (GUI) | `SidePanel`/`show_inside` → `Panel`/`show` | compilação |
| AUD-02 | `destroy_evidence_core`: registra antes de apagar, retomável | 4 testes em `service::tests` |
| AUD-03a | `analyze` marca o job `FAILED` se `begin_immediate` falha | sem teste dedicado |
| AUD-06 | `build_timeline`: ordena antes de truncar | 2 testes |
| AUD-25 | `integrity_failure_error`: falha de auditoria não é engolida | 2 testes |
| AUD-32 | YARA não recarrega/sobrescreve a timeline | 1 teste (`should_reload_timeline`) |
| AUD-09 | `wide ascii` nos padrões, ruleset 1.1.0 | 4 testes + controle negativo |

### Defeitos conhecidos ainda abertos (cada um com ticket)

AUD-04 → F1-17 · AUD-05/18 → F1-18 · AUD-07/08/10 → F1-19 · AUD-15 → F1-22 · AUD-16/17 → F1-24 · AUD-03b-d/11 → F1-10 · AUD-12 → F1-25 · AUD-19/20/27 → F1-26..F1-31 · AUD-13 → F1-15 · AUD-31 → F1-16 · AUD-14 (mensagens genéricas) → F1-23.

### Estado dos tickets existentes

F1-01..F1-05 continuam `IMPLEMENTED / PENDING-HUMAN`: agora compilam e seus testes passam em 1.91.1, mas nenhum gate rodou no toolchain pinado. O backlog vigente está em `docs/FASE1_BACKLOG.md`.


## Atualização 2026-10-03 — CLI headless (ADR-0019) e primeiro teste ponta a ponta com engine real

Ambiente: igual ao de 2026-10-02 (rustc 1.91.1 via `apt`, `--ignore-rust-version`; binário do app compilado **sem a GUI**, com um stub no lugar de `src/gui/`, porque egui 0.36.2 não compila nesse rustc sem as cópias locais já descritas). O engine é o binário real (`dfir-engine`, com `yara-x`). Nada abaixo é `VERIFIED`.

| Execução | Resultado |
|---|---|
| EVTX de teste | `security.evtx` do repositório da crate `evtx` (omerbenamram/evtx, `samples/`), 2.166.784 bytes, SHA-256 `5f29a03c…9a4d8d`. **Não foi incluído no pacote** (licença/proveniência a registrar em F1-16). |
| `scripts/e2e_headless.py` (21 verificações) | **20 passam, 1 falha.** Passam: importação (SHA-256 e tamanho preservados, formato EVTX), listagem, análise EVTX (2261 eventos), timeline não vazia e em ordem cronológica, Event ID extraído em todos os eventos exibidos, varredura YARA, `findings`, `audit-verify`, recusa de descarte com nome errado, descarte, arquivo físico removido, evidência fora do catálogo, auditoria íntegra depois do descarte. **Falha (defeito conhecido):** "sem truncamento": 2261 itens, 1000 exibidos, `truncated=true` (F1-18). |
| Limite de saída forçado (`--max-output-bytes 100000`) | Falha com `OUTPUT_LIMIT: result exceeds max_output_bytes`, como projetado. |
| Tamanho da saída | `results.jsonl` = 2.704.451 bytes para 2261 eventos (~1.196 B/evento): o teto padrão de 4 MiB comporta ~3,5 mil eventos. **AUD-05 confirmado** (era `[P]`). |
| `cargo test -p dfir-app` (sem GUI) | 106 ok (inclui 3 testes novos de parsing da CLI). A árvore completa tem 1 teste de GUI a mais, executado e aprovado em 2026-10-02, **não reexecutado** nesta data. |

**Defeito novo encontrado e corrigido — AUD-35:** `dfir-app import` abortava com `thread 'main' has overflowed its stack` em build debug. Causa: dois buffers `[0_u8; 1024 * 1024]` em `async fn` (`copy_with_hash` e `hash_file` em `service/mod.rs`) entram no state machine do future e, no debug, somam além dos 8 MiB da pilha. Medido: falhava com pilha de 16/32 MiB e passava com 64 MiB; com os buffers na heap passa na pilha padrão de 8 MiB. **Risco aberto:** não verifiquei se a GUI em build debug (`just run`) sofria do mesmo problema (as tarefas do backend rodam em threads de 8 MiB e o future é movido para a heap por `tokio::spawn`); a correção nos buffers vale para ela também, mas a GUI não foi exercitada.

**Não verificado:** o YARA não encontrou nada neste EVTX (esperado, é um log benigno); portanto o teste ponta a ponta **não** demonstra a correção `wide` (AUD-09) com dado real, só os testes sintéticos de 2026-10-02. A execução no toolchain 1.97.1, no Windows e com a GUI continua pendente.

## Atualização 2026-10-04 — sem limite de saída/truncamento (ADR-0020) e refinos

Ambiente: rustc 1.91.1 via `apt`, `--ignore-rust-version`, com as cópias locais descartáveis de `emath`, `epaint`, `egui`, `wasmtime-environ` e `wasmtime` descritas em 2026-10-02 (para compilar a GUI); o binário usado no ponta a ponta foi compilado **sem a GUI**. Nada abaixo é `VERIFIED`.

| Execução | Resultado |
|---|---|
| `cargo test -p dfir-engine -p dfir-protocol` | engine **38 ok** (inclui varredura de arquivo esparso de 300 MiB e contagem exata de ocorrências), protocolo **11 ok** |
| `cargo test -p dfir-app` (árvore completa, com GUI) | **112 ok** |
| `scripts/e2e_headless.py` (engine real, `security.evtx` real de 2.166.784 B, SHA-256 `5f29a03c…`) | **29/29**: 2261 eventos, nenhum descartado, ordem cronológica, Event ID em 100 %, hash/custódia/auditoria íntegros, descarte exige nome exato; Mimikatz em ASCII **e** UTF-16LE detectado com o engine real (confirma AUD-09) e **sem duplicar na reexecução** (confirma AUD-07 corrigido) |
| Falha encontrada e corrigida durante o trabalho | com o teto em `i64::MAX`, **toda análise falhava** (`RFC 8785 canonicalization rejects integers outside ±2^53` ao gravar os limites na auditoria); teto passou a 2^53 − 1 e há teste |
| Falha encontrada e corrigida durante o trabalho | com `MAX_LINE_BYTES` novo mas `DFIR_ENGINE_MAX_LINE_BYTES` ainda em 64 KiB, o app derrubava o job (`engine output line exceeds configured limit of 65536 bytes`); o padrão da configuração agora vem do protocolo |

**Defeitos corrigidos nesta data:** AUD-05 (limite de saída), AUD-06 (truncamento da timeline), AUD-07 (YARA duplicado), AUD-08 (janela de 500), AUD-03 b–d (job órfão, parcial), truncamento silencioso do YARA-X (`max_scan_size`) e corte de 100 ocorrências por padrão.

**Defeito novo aberto — AUD-36:** EVTX truncado (primeiros 1,2 MB de 2,1 MB) termina `COMPLETED` com 1587 de 2261 eventos e nenhum erro; o engine não sinaliza chunks ausentes e o app descarta erros não fatais (AUD-04). Ticket F1-17.

**Não verificado:** GUI renderizada (o filtro com cache e a tabela sem o aviso compilam e passam nos testes unitários, mas não foram vistos em tela); memória com EVTX de milhões de eventos (a timeline é lida inteira para a memória: F1-18 persistência/paginação continua necessária); `RLIMIT_AS` de 1 GiB contra varredura YARA de arquivo muito grande no Linux; clippy, nextest, deny, audit e runtime no Windows/Linux com o toolchain 1.97.1.

## Atualização 2026-10-11 — riscos da ADR-0020 resolvidos (ADR-0021)

Ambiente: rustc 1.91.1 via `apt`, `--ignore-rust-version`, com as cópias locais descartáveis de 5 crates de terceiros (para compilar a GUI); binário do teste ponta a ponta compilado **sem a GUI**; GUI executada em Xvfb com renderização em software (llvmpipe). Nada abaixo é `VERIFIED`.

| Execução | Resultado |
|---|---|
| `cargo test` engine / protocolo / app (com GUI) | **43 / 11 / 123 ok** (+1 `#[ignore]` de desempenho) |
| `scripts/e2e_headless.py` (engine real, `security.evtx` real de 2.166.784 B) | **38/38**: timeline lida por páginas (2261 = `item_count` = total), filtro no banco (`4624` → 583 eventos), arquivo íntegro sem avisos, **EVTX truncado** conclui com `EVTX_TRUNCATED` + `EVTX_RECORDS_MISSING` gravados no job, margem de disco absurda recusa análise e importação, hash/custódia/auditoria íntegros, Mimikatz ASCII e UTF-16 detectado e sem duplicar |
| Desempenho (`timeline_one_million_events_index_and_paging_timing`, build de depuração, 1 CPU) | 1 M de eventos: **indexar ~49 s**, página no início ~4 ms, no meio ~104 ms, no fim ~178 ms, filtro + contagem ~375 ms |
| Guarda de disco com o binário real | importação recusada (margem 1 TiB), análise recusada **antes de criar job** (0 jobs), análise interrompida no meio (margem = livre − 1 MiB) → job `FAILED/DISK_SPACE_LOW` |
| Arquivo grande | 1,1 GiB com marcador nos últimos bytes: antes `Cannot allocate memory` (RLIMIT_AS) e depois `timeout` (60 s); agora **achado** (YARA, Mimikatz) |
| GUI real sob Xvfb (capturas) | lista de evidências; seleção; seção Timeline carrega o último job do banco ("2261 eventos"); rolagem carrega páginas sob demanda; evidência truncada mostra os dois avisos em laranja ("1587 eventos") |
| `cargo deny --workspace check` (0.20.2, banco RustSec de 2026-10-11) | **advisories ok, bans ok, licenses ok, sources ok** após as correções de `deny.toml` e `yoke-derive` |

**Não verificado / limites:** digitar no filtro **na GUI** (o teclado sintético do Xvfb não chegou ao egui; o SQL do filtro tem testes e o filtro roda no ponta a ponta pela CLI); GUI no Windows; guarda de disco no Windows (não mede); `cargo clippy` (não há clippy para o rustc 1.91 no repositório do sistema) e `cargo audit`/`vet`/`nextest`; qualquer gate no toolchain 1.97.1; EVTX com milhões de eventos reais; EVTX truncado exatamente na fronteira de chunk com cabeçalho defasado.

## Atualização 2026-10-11 (2) — testes de GUI e observabilidade (ADR-0022)

Ambiente: igual ao anterior (rustc 1.91.1, `--ignore-rust-version`, cópias locais descartáveis de 5 crates de terceiros para compilar a GUI). Nada abaixo é `VERIFIED`.

| Execução | Resultado |
|---|---|
| `cargo test -p dfir-app` (árvore completa, com GUI) | **134 ok** (+1 `#[ignore]` de desempenho), incluindo **6 testes de GUI com `egui_kittest`** (digitar o filtro e clicar em "Filtrar" envia um único pedido da página 0 com o filtro sem espaços; clique com texto inalterado não consulta o banco; avisos do parser visíveis; página ausente pedida uma vez ao longo de vários quadros; estado vazio; respostas de filtro/job antigos descartadas), 4 de rotação/permissão do log e 1 do relatório de diagnóstico |
| `cargo test -p dfir-engine -p dfir-protocol` | **43 / 11 ok** (inalterado) |
| Binário real (`dfir-app`) | `dfir.log` criado com permissão 0600; eventos `evidence imported` (127 ms), `timeline indexed` (2261 itens, 570 ms), `analysis completed` e `analysis failed` (com a recusa por disco); **o nome do arquivo de evidência não aparece no log nem no `diagnostics`** (0 ocorrências); `dfir-app diagnostics` devolve versões, contagens, jobs por estado, espaço livre e arquivos de log |
| `cargo deny --workspace check` com a dependência nova | advisories, bans, licenses e sources ok |
| `scripts/doc_gate.py`, `scripts/validate_fase0.py`, `tests/security/*.py` | PASS (o `validate_fase0.py` agora também confere que cada ADR ≥ 0018 é citada na spec e que a documentação não cita identificadores removidos) |

**Não verificado:** os mesmos limites de antes (toolchain 1.97.1, Windows, clippy/nextest/`audit`); as demais telas da GUI não têm teste de interface; o `dfir.log` pode conter o texto dos erros (caminhos do sistema do operador).

## Atualização 2026-10-13 — Etapa 1 do plano de conclusão do protótipo (ADR-0023)

Ambiente na data: sem `cargo`/`rustc`; só Python 3 com SQLite 3.45.1. **Os testes Rust foram executados em seguida (2026-10-14), ver a atualização abaixo.** Nada é `VERIFIED` no 1.97.1 pinado.

| Item | Estado | Evidência |
|---|---|---|
| Migration `0010` (`CHECK` de `result_type` estendido) | IMPLEMENTED / verificada em SQLite | 0001–0009 aplicadas em transação com FKs ativas, dados semeados (IOC, gap com `supersedes`, recomendação, três relações), depois a 0010: linhas e relações idênticas; `sequence` e contador do AUTOINCREMENT preservados; 5 índices e 2 triggers recriados; `foreign_key_check` vazio; `integrity_check` ok; `FINDING`/`ROOT_CAUSE`/`PLAN_ITEM` aceitos com `sequence` 5, 6, 7; `BOGUS` recusado; UPDATE/DELETE bloqueados; FKs de relações e de `supersedes` ativas; tabela vazia; **controle:** sem `defer_foreign_keys` a migration falha com `FOREIGN KEY constraint failed` e o rollback preserva as 4 linhas |
| `AnalyticalResultType::{Finding, RootCause, PlanItem}` | IMPLEMENTED / PENDING-HUMAN | `src/persistence/analytical_results.rs` |
| Testes Rust da 0010 e dos tipos | executados em 2026-10-14: verdes | `src/persistence/db.rs` (2), `src/persistence/analytical_results.rs` (+3 de tipo e inserção, ajustado 1) |
| `scripts/doc_gate.py`, `scripts/validate_fase0.py` | ver relatório da rodada | spec com nota (7) citando ADR-0023, cópia da raiz idêntica |

**O que o operador precisa rodar:** `just fmt && just check && just test` (e `just clippy`). Pontos de atenção no primeiro `cargo test`: (1) `sqlx::query(...)` executando o arquivo inteiro da migration com vários comandos; se o sqlx 0.9 recusar, trocar por `sqlx::raw_sql` nos testes de `db.rs`; (2) o teste `embedded_migrator_applies_0010_on_a_fresh_database` confirma que o migrator embutido aplica a 0010 com o pragma dentro da transação; (3) `cargo fmt` pode reformatar trechos dos testes novos (escritos à mão).

## Atualização 2026-10-14 — Etapas 1 e 2 verificadas em rustc 1.91.1; Finding e causa-raiz (F1-25)

Ambiente: `rustc`/`cargo` **1.91.1** via `apt` (`--ignore-rust-version`: o 1.97.1 pinado não está disponível). Para compilar a GUI, `emath`, `epaint` e `egui` 0.36.2 foram usadas em **cópias locais descartáveis** com 6 substituições equivalentes de APIs do `std` ausentes no 1.91 (`VecDeque::pop_front_if`/`pop_back_if`, `array_windows`, `Vec::push_mut`), fora do projeto e do `Cargo.lock`; isso não valida essas crates no 1.97.1. Nada abaixo é `VERIFIED` no sentido do AGENTS §10.

| Item | Estado | Evidência |
|---|---|---|
| Etapa 1: migration 0010 e tipos | IMPLEMENTED / PENDING-HUMAN | `cargo test` 127 ok (1 ignorado) antes da Etapa 2, incluindo `migration_0010_keeps_rows_relations_indexes_and_append_only_guards` e `embedded_migrator_applies_0010_on_a_fresh_database`. O `rustfmt` reformatou 3 `assert!` do teste de `db.rs` (corrigidos) |
| F1-25: `persistence/findings.rs` | IMPLEMENTED / PENDING-HUMAN | 7 testes: vocabulário, persistência com ligações, recusas (título, severidade `NONE`, sem evidência, NIST inválido, tipo e incidente errados), revisão por `supersedes`, regras de nível de causa e lacunas, matriz de risco |
| F1-25: `service/findings.rs` | IMPLEMENTED / PENDING-HUMAN | 2 testes: ida e volta por `findings_view` e recusas de vocabulário e incidente |
| F1-25: aba "Achados" (`gui/findings.rs`, `backend.rs`, `incidents.rs`, `mod.rs`) | IMPLEMENTED / PENDING-HUMAN | 6 testes (2 de lógica, 4 `egui_kittest`): exibição, pedido único ao abrir, envio dos valores digitados e erro mantendo o formulário |
| Suíte completa | verde | `cargo test -p dfir-app --bin dfir-app`: **154 passed, 0 failed, 1 ignored** |
| `cargo clippy --all-targets --all-features -- -D warnings` | limpo | após remover 3 funções de serviço usadas só por testes, trocar atribuições após `Default` num teste e anotar 2 casts em `infrastructure/disk.rs` (anteriores) |
| `cargo fmt --all -- --check` | limpo | |

**Não executado:** o binário com a GUI numa janela real (Xvfb); `cargo test` do engine e do workspace inteiro (não alterados); qualquer build no toolchain 1.97.1.

**Limites conhecidos de F1-25:** a interface não expõe `supporting_result_ids` nem a revisão de um achado. O `AGENTS §10` pede `just fmt && just check && just test && just clippy` no 1.97.1 antes de chamar de `VERIFIED`.

## Atualização 2026-10-14 (2) — Etapa 3: item de plano (F1-29, absorve F1-06)

Ambiente como na atualização anterior (rustc 1.91.1 via `apt`, `--ignore-rust-version`, cópias locais descartáveis de `emath`/`epaint`/`egui`). Nada abaixo é `VERIFIED` no sentido do AGENTS §10.

| Item | Estado | Evidência |
|---|---|---|
| `persistence/plan_items.rs` | IMPLEMENTED / PENDING-HUMAN | 7 testes: vocabulário e domínios de criptografia, achado ligado × recomendação geral, obrigatórios, detalhes de proteção, NIST e vínculos inválidos, revisão e ciclos de dependência, cobertura com domínios vazios |
| `service/plan.rs` | IMPLEMENTED / PENDING-HUMAN | 3 testes: ordenação por domínio e prioridade com cobertura, revisão que mostra só a vigente, recusas de vocabulário e de detalhes de proteção ausentes |
| Aba "Plano" (`gui/plan.rs`, `backend.rs`, `incidents.rs`, `mod.rs`) | IMPLEMENTED / PENDING-HUMAN | 7 testes (3 de lógica, 4 `egui_kittest`): exibição com cobertura e marca de recomendação geral, pedido único ao abrir, envio dos detalhes de proteção só nos domínios que os exigem, erro mantendo o formulário |
| Suíte completa | verde | `cargo test -p dfir-app --bin dfir-app`: **171 passed, 0 failed, 1 ignored** |
| `cargo clippy --all-targets --all-features -- -D warnings` | limpo | |
| `cargo fmt --all -- --check` | limpo | |

Corrigido durante a verificação: um erro de lifetime em `show_items` (`map_or` com `&'static str`), apontado pelo compilador.

**Não executado:** o binário com a GUI numa janela real; `cargo test` do engine e do workspace; qualquer build no 1.97.1 pinado.

## Atualização 2026-10-15 — Etapa 4: visão NIST por Function (F1-30)

Ambiente como nas atualizações anteriores (rustc 1.91.1 via `apt`, `--ignore-rust-version`, cópias locais descartáveis de `emath`/`epaint`/`egui`). Nada abaixo é `VERIFIED` no sentido do AGENTS §10.

| Item | Estado | Evidência |
|---|---|---|
| `persistence/nist_functions.rs` | IMPLEMENTED / PENDING-HUMAN | 4 testes: contagens por Function na ordem oficial, Functions sem dados com zeros, prefixos e prioridades desconhecidos ignorados sem arquivar errado, estados do gap |
| `service/nist.rs::function_view` | IMPLEMENTED / PENDING-HUMAN | 1 teste: sem profiles, com profiles e sem gap, gaps em dia, gaps desatualizados após nova versão do Current e recalculo |
| Seção "Visão por Function" (`gui/nist.rs`, `backend.rs`, `mod.rs`) | IMPLEMENTED / PENDING-HUMAN | 4 testes `egui_kittest`: pedido único ao abrir (quatro comandos), seis Functions + Tier + nota de que não há score, aviso de gap desatualizado, atualização depois de recalcular os gaps |
| Suíte completa | verde | `cargo test -p dfir-app --bin dfir-app`: **180 passed, 0 failed, 1 ignored** |
| `cargo clippy --all-targets --all-features -- -D warnings` | limpo | |
| `cargo fmt --all -- --check` | limpo | |

**Não executado:** o binário com a GUI numa janela real; `cargo test` do engine e do workspace; qualquer build no 1.97.1 pinado.

## Atualização 2026-10-16 — Etapa 5: gerador dos entregáveis (F1-07)

Ambiente como nas atualizações anteriores (rustc 1.91.1 via `apt`, `--ignore-rust-version`, cópias locais descartáveis de `emath`/`epaint`/`egui`). Nada abaixo é `VERIFIED` no sentido do AGENTS §10.

| Item | Estado | Evidência |
|---|---|---|
| `service/deliverables/markdown.rs` | IMPLEMENTED / PENDING-HUMAN | 5 testes: escape de Markdown, HTML, links e caracteres de controle; trechos de código; tabelas com largura fixa mesmo com células hostis; citação de ids; seção sem registro |
| `service/deliverables/` (coleta, cinco renderizadores, escrita, manifest, auditoria) | IMPLEMENTED / PENDING-HUMAN, parcial | 8 testes: cinco documentos com escopo EVTX declarado, incidente vazio sem afirmar causa, determinismo e independência de ordem, texto hostil neutralizado, ids de origem por documento, escrita com hashes e idempotência, recusas de pasta (armazenamento interno, id inseguro, link simbólico, caminho ocupado), geração a partir do banco com auditoria sem caminhos |
| Aba "Entregáveis" (`gui/deliverables.rs`, `backend.rs`, `incidents.rs`, `mod.rs`) | IMPLEMENTED / PENDING-HUMAN | 6 testes `egui_kittest`: nada pedido sem clique, pré-visualização pedida uma vez, lista e conteúdo dos documentos, resposta atrasada de outro incidente ignorada, geração com pasta e hashes, erro |
| `dfir-app deliverables` (`cli.rs`, `main.rs`) | IMPLEMENTED / PENDING-HUMAN | 1 teste: seis arquivos, padrão na pasta de exportações, incidente inexistente falha |
| Suíte completa | verde | `cargo test -p dfir-app --bin dfir-app`: **200 passed, 0 failed, 1 ignored** |
| `cargo clippy --all-targets --all-features -- -D warnings` | limpo | após trocar atribuições posteriores a `Default` por inicialização com `..Default::default()` em dois testes |
| `cargo fmt --all -- --check` | limpo | |

Defeito encontrado ao **ler os documentos renderizados** (não pelas asserções): "Ativos afetados" e "Dados afetados" do Entregável 1 saíam vazios porque o payload de impacto traz lista. Corrigido com `field_text` e coberto por asserção.

**Não executado:** o binário com a GUI numa janela real (inclusive o seletor nativo de pasta); `cargo test` do engine e do workspace; qualquer build no 1.97.1 pinado; geração dos documentos sobre um incidente com dados reais de EVTX (isso é a Etapa 7).

## Atualização 2026-10-18 — Etapa 6: proveniência, lacunas e máscara (F1-31 e F1-32)

Ambiente como nas atualizações anteriores (rustc 1.91.1 via `apt`, `--ignore-rust-version`, cópias locais descartáveis de `emath`/`epaint`/`egui`). Nada abaixo é `VERIFIED` no sentido do AGENTS §10.

| Item | Estado | Evidência |
|---|---|---|
| Migration `0011` (`SOURCE_GAP`, `provided_by`, `contains_personal_data`) | IMPLEMENTED / verificada | Em SQLite 3.45.1 (11 verificações, com os tipos da 0010 semeados) e em Rust: `migration_0011_keeps_rows_adds_source_gap_and_the_evidence_columns` e `embedded_migrator_applies_0011_on_a_fresh_database` |
| `persistence/source_gaps.rs`, `service/gaps.rs` | IMPLEMENTED / PENDING-HUMAN | 4 testes de persistência (vocabulário, período, `OTHER` sem nota, revisão) e 2 de serviço (ordem estável, só a revisão vigente, recusas) |
| `service/provenance.rs` e importação | IMPLEMENTED / PENDING-HUMAN | 5 testes: sem declaração grava `NOT_PROVIDED` e nenhuma data de coleta inventada; com declaração grava e devolve; dado inválido recusado antes de copiar; campos em branco ausentes; linhas antigas e data de importação não contam como declaradas |
| Aba "Lacunas" (`gui/gaps.rs`) | IMPLEMENTED / PENDING-HUMAN | 4 testes (vocabulário, pedido único, exibição, envio e erro) |
| Aba de evidências: formulário, "Tratamento", resumo | IMPLEMENTED / PENDING-HUMAN | 4 testes `egui_kittest`: proveniência enviada e formulário limpo ao importar, formulário em branco, tabela por tratamento, resumo com "não informado" |
| `service/deliverables/redaction.rs` e opções | IMPLEMENTED / PENDING-HUMAN | 3 testes de detecção e marcadores; testes do gerador: nenhum valor original nos cinco documentos nem no manifest, mesmos marcadores entre documentos, determinismo e ordem de entrada, contagens no manifest, escolha na auditoria |
| Opção de máscara na aba "Entregáveis" e `--no-redaction` | IMPLEMENTED / PENDING-HUMAN | 2 testes novos: padrão mascarar e confirmação para desligar; aviso e confirmação na tela. 1 teste de CLI para `import` com proveniência |
| Suíte completa | verde | `cargo test -p dfir-app --bin dfir-app`: **230 passed, 0 failed, 1 ignored** |
| `cargo clippy --all-targets --all-features -- -D warnings` | limpo | |
| `cargo fmt --all -- --check` | limpo | |

Defeito real encontrado pelos testes: a lista de evidências do Entregável 1 seguia a ordem de entrada (a saída variava com a ordem dos dados). Corrigido em `render_all` e coberto por teste.

**Não executado:** o binário com a GUI numa janela real (inclusive os seletores nativos de arquivo e de pasta); `cargo test` do engine e do workspace; qualquer build no 1.97.1 pinado; geração dos documentos sobre dados reais de EVTX (Etapa 7).

## Atualização 2026-10-23 — Etapa 7: incidente de demonstração, e2e dos cinco documentos e reconciliação (F1-11 e F1-12, reduzidos)

Ambiente: rustc/cargo 1.91.1 (apt), `--ignore-rust-version`; cópias locais descartáveis de `emath`/`epaint`/`egui` e, para compilar o engine, de `wasmtime-environ`/`wasmtime` (substituições equivalentes de `fmt::from_fn` e das APIs do `std` ausentes no 1.91), fora do projeto e do `Cargo.lock`. Decisões e detalhes: ADR-0023 (atualização da Etapa 7).

| Item | Estado | Evidência |
|---|---|---|
| Defeito: `analyze` com `ColumnNotFound("source")` | corrigido | o e2e anterior à correção terminava com a análise falhando; `evidence_columns!` em `service/mod.rs`; `analyze_loads_the_evidence_row_before_validating_the_kind` falha com a consulta antiga e passa com a nova |
| Defeito: evidência descartada sumia dos documentos | corrigido | `a_destroyed_evidence_is_still_cited_with_its_hash_and_custody` (hash, `DESTROYED`, nota e ausência da frase falsa); `list_evidence_including_destroyed`, `EvidenceSection::is_destroyed`, `report.rs` |
| `service/demo.rs`, `dfir-app demo`, `just demo` (F1-11) | IMPLEMENTED / PENDING-HUMAN | 7 testes do seed + 1 de parsing da CLI; execução real com `security.evtx`: 2 jobs reais concluídos e as contagens do cenário |
| Classe do dado `DEMO` (`import_evidence_classified`) | IMPLEMENTED / PENDING-HUMAN | teste: `status = DEMO` e `DEMO_DATASET_SEEDED` na auditoria; classe desconhecida recusada antes de copiar |
| `scripts/e2e_headless.py` (cinco documentos) | verde | **79/79** com engine e `security.evtx` reais (SHA-256 `5f29a03c…9a4d8d`): 38 anteriores + 41 novas |
| `docs/DEMO.md`, `docs/OPERATION.md` | escritos | comandos de volume e de backup **não executados** neste ambiente |
| Suíte do aplicativo | verde | `cargo test -p dfir-app`: **240 passed, 0 failed, 1 ignored** |
| `cargo clippy -p dfir-app --all-targets --all-features -- -D warnings` | limpo | |
| `rustfmt --edition 2024 --check` | limpo | |
| Reconciliação dos documentos vivos | feita | README (Estado do MVP e comandos), CHANGELOG, backlog, RTM, DEMAND_TRACE, RELEASE_GATE, fixtures, prompt de continuidade |

Defeitos reais encontrados por rodar o caminho inteiro (nenhum dos dois aparecia nos 230 testes anteriores): `analyze` quebrado desde a Etapa 6 e evidência descartada omitida dos documentos. Limite documentado: a matriz de proteção do Entregável 5 agrupa pelo texto exato da classe do dado.

**Não executado:** o binário com a GUI numa janela real; qualquer build no 1.97.1 pinado; a execução com a rede cortada (o aplicativo não usa rede, mas isso não foi medido); `cargo test` do engine e do workspace (o clippy 1.91 acusa 3 `assert!(true)` antigos nos testes de `protocol`); os comandos de LUKS e BitLocker de `docs/OPERATION.md`.

**Pendente (fora do plano da Etapa 7):** cópia e verificação automáticas do banco antes de migration (RF-032); F1-26, F1-27 e F1-28 (a especificação os classifica como MVP); filtros estruturados da timeline; verificação do operador no toolchain pinado (F1-14).

## Atualização 2026-10-27 — contenção do engine, evento normalizado, detecção por regras, cópia antes de migration

Pedido do operador: concluir as etapas da análise de contenção do engine e finalizar os pedidos e restrições da demanda original. Decisões: ADR-0024, ADR-0025, ADR-0026. Catálogo de regras: `docs/DETECTION_RULES.md`.

### O que foi executado (e onde)

Ambiente: Linux x86_64 (kernel 6.18, Landlock ABI 7), **rustc/cargo 1.97.1 — o toolchain pinado** (instalado do conda-forge; não é o apt 1.91.1 das rodadas anteriores, e **não precisou de cópias locais de crates**). `Cargo.lock` com `--locked`.

| Verificação | Resultado |
|---|---|
| `cargo test --locked --workspace` | app **288 ok** (1 ignorado), engine ~~**80 ok**~~ **83 ok** + **7** de integração do sandbox [corrigido em 2026-11-04, Gap origem: REL-02; 83 é a contagem do relatório de 2026-10-27, da rodada de 2026-10-29 e do código], protocolo **26 ok** |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | limpo (3 avisos `assertions_on_constants` que **já existiam** nos testes de `protocol` foram corrigidos) |
| `cargo fmt --all -- --check` | limpo |
| `scripts/e2e_headless.py` com engine e `security.evtx` reais | **110/110** (eram 79) |
| `cargo check --target x86_64-pc-windows-gnu -p dfir-engine` | compila sem avisos (type-check; **nada foi executado em Windows**) |
| `scripts/validate_fase0.py`, `scripts/doc_gate.py` | ver o relatório de alterações |

**Linha de base no toolchain pinado, antes de qualquer mudança:** o pacote 2026-10-23 compilou e passou (app 240, engine 43, protocolo 11) em `--locked` com o 1.97.1. Até aqui a documentação afirmava que "nada é `VERIFIED` no 1.97.1" por nunca ter sido tentado.

### Defeitos reais encontrados ao rodar o caminho inteiro (nenhum aparecia nos testes anteriores)

1. **O engine entrava em pânico sob seccomp** ao criar o runtime do tokio: o driver de sinais (ligado pela unificação de features só quando o workspace é compilado junto) cria um `socketpair`. `cargo test -p dfir-engine` não reproduz; o teste de integração e o e2e (build do workspace) sim. `socketpair` ficou liberado, deliberadamente (ADR-0024 D-3).
2. **`analysis_jobs.kind` tinha `CHECK` com só dois tipos**: a análise de detecção falhava ao gravar o job. Só apareceu com o banco real. A migration 0012 recria a tabela (com teste de preservação de dados).
3. **Um filtro de timeline inválido derrubava a tela** (estado de erro escondia a própria caixa de filtro). A interface passou a validar a sintaxe antes de enviar.
4. **Evidência descartada** aparecia na nova seção 4.3 como "nenhuma evidência citada" e, pior, como "nenhum candidato" para candidatos apagados com ela. Estado `Discarded` explícito.
5. `verify_published` (backup) abria o arquivo adulterado antes de conferir o hash.

### Pendente

- Execução do operador em **Windows** (Job Object nunca rodou) e em **Linux aarch64** (seccomp só x86_64; `required` recusa analisar).
- Verificação em **incidente real**; as regras foram vistas só em EVTX real sem ataque e em eventos sintéticos.
- F1-14 (CI automático). ~~F1-15 (`cargo audit`/`cargo deny` não reexecutados)~~: **superado em 2026-10-29**, ver a atualização abaixo; o `cargo deny` **já falhava** no lock de 2026-10-27 por avisos novos do RustSec (não por esta rodada).
- F1-27 (adiado), F1-16 (fixtures EVTX versionadas), F1-19 (vínculo), F1-24 (custódia visível).
- A GUI **não foi aberta numa janela real** (só `egui_kittest`).

## Atualização 2026-10-29 — dependências (F1-15)

`cargo audit` (base RustSec de 2026-10-29, 1 386 avisos) achou **9 vulnerabilidades** no lock de 2026-10-27: 8 do `wasmtime 45.0.3` (preso por `yara-x 1.20.0`) e 1 do `ctutils`, além de `tokio` *unsound* e `clipboard-win` *yanked*. O `cargo deny --workspace check` falhava. Aplicado `cargo update` completo (111 pacotes): `yara-x 1.22.1`, `wasmtime 48.0.8`, `tokio 1.54.1`, `ctutils 0.4.7`. Depois: `cargo audit` **0 vulnerabilidades**; `cargo deny --workspace check` **ok** (`advisories`, `bans`, `licenses`, `sources`).

Com o lock novo, **neste pacote exato**, em Linux x86_64 e rustc 1.97.1: `fmt --check`, `clippy -D warnings` (limpo), `test --workspace` (app 288, engine 83 + 7 do sandbox, protocolo 26), e2e 110/110, e o engine real sobre o `security.evtx` sob `required` (Landlock ABI 7 + seccomp) com o JIT do wasmtime 48, com resultados idênticos (2 261 eventos; 6 candidatos). **Não repetido:** Windows (Job Object) e aarch64. **Não atualizados, de propósito:** `eframe`/`egui` 0.37.0, `evtx` 0.14.0 e `serde_json_canonicalizer` 0.4.1 (versões 0.x; a última alimenta a cadeia de auditoria).

**Pendente, decisão sua:** as 3 exceções informativas novas do `deny.toml` (`zvariant`, `zbus_names`, `zvariant_derive`, só da UI). **Regra daqui em diante:** atualizar `yara-x`/`wasmtime` exige repetir `cargo test --workspace` (inclui `engine/tests/sandbox.rs`) e o e2e, porque o filtro seccomp e o Landlock foram validados contra o JIT de cada versão.

