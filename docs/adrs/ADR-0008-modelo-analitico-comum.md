<!-- doc-type: decision | status: accepted -->
# ADR-0008 — Modelo analítico comum e versionamento de resultados (F1-01)

## Status

ACEITA — 2026-08-25

## Contexto

A Fase 1 introduz cinco capacidades analíticas (F1-02 YARA-X, F1-03 gap NIST, F1-04 ATT&CK/caminho crítico/impacto, F1-05 IOCs/recomendações) que, segundo o próprio ticket F1-01, devem "compartilhar identificadores, evidência, confiança, origem e versão de regra/modelo". A entidade lógica `Finding`/`ATTACKMapping`/`Assessment`/`Gap`/`IOC` já aparece no modelo de dados da especificação canônica (seção 22.1), mas sem um contrato físico único: sem F1-01, cada ticket reimplementaria por conta própria identidade, versionamento, ligação a evidência/incidente, confiança e trilha de auditoria — violando exatamente o "compartilhem" que o próprio ticket pede.

O código existente não tinha um local natural para esse contrato: `web/mod.rs` já está com 1420 linhas (handlers HTTP + DTOs + SQL inline) e `persistence/` até então continha apenas `audit.rs` e `db.rs`. Havia um diretório `src/domain/` vazio, sem `mod domain;` em `main.rs` — vestígio inerte de um scaffolding anterior, sem uso ou documentação associada.

## Decisão

- Nova tabela `analytical_results` (migration `0006_analytical_results.sql`), **append-only** (triggers `BEFORE UPDATE`/`BEFORE DELETE` com `RAISE(ABORT, ...)`, mesmo padrão de `audit_events`/`chain_of_custody`). Reprocessamento nunca faz `UPDATE`: insere uma nova linha com `supersedes` apontando para a anterior; o "resultado atual" de uma `correlation_key` é resolvido por consulta (`NOT EXISTS` sobre `supersedes`), não por um ponteiro mutável `superseded_by` — o que eliminaria a necessidade do próprio trigger append-only.
- Colunas compartilhadas: `result_type`, `correlation_key` (chave estável que agrupa versões sucessivas do mesmo achado lógico — cada F1-0x decide como derivá-la), `evidence_id`/`incident_id` (ao menos um obrigatório via `CHECK`), `analysis_job_id` (referência **lógica**, não FK — ver abaixo), `rule_or_model_id`/`rule_or_model_version`, `confidence` (`LOW`/`MEDIUM`/`HIGH`/`CONFIRMED`, vocabulário confirmado contra as seções 13/14/15/16/17 da spec canônica), `payload`/`provenance` (JSON canonicalizado JCS), `supersedes`.
- `analysis_job_id` não é uma foreign key: `DELETE /api/evidence/{id}` (rota já existente e ACCEPTED) apaga fisicamente as linhas de `analysis_jobs` associadas, mas preserva a linha de `evidence` (marca `preservation_status='DESTROYED'`). Um resultado analítico é um artefato forense e não pode depender da sobrevivência do job efêmero que o produziu — a mesma razão pela qual `audit_events`/`chain_of_custody` nunca são apagados.
- Política de campos obrigatórios por `result_type` (`requires_evidence`/`requires_incident`/`requires_confidence`) é validada em **Rust**, não em `CHECK` de schema — permite que F1-02..F1-05 refinem a política sem nova migration.
- Nova tabela `analytical_result_relations` (`DERIVED_FROM`, `SEQUENCE_NEXT`) para o caminho crítico (F1-04) e para recomendações que referenciam os achados que as sustentam (F1-05) — tabela própria em vez de array JSON, preservando integridade referencial via FK.
- Módulo novo `src/persistence/analytical_results.rs`, seguindo exatamente os padrões de `audit.rs` (mesma disciplina de canonicalização JCS, mesma forma de transação `BEGIN IMMEDIATE`/commit/rollback, mesmo estilo de query com tuplas em vez de `FromRow` derivado). Cada insert grava um evento `ANALYTICAL_RESULT_RECORDED` na cadeia de auditoria, na mesma transação.
- `src/domain/` (vazio, órfão, sem `mod domain;`) foi removido. O novo módulo foi colocado em `src/persistence/`, não em um novo `src/domain/`, para não introduzir uma camada arquitetural (domain vs. persistence) que não existe em nenhum outro lugar do código — menor mudança estrutural suficiente.
- Fora de escopo, deliberadamente: `CurrentProfile`/`TargetProfile`/`TierAssessment` (seção 13) não entram neste envelope — são dados de configuração/referência da organização, não achados versionados com confiança; ficam a critério de F1-03. `NistGap` (RF-010, "reproduzível a partir dos dados") é o que se encaixa no contrato e está incluído.

### Alternativas consideradas

- **Uma tabela física por categoria (`findings`, `attack_mappings`, `gaps`, `iocs`, `remediations`), espelhando a seção 22.1 literalmente.** Rejeitada: cada uma reimplementaria identidade/versionamento/confiança/auditoria isoladamente — exatamente o oposto do "compartilhem" pedido pelo ticket. O envelope comum + `payload` JSON tipado por cada F1-0x atinge o mesmo resultado sem duplicação.
- **`superseded_by` como ponteiro mutável na linha antiga, atualizado quando uma nova versão chega.** Rejeitada: exigiria `UPDATE` na tabela, quebrando a garantia append-only e os triggers que a impõem. A resolução por consulta (`NOT EXISTS`) dá o mesmo resultado sem mutação.
- **Extrair `ensure_jcs_safe`/canonicalização para um módulo compartilhado com `audit.rs`.** Considerada e rejeitada nesta rodada: `audit.rs` é uma capacidade ACCEPTED, de integridade forense, e este ambiente de implementação não tem `cargo` disponível para confirmar via `cargo nextest run` que uma extração não altera nenhum hash produzido. A lógica foi deliberadamente duplicada (pequena, ~15 linhas) em vez de arriscar uma refatoração não verificável em código de integridade já aceito. Fica como candidato a um ticket futuro, testado com os gates Rust reais.
- **Relação `related_result_ids` como array JSON dentro do próprio `analytical_results`.** Rejeitada em favor de uma tabela dedicada (`analytical_result_relations`): permite integridade referencial via FK e consultas "quem referencia X" sem varrer payloads — mais consistente com o resto do schema, que usa tabelas relacionais explícitas (`chain_of_custody`, `audit_events`) em vez de blobs.

## Consequências

Positivas:

- F1-02 a F1-05 podem ser implementados sem reinventar identidade, versionamento, ligação a evidência/incidente, confiança ou trilha de auditoria — cada um define apenas seu `payload` tipado e sua política de campos obrigatórios.
- Histórico de reprocessamento é preservado por construção (nunca há `UPDATE` na tabela).
- `analytical_result_relations` já está pronta para F1-04 (sequenciamento) e F1-05 (recomendação → achados) sem exigir uma segunda migration.
- Diretório órfão `src/domain/` removido.

Negativas / dívida técnica assumida:

- Duplicação intencional do helper JCS entre `audit.rs` e `analytical_results.rs` (ver alternativas consideradas) — candidato a consolidação futura, testada.
- A política de campos obrigatórios por tipo (`requires_confidence`/`requires_evidence`/`requires_incident`) é a leitura de F1-01 sobre as seções 13–20 da spec canônica; F1-02..F1-05 podem precisar ajustá-la ao implementar cada categoria — o código foi desenhado exatamente para permitir isso sem migration.

## Evidência de execução

Testado com `cargo nextest run` **não disponível** neste ambiente de implementação (sem `cargo`/`rustc` instalados). O módulo (`src/persistence/analytical_results.rs`) inclui uma suíte de testes (`#[cfg(test)] mod tests`) cobrindo: round-trip de enums, canonicalização JCS (ordem de chaves e rejeição de inteiros fora de `±2^53`), exigência de evidência/incidente, exigência de confiança por tipo, gravação do evento de auditoria, preservação de histórico via `supersedes` (incluindo contagem total de linhas), rejeição de `supersedes` com `correlation_key` divergente, comportamento append-only da tabela (UPDATE/DELETE rejeitados pelos triggers) e relações entre resultados. Esta suíte foi revisada manualmente linha a linha contra as assinaturas reais de `audit.rs`, `persistence/db.rs` e os schemas de `evidence`/`incidents`/`audit_events`, mas **não foi compilada nem executada** — status **IMPLEMENTED / PENDING-HUMAN** (ver `docs/FASE1_STATUS.md`). A execução de `cargo fmt --all -- --check`, `cargo check --locked --all-targets`, `cargo clippy --locked --all-targets --all-features -- -D warnings` e `cargo nextest run --locked` fica para o operador do projeto.
