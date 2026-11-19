<!-- doc-type: reference | status: current -->
# API de serviço — contrato interno de `src/service/*`

> Gap origem: AUS-05 (2026-11-04). Tabelas geradas das assinaturas públicas do código; descrições vêm dos comentários `///` (onde não há, aparece "—").

**Não há API HTTP.** O servidor HTTP, as rotas e a porta de rede foram removidos pela ADR-0017. A interface nativa (`src/gui/`) e a CLI (`src/main.rs`) chamam estas funções diretamente, no mesmo processo. Decisão: `docs/adrs/ADR-0017-interface-nativa-eframe-egui.md`; arquitetura da interface: `docs/NATIVE_UI_ARCHITECTURE.md`.

Convenções: o primeiro parâmetro de quase todas as funções é `services: &AppServices` (pool SQLite, caminhos do storage e limites do engine) e foi omitido das tabelas; as funções que recebem `db`/`pool` mostram o parâmetro. O tipo de erro é `AppError`, via `AppResult<T>`. Só aparecem funções públicas (`pub`); as `pub(crate)` não fazem parte do contrato.

## Funções por módulo

### `src/service/mod.rs` — Evidência, análise, linha do tempo e incidentes

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `async list_evidence` | `—` | `AppResult<Vec<EvidenceResponse>>` | — |
| `async destroy_evidence` | `evidence_id: &str` | `AppResult<()>` | — |
| `async link_evidence_to_incident` | `evidence_id: &str, incident_id: &str` | `AppResult<()>` | Vincula a evidência ao incidente de forma idempotente, em transação, gravando `EVIDENCE_LINKED_TO_INCIDENT` na auditoria (F1-19). |
| `async evidence_for_incident` | `incident_id: &str` | `AppResult<Vec<EvidenceResponse>>` | Evidências vigentes do incidente, sem janela de tempo (F1-19). |
| `async import_evidence_with_provenance` | `source_path: &Path, max_upload_bytes: u64, declared: &provenance::EvidenceProvenance` | `AppResult<EvidenceResponse>` | Importa uma evidência a partir de um caminho local já escolhido pelo operador (diálogo nativo de arquivo, `src/gui`). |
| `async import_evidence_classified` | `source_path: &Path, max_upload_bytes: u64, declared: &provenance::EvidenceProvenance, data_class: &str` | `AppResult<EvidenceResponse>` | Como `import_evidence_with_provenance`, com a classe do dado explícita. |
| `async analyze` | `evidence_id: &str, kind: &str, max_output_bytes: Option<u64>, max_items: Option<u64>` | `AppResult<AnalyzeResponse>` | Executa uma análise e registra o resultado no log (job, tipo, itens, avisos e duração total, ou o erro). |
| `async recover_orphaned_jobs` | `db: &SqlitePool` | `AppResult<u64>` | Recupera jobs que ficaram `PENDING`/`PROCESSING` porque o app anterior foi fechado ou morreu no meio da análise (F1-10, AUD-03). |
| `async latest_timeline_job` | `evidence_id: &str` | `AppResult<Option<String>>` | O job EVTX concluído mais recente da evidência, se houver (a GUI o usa para mostrar a timeline ao selecionar a evidência, sem reanalisar). |
| `async timeline_page` | `job_id: &str, offset: u64, limit: u64, filter: Option<&str>, want_total: bool` | `AppResult<TimelinePage>` | Uma página da timeline de um job, ordenada por UTC. |
| `async evidence_analytical_results` | `evidence_id: &str` | `AppResult<Vec<AnalyticalResultResponse>>` | — |
| `async create_incident` | `name: &str` | `AppResult<IncidentResponse>` | — |
| `async list_incidents` | `—` | `AppResult<Vec<IncidentResponse>>` | — |
| `async diagnostics_report` | `db: &SqlitePool, paths: &StoragePaths, settings: serde_json::Value` | `AppResult<serde_json::Value>` | Relatório de diagnóstico para suporte (ADR-0022): versões, contagens do banco, falhas recentes por **código**, espaço livre e arquivos de log. |

`timeline_page` falha fechado quando os warnings persistidos contêm JSON inválido, `warning_count` negativo ou contagem divergente do array. `warnings = NULL` com contagem zero continua válido para jobs legados; nenhum detalhe inválido vira lista vazia (`EVD-021`).

Ao indexar ou reindexar `results.jsonl`, JSON inválido ou profundidade acima de `MAX_JSON_DEPTH` produz erro com o número da linha e aciona rollback, preservando eventos e índice anteriores (`EVD-025`). Toda linha de resultado também precisa conter `items` como array, conforme `ResultMessage`: campo ausente ou de outro tipo falha nos leitores de timeline e analíticos, enquanto `items: []` continua válido (`EVD-026`).

`evidence_analytical_results` e as leituras vigentes por incidente também falham fechado quando `payload` ou `provenance` persistidos contêm JSON inválido. A conversão usa `TryFrom` e propaga um diagnóstico específico em vez de produzir `null` (`EVD-023`).

### `src/service/detection.rs` — Detecção por regras e candidatos (ADR-0025)

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `async run_detection` | `evidence_id: &str` | `AppResult<DetectionRun>` | Roda as regras de detecção sobre a evidência e grava os candidatos. |
| `async list_candidates` | `evidence_id: &str, status: Option<&str>` | `AppResult<Vec<CandidateView>>` | — |
| `async candidate_counts` | `evidence_id: &str` | `AppResult<CandidateCounts>` | — |
| `async confirm_candidate` | `candidate_id: &str, incident_id: &str, analyst_note: Option<&str>` | `AppResult<ConfirmOutcome>` | Confirma o candidato: cria o mapeamento ATT&CK, o achado e os IOCs ligados ao incidente escolhido, e marca o candidato como confirmado. |
| `async reject_candidate` | `candidate_id: &str, reason: &str` | `AppResult<()>` | Rejeita o candidato. |

`list_candidates` e `confirm_candidate` falham fechado quando `event_refs` ou IOCs persistidos contêm JSON inválido; não convertem o dado em lista vazia. A confirmação valida ambos antes de criar mapeamento, achado ou IOC (regressões em `EVD-020`).

### `src/service/findings.rs` — Achados, mapeamentos ATT&CK, caminho crítico, impacto, IOCs e recomendações

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `async create_attack_mapping` | `incident_id: &str, evidence_id: Option<&str>, tactic: &str, technique_id: &str, sub_technique_id: Option<&str>, basis: &str, confidence: &str, analyst_note: Option<&str>` | `AppResult<String>` | — |
| `async list_attack_mappings` | `incident_id: &str` | `AppResult<Vec<AnalyticalResultResponse>>` | — |
| `async create_critical_path_stage` | `incident_id: &str, evidence_id: Option<&str>, sequence_index: i64, stage_label: &str, basis: &str, confidence: &str` | `AppResult<String>` | — |
| `async link_critical_path_stages` | `incident_id: &str, from_stage_id: &str, to_stage_id: &str` | `AppResult<()>` | Ambos os estágios devem ser `CRITICAL_PATH_LINK` do incidente informado; relações entre incidentes falham antes da escrita (`EVD-030`). |
| `async critical_path` | `incident_id: &str` | `AppResult<CriticalPathView>` | Grafo do caminho crítico: estágios vigentes + arestas `SEQUENCE_NEXT` entre eles. |
| `async create_impact_assessment` | `incident_id: &str, availability_impact: &str, confidentiality_impact: &str, integrity_impact: &str, affected_assets: &str, affected_data: &str, operational_disruption: &str, financial_impact: Option<&str>, confidence: &str` | `AppResult<String>` | — |
| `async current_impact_assessment` | `incident_id: &str` | `AppResult<Option<AnalyticalResultResponse>>` | — |
| `async create_ioc` | `incident_id: &str, evidence_id: &str, ioc_type: &str, value: &str, event_context: Option<&str>, confidence: &str, observed_at: Option<&str>` | `AppResult<String>` | — |
| `async list_iocs` | `incident_id: &str` | `AppResult<Vec<AnalyticalResultResponse>>` | — |
| `async create_recommendation` | `incident_id: &str, category: &str, title: &str, description: &str, confidence: Option<&str>, supporting_result_ids: &[String], supersedes: Option<&str>` | `AppResult<String>` | — |
| `async list_recommendations` | `incident_id: &str, category: Option<&str>` | `AppResult<Vec<AnalyticalResultResponse>>` | — |
| `async export_incident_findings` | `incident_id: &str` | `AppResult<ExportBundle>` | Exporta achados de um incidente (IOCs + recomendações). |
| `async create_finding` | `incident_id: &str, input: &FindingInput` | `AppResult<String>` | — |
| `async create_root_cause` | `incident_id: &str, input: &RootCauseInput` | `AppResult<String>` | — |
| `async findings_view` | `incident_id: &str` | `AppResult<FindingsView>` | Falha com ID/campo se um Finding vigente tiver severity inválida/`NONE` ou confidence inválida/ausente, em vez de omiti-lo da matriz (`EVD-031`). |

`list_attack_mappings`, `findings_view` e `deliverables::collect` exigem que ATTACK_MAPPING vigente tenha tática canônica, técnica/sub-técnica válida, base textual, confiança presente, relação base/confiança válida, `evidence_status` coerente e correlation key correspondente. Dezessete formas alcançáveis falham com ID/campo; DOCUMENTED e LOW/GAP continuam válidos, inclusive com sub-técnica ausente/nula (`EVD-048`).

`current_impact_assessment` exige que `availability_impact`, `confidentiality_impact` e `integrity_impact` sejam níveis canônicos (`NONE` incluso) e que a confiança vigente esteja presente e seja reconhecida. Nível desconhecido, ausente ou não textual, ou confiança inválida/ausente, falha com ID/campo antes da UI ou `deliverables::collect`; impacto financeiro ausente permanece válido (`EVD-046`).

`list_iocs` e `export_incident_findings` validam todo IOC vigente antes de retornar: `ioc_type` deve ser canônico, `normalized_value` deve ter forma válida e já estar normalizado, e a correlation key deve corresponder exatamente a incidente/tipo/valor. Nove formas inválidas falham com ID/campo; IOC legado sem `sighting_count` continua válido (`EVD-047`).

`list_recommendations` e `export_incident_findings` validam toda categoria vigente antes de retornar. Categoria desconhecida, ausente ou não textual falha com o ID da recomendação e o campo `category`, em vez de desaparecer do filtro ou ser exportada como estratégica (`EVD-039`).

`create_ioc` valida o agregado vigente antes de acrescentar uma ocorrência: `sightings` deve ser array, `sighting_count` presente deve ser inteiro não negativo e pelo menos igual ao tamanho do array, e o incremento não pode exceder `u64`. Qualquer falha impede a nova versão. O legado sem `sighting_count` continua válido e infere o total do array (`EVD-027`).

### `src/service/plan.rs` — Plano de fortalecimento

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `async create_plan_item` | `incident_id: &str, input: &PlanItemInput` | `AppResult<String>` | — |
| `async plan_view` | `incident_id: &str` | `AppResult<PlanView>` | — |

Ao revisar um item, a checagem de ciclos exige que `depends_on` presente nos itens vigentes seja um array de strings. Campo ausente continua representando a lista legada vazia; tipo errado ou elemento não-string falha antes de criar a revisão (`EVD-028`).

`plan_view` exige que `domain`, `priority`, `effort`, `cost_class`, `phase` e `support_kind` vigentes usem os vocabulários canônicos antes de ordenar ou calcular cobertura. Valor desconhecido, ausente ou não textual falha com o ID do item e o campo, em vez de receber defaults, desaparecer da cobertura ou chegar aos entregáveis (`EVD-040`/`EVD-041`).

`plan_view` aplica a mesma exigência a `finding_ids`, `nist_subcategories` e `depends_on` antes de ordenar os itens ou permitir `deliverables::collect`. Assim dados malformados não viram listas vazias nem uma falsa recomendação geral nos documentos; campos ausentes continuam válidos (`EVD-029`).

### `src/service/gaps.rs` — Lacunas de cobertura

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `async create_source_gap` | `incident_id: &str, input: &SourceGapInput` | `AppResult<String>` | — |
| `async list_source_gaps` | `incident_id: &str` | `AppResult<Vec<AnalyticalResultResponse>>` | Lacunas vigentes do incidente, da fonte esperada e do período mais antigos para os mais recentes (ordem estável: o id desempata). |

`list_source_gaps` valida todas as lacunas vigentes antes de ordenar ou permitir `deliverables::collect`: `expected_source` deve ser texto não vazio; `period_from`/`period_to` ausentes ou nulos continuam válidos, mas valores presentes devem ser data/RFC 3339 e o início não pode superar o fim; `reason` deve pertencer ao vocabulário canônico e `OTHER` exige nota textual não vazia. Payload inválido falha com ID/campo em vez de receber chave de ordenação vazia ou chegar aos relatórios (`EVD-045`).

### `src/service/nist.rs` — NIST CSF 2.0

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `async create_profile` | `incident_id: &str, profile_type: &str, version_label: &str, source: &str, tier: Option<&str>, tier_justification: Option<&str>, entries: &[ProfileEntryInput]` | `AppResult<String>` | — |
| `async profile_history` | `incident_id: &str, profile_type: nist_profiles::ProfileType` | `AppResult<ProfileHistoryView>` | Histórico completo (todas as versões, mais recente por último) de um profile CURRENT ou TARGET, entradas incluídas. |
| `async run_gap_analysis` | `incident_id: &str` | `AppResult<Vec<String>>` | Recalcula a Gap Analysis a partir dos profiles CURRENT/TARGET atuais (RF-010: reproduzível a partir dos dados persistidos, sem Internet) e persiste cada gap via F1-01. |
| `async current_gap_analysis` | `incident_id: &str` | `AppResult<Vec<AnalyticalResultResponse>>` | Gaps NIST atualmente vigentes (não superseded, e sem `status: "CLOSED"` no payload — ver `nist_profiles::run_gap_analysis`) de um incidente. |
| `async function_view` | `incident_id: &str` | `AppResult<NistFunctionView>` | Agrega os profiles Current e Target vigentes e os gaps vigentes por Function. |

`current_gap_analysis` exclui primeiro gaps `CLOSED` e valida os demais antes de retornar: `subcategory_id` deve ter forma/Function válidas e corresponder exatamente à correlation key, e `priority` deve ser HIGH, MEDIUM ou LOW. Valor desconhecido, ausente, não textual ou divergente da identidade falha com o ID da lacuna e o campo, impedindo que um gap listado desapareça da cobertura ou seja atribuído à Function errada (`EVD-042`/`EVD-044`).

`run_gap_analysis` rejeita payload inválido de um `NIST_GAP` vigente antes de abrir a transação. Além de JSON malformado (`EVD-024`), `subcategory_id` e `priority` de gaps abertos seguem a mesma validação da leitura, incluindo identidade com a correlation key; o erro identifica ID/campo e nenhuma sucessora é escrita (`EVD-043`/`EVD-044`). `CLOSED` sem prioridade continua recalculável e o legado aberto sem `status` permanece válido quando os campos obrigatórios são válidos.

### `src/service/timeline_filter.rs` — Filtro estruturado da timeline

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `parse` | `input: &str` | `Result<TimelineFilter, String>` | Interpreta o texto do filtro. |

### `src/service/demo.rs` — Incidente de demonstração

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `async seed` | `evtx: &Path, max_upload_bytes: u64, run_analysis: bool` | `AppResult<DemoSeedReport>` | Importa o `.evtx`, executa as análises reais (se pedido) e registra o cenário sintético num incidente novo. |

### `src/service/deliverables/mod.rs` — Documentos de entrega (`deliverables::…`)

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `render_all` | `data: &DeliverableData` | `AppResult<DeliverableSet>` | Renderiza os cinco documentos. |
| `render_with` | `data: &DeliverableData, options: RenderOptions` | `AppResult<DeliverableSet>` | Renderiza aplicando as opções: com a máscara ligada, os dados pessoais são trocados por marcadores antes de qualquer documento ser escrito. |
| `async collect` | `incident_id: &str` | `AppResult<DeliverableData>` | Lê do banco tudo que os documentos usam. |
| `async preview` | `incident_id: &str, options: RenderOptions` | `AppResult<DeliverableSet>` | Pré-visualização: coleta e renderiza sem gravar nada. |
| `write_set` | `set: &DeliverableSet, out_dir: &Path, paths: &StoragePaths, generated_at: &str` | `AppResult<GeneratedDeliverables>` | Grava os documentos e o manifest. |
| `async generate` | `incident_id: &str, out_dir: &Path, options: RenderOptions` | `AppResult<GeneratedDeliverables>` | Gera os cinco documentos e o manifest em `out_dir`, e registra o evento na trilha de auditoria (só nomes de arquivo e hashes, sem caminhos). |

### `src/service/deliverables/markdown.rs` — Documentos de entrega: renderização Markdown

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `esc` | `raw: &str` | `String` | Escapa texto de dados para uso em linha: colapsa espaços, quebras de linha e caracteres de controle (inclusive `ESC`) num único espaço e protege os caracteres que o Markd |
| `code` | `raw: &str` | `String` | Texto de dados como código em linha. |
| `short` | `id: &str` | `String` | Os 8 primeiros caracteres de um id, para citar a origem sem poluir o texto (o id completo vai para o manifest). |

### `src/service/deliverables/redaction.rs` — Documentos de entrega: mascaramento

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `redact` | `data: &DeliverableData` | `(DeliverableData, RedactionSummary)` | Cópia dos dados com os dados pessoais mascarados, e o resumo do que foi mascarado. |

### `src/persistence/backup.rs` — cópia verificada antes de migration (ADR-0026, RF-032)

| Função | Parâmetros relevantes | Retorno | Descrição |
|---|---|---|---|
| `async migrate_with_backup` | `pool: &SqlitePool, migrator: &Migrator, backups_dir: &Path, min_free_bytes: u64` | `Result<BackupOutcome, sqlx::Error>` | Cria a cópia consistente do banco, calcula o SHA-256, verifica a cópia e só então publica; depois aplica as migrations. |
| `async verify_published` | `dir: &Path` | `Result<BackupManifest, sqlx::Error>` | Reconfere uma pasta de backup já publicada (`dfir-app backup-verify <pasta>`). |

## Mapa das rotas HTTP removidas (histórico, ADR-0017)

O conteúdo histórico da API HTTP está em `docs/archive/API_HTTP_v1_superseded.md`. As funções equivalentes hoje, por área:

| Rota HTTP (removida) | Função equivalente hoje |
|---|---|
| `POST/GET/DELETE /api/evidence*` | `src/service/mod.rs`: `import_evidence_with_provenance`, `import_evidence_classified` (o nome `import_evidence` deixou de existir), `list_evidence`, `destroy_evidence` |
| `POST /api/analyze`, `GET /api/jobs/{id}*` | `src/service/mod.rs`: `analyze`, `timeline_page` (timeline paginada no SQLite, ADR-0021) |
| `POST/GET /api/incidents*` | `src/service/mod.rs`: `create_incident`, `list_incidents` |
| `POST/GET /api/incidents/{id}/nist/*` | `src/service/nist.rs` |
| `POST/GET /api/incidents/{id}/attack-mappings`, `/critical-path*`, `/impact-assessment` | `src/service/findings.rs` |
| `POST/GET /api/incidents/{id}/iocs`, `/recommendations`, `GET .../export` | `src/service/findings.rs` |
| (novo, F1-28) detecção por regras, candidatos, confirmar e rejeitar | `src/service/detection.rs`: `run_detection`, `list_candidates`, `confirm_candidate`, `reject_candidate`; CLI `dfir-app detect`, `candidates`, `candidate-confirm`, `candidate-reject` |
| (novo, RF-032) cópia do banco antes de migration | `src/persistence/backup.rs`: `migrate_with_backup`, `verify_published`; CLI `dfir-app backup-verify` |
