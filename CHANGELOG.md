<!-- doc-type: log | status: current -->
# Changelog

## Unreleased - 2026-11-18 - verificação ambiental final (`EVD-032`)

- Sem mudança de código sobre `7080dd5`: `just supply-chain` passou com as exceções já documentadas; `cargo deny --workspace check` terminou com advisories/bans/licenses/sources OK.
- Uma janela real isolada permaneceu aberta por 25 s até o timeout 124 esperado, com backend Gl/Mesa Intel. O log não teve diagnósticos de fonte/glyph/scroll nem nomes de fixtures. Não houve navegação manual ou incidente real.
- `just verify` e e2e não foram repetidos; permanecem em `EVD-031` com 457 passed/1 skipped e 110/110. Doc-gate PASS.

## Unreleased - 2026-11-18 - Finding inválido não desaparece da matriz de risco (`EVD-031`)

- `findings_view` descartava silenciosamente achados com severidade/confiança persistida inválida. A reprodução com `SEVERE` gerou o Entregável 2 com um achado, zero células de risco e linha CRITICAL zerada.
- A agregação agora exige severidade reconhecida e diferente de `NONE`, e confiança presente/reconhecida. O erro identifica Finding e campo; `collect` falha antes da renderização.
- `EVD-031`: três casos sem renderização, dois controles válidos, `just verify` com 457 passed/1 skipped, e2e 110/110 e doc-gate PASS. A fixture `CERTAIN` foi rejeitada pelo CHECK do SQLite no setup e não conta como evidência comportamental.

## Unreleased - 2026-11-18 - caminho crítico não aceita relações entre incidentes (`EVD-030`)

- `link_critical_path_stages` validava apenas o tipo dos estágios. A reprodução criou A0→B0 no contexto de A e B0→B1 no mesmo contexto; o grafo de A expôs uma aresta para um estágio estrangeiro.
- A persistência agora recebe o `incident_id` do serviço e exige que ambos os lados pertençam a ele antes de inserir `SEQUENCE_NEXT`. Ligação válida e ramificação continuam permitidas, e o erro de escopo mantém diagnóstico específico.
- `EVD-030`: regressão real de serviço/grafo, três testes focados, `just verify` com 456 passed/1 skipped, e2e 110/110 e doc-gate PASS. Não houve reparo de relações antigas; supply chain permanece em `EVD-020`.

## Unreleased - 2026-11-18 - arrays de plano inválidos não chegam aos entregáveis (`EVD-029`)

- `plan_view` aceitava `finding_ids`, `nist_subcategories` e `depends_on` malformados, e os documentos os convertiam em listas vazias. A reprodução gerou o Entregável 4 com a falsa frase **Recomendação geral** para um item com achado.
- Os três campos presentes agora precisam ser arrays de strings antes da ordenação/coleta. Campo ausente continua legado vazio; os três casos inválidos falham antes de renderizar, sem mudar assinaturas dos renderizadores.
- `EVD-029`: regressão real de collect/render, controles de legado e plano válido, `just verify` com 455 passed/1 skipped, e2e 110/110 e doc-gate PASS. Supply chain permanece em `EVD-020`; nenhuma janela real nova foi aberta.

## Unreleased - 2026-11-18 - dependência de plano inválida não permite revisão (`EVD-028`)

- `depends_on_of` convertia campo com tipo errado em lista vazia e descartava elementos não-string. Uma sucessora de B com `depends_on` string contendo A permitiu revisar A para depender de B, contornando a checagem de ciclo.
- Campo presente agora precisa ser array de strings e qualquer erro ocorre antes da revisão. Campo ausente continua sendo a lista legada vazia; os dois casos malformados não criam sucessora.
- `EVD-028`: regressão com dois casos, controles de campo legado e ciclo válido, `just verify` com 453 passed/1 skipped, e2e 110/110 e doc-gate PASS. Supply chain permanece em `EVD-020`; nenhuma janela real nova foi aberta.

## Unreleased - 2026-11-18 - agregado IOC inválido não gera nova versão (`EVD-027`)

- `sightings` não-array virava vetor vazio e `sighting_count` inválido virava zero/`max(len)`. A reprodução com contador 7 e `sightings = {}` criou uma sucessora com contador 8 e apenas uma ocorrência retida.
- A agregação agora valida array, inteiro e consistência da contagem, e usa `checked_add` antes de qualquer inserção. Apenas o legado sem `sighting_count` infere o total do array; os quatro casos inválidos falham sem nova linha.
- `EVD-027`: um teste com quatro casos, seis testes de agregação incluindo o legado, `just verify` com 451 passed/1 skipped, e2e 110/110 e doc-gate PASS. Supply chain permanece em `EVD-020`; nenhuma janela real nova foi aberta.

## Unreleased - 2026-11-18 - `RESULT` persistido exige array `items` (`EVD-026`)

- `ResultMessage` exige `items: Vec<Value>`, mas os leitores de timeline e resultados analíticos aceitavam campo ausente ou de outro tipo como zero itens. As reproduções retornavam `Ok([])` e `Ok((2, 0))` durante reindexação.
- Agora ambos falham com a linha para campo ausente/tipo errado. A timeline preserva o índice anterior via rollback, os erros analíticos de JSON/profundidade também indicam a linha, e `items: []` continua válido.
- `EVD-026`: dois testes novos com os controles, o teste anterior de rollback, `just verify` com 450 passed/1 skipped, e2e 110/110 e doc-gate PASS. Supply chain permanece em `EVD-020`; nenhuma janela real nova foi aberta.

## Unreleased - 2026-11-18 - JSONL inválido preserva o índice anterior da timeline (`EVD-025`)

- A reindexação apagava o índice e ignorava JSON inválido ou profundidade excessiva antes de commitar. A reprodução com dois eventos e uma segunda linha inválida retornava `Ok((2, 0))`.
- Agora esses casos retornam erro com a linha e acionam rollback. A regressão verifica que os dois eventos e `timeline_index = (2, 0)` sobrevivem tanto a JSON inválido quanto a 257 arrays aninhados; linhas válidas sem `items` mantêm o comportamento anterior.
- `EVD-025`: novo teste com dois casos, 29 testes existentes de timeline, `just verify` com 448 passed/1 skipped, e2e 110/110 e doc-gate PASS. Supply chain permanece em `EVD-020`; nenhuma janela real nova foi aberta.

## Unreleased - 2026-11-18 - gap NIST inválido não é fechado nem substituído (`EVD-024`)

- Com Current/Target já atendidos, um `NIST_GAP` com payload inválido era tratado como aberto e `run_gap_analysis` retornava um novo ID de fechamento. Agora a desserialização falha com `Protocol` antes de `BEGIN IMMEDIATE`.
- A regressão exige exatamente uma linha analítica e o mesmo registro inválido vigente, sem sucessora. Os fechamentos válidos por `target_met` e `no_longer_targeted` continuam passando.
- `EVD-024`: cinco testes de Gap Analysis, `just verify` com 447 passed/1 skipped, e2e 110/110 com as dez lacunas válidas, e doc-gate PASS. Supply chain permanece em `EVD-020`; nenhuma janela real nova foi aberta.

## Unreleased - 2026-11-18 - payload/provenance analíticos inválidos não viram null (`EVD-023`)

- As reproduções retornavam `payload: Null` por evidência e `provenance: Null` por incidente. `AnalyticalResultResponse` agora usa `TryFrom` com erros específicos, e as duas coleções propagam a falha para todos os consumidores.
- A evidência e o incidente foram criados pelos fluxos reais; as linhas inválidas foram inseridas sem updates ou remoção de triggers. Nenhum produtor ou schema mudou.
- `EVD-023`: dois testes focados, `just verify` com 446 passed/1 skipped, e2e 110/110 com todos os resultados e documentos válidos, e doc-gate PASS. Supply chain permanece em `EVD-020`; nenhuma janela real nova foi aberta.

## Unreleased - 2026-11-18 - metadados inválidos de custódia/auditoria não viram null (`EVD-022`)

- As reproduções inseriram registros inválidos sem remover triggers append-only. Antes da correção, `list_custody` e `list_events` retornavam `metadata: Null` com sucesso; agora propagam `Protocol` com o tipo do registro e sem expor o JSON bruto.
- `verify_custody` e `verify` continuam retornando `false` para esses registros. Nenhum hash, escrita ou comportamento de dados válidos mudou.
- `EVD-022`: dois testes focados, `just verify` com 444 passed/1 skipped, e2e 110/110 com cadeia íntegra antes e depois dos descartes, e doc-gate PASS. Supply chain permanece em `EVD-020`; nenhuma janela real nova foi aberta.

## Unreleased - 2026-11-18 - warnings de análise não desaparecem em JSON inválido (`EVD-021`)

- Antes da correção, `timeline_page_core` retornava `COMPLETED`, `warning_count: 1` e `warnings: []` tanto para JSON inválido quanto para um array vazio divergente. `JobStatusResponse` agora rejeita JSON inválido, contagem negativa e contagem divergente com diagnóstico específico.
- A serialização perdeu o fallback `[]` e passou para a transação que já faz rollback e marca o job como `FAILED`. Jobs legados com `warnings = NULL` e contagem zero continuam válidos.
- `EVD-021`: três testes focados, `just verify` com 442 passed/1 skipped, e2e 110/110 com os dois avisos do EVTX truncado persistidos, e doc-gate PASS. Supply chain permanece em `EVD-020`; nenhuma janela real nova foi aberta.

## Unreleased - 2026-11-18 - JSON inválido de candidatos falha fechado (`EVD-020`)

- Reprodução anterior à correção: `event_refs` ou IOCs persistidos com JSON inválido eram convertidos em listas vazias. A confirmação era aceita; no caso de IOCs, retornava `ioc_ids: []` após criar mapeamento e achado.
- Desserialização agora propaga erro específico, a listagem não esconde o dado inválido e a confirmação valida `event_refs` e IOCs antes de qualquer escrita analítica. Duas regressões exigem zero mapeamento/achado/IOC e candidato ainda pendente.
- `EVD-020`: `just verify` com 440 passed/1 skipped, e2e 110/110, supply chain e doc-gate verdes. Novo smoke real de 25 s no commit corrigido terminou em timeout 124 esperado, com adaptador Intel GL e sem avisos de fonte/glyph/ScrollArea ID no log isolado.

## Unreleased - 2026-11-18 - janela real isolada e gates finais (`EVD-019`)

- `timeout 25 ./target/debug/dfir-app` abriu a janela real no Xorg `:0` com `DFIR_DATA_ROOT` isolado no scratch e terminou em 124 como esperado. O log registrou o adaptador Mesa Intel HD Graphics 2000, backend GL, e os diagnósticos EGL conhecidos ficaram contextualizados.
- O log isolado não contém `No font`, `No glyph`, avisos de `ScrollArea ID`, os emojis removidos ou nomes de fixtures. Nenhum dado de caso ou default data root foi usado; a evidência não declara navegação manual por todas as telas.
- `EVD-019`: supply chain verde, e2e 110/110 e doc-gate PASS. A suíte Rust completa permanece em `EVD-018` com 438 passed/1 skipped; Windows, aarch64, incidente real e CI não foram promovidos.

## Unreleased - 2026-11-18 - scroll da grade NIST coberto por teste (`EVD-018`)

- Novo teste sem snapshots renderiza `function_section` a 600 px, exige `Gap baixo` inicialmente fora da viewport e usa um evento real `MouseWheel` para revelar a última coluna.
- Prova de sensibilidade: a mutação temporária para scroll vertical não moveu o conteúdo, que permaneceu em x=674,4..731,3, e falhou; a restauração horizontal passou. A mutação não foi commitada.
- `EVD-018`: `just verify` com 438 passed/1 skipped e doc-gate PASS. Nenhum código de produção mudou; supply chain/e2e permanecem em `EVD-014`, sem janela real ou verificação externa nova nesta etapa.

## Unreleased - 2026-11-18 - ações de evidência e scroll cobertos por teste (`EVD-017`)

- Novo teste sem snapshots verifica as quatro ações na faixa vertical de 48 px e sem sobreposição a 780 px. Após resize a 600 px, `Destruir` começa fora da viewport, dois eventos reais `MouseWheel` o revelam e o clique confirma `pending_destroy`.
- Prova de sensibilidade: a mutação temporária de `ScrollArea::horizontal` para vertical deixou `Destruir` em x=706,0..769,1 e falhou; a restauração passou. Ajustes de fixture durante a reprodução foram separados de defeitos de produção e estão documentados na evidência.
- `EVD-017`: `just verify` com 437 passed/1 skipped e doc-gate PASS. Nenhum código de produção mudou; supply chain/e2e permanecem em `EVD-014`, sem janela real ou verificação externa nova.

## Unreleased - 2026-11-18 - abas testadas na largura mínima real (`EVD-016`)

- `all_tabs_stay_inside_the_minimum_content_width` agora renderiza o conteúdo a 780 px, correspondentes à janela mínima de 960 px menos os 180 px da navegação. O teste anterior usava 960 px para o conteúdo e não cobria esse limite real.
- Prova de sensibilidade: a implementação com `horizontal_wrapped` passou; a mutação temporária para `horizontal` falhou com `Plano` até x=819,3; a restauração voltou a passar. A mutação não foi commitada.
- `EVD-016`: `just verify` com 436 passed/1 skipped e doc-gate PASS. Nenhum layout adicional mudou; supply chain/e2e permanecem em `EVD-014`, sem janela real ou verificação externa nova.

## Unreleased - 2026-11-18 - navegação incluída no teste estrito de glifos (`EVD-015`)

- O bloco real da navegação foi extraído sem mudança de comportamento para `DfirApp::navigation` e passou a ser renderizado por um teste com o default estrito do `egui_kittest`. Isso fecha a lacuna do teste anterior, que cobria `EvidenceState` mas não os rótulos laterais alterados no `26cc8b3`.
- Prova de sensibilidade: ao restaurar 📁 temporariamente, o teste falhou com `No glyph for 📁 (U+1F4C1)`; após restaurar o rótulo corrigido, voltou a passar. A mutação não foi commitada.
- `EVD-015`: `just verify` com 436 passed/1 skipped, doc-tests, fmt/check/clippy e doc-gate verdes. Supply chain/e2e não repetidos e mantidos em `EVD-014`; nenhuma janela real ou verificação externa nova.

## Unreleased - 2026-11-18 - ajuste visual da GUI verificado (`EVD-014`)

- No commit `26cc8b3`: removidos os emojis sem glifo das ações e da navegação, abas de incidentes passam a quebrar linha, profiles Current/Target usam IDs de scroll distintos e a tabela de evidências ganha scroll horizontal e linhas mais altas. Três testes de regressão cobrem glifos, abas estreitas e conflito de IDs.
- Verificação no Linux x86_64 com Rust 1.100.0: fmt/check/clippy/doc-tests verdes, nextest 435 passed/1 skipped, supply chain verde, doc-gate PASS e e2e 110/110. Detalhes em `docs/evidence/2026-11-18-evd-014.txt`.
- Nenhuma janela real foi aberta nesta rodada. Windows, Linux aarch64, incidente real e CI automático permanecem `PENDING-HUMAN`/`WAIVED`, sem promoção a `VERIFIED`.

## Unreleased — 2026-11-18 — verificação integrada final (`EVD-013`)

- No commit `f8dd8c6`: doctor 0 erros/1 aviso (Python 3.13.5 versus referência 3.14.6); `just verify` com nextest 432 passed/1 skipped, doc-tests, fmt/check/clippy; `just supply-chain` verde com exceções documentadas; `just doc-gate` PASS; `just e2e tests/fixtures/senai-demo.evtx` 110/110. Artefato: `docs/evidence/2026-11-18-evd-013.txt`.
- Windows, Linux aarch64, incidente real e CI automático não foram executados/implementados. Permanecem `PENDING-HUMAN`/`WAIVED`, sem promoção a `VERIFIED`.

## Unreleased — 2026-11-18 — plano de verificação externa e preparação do e2e multiplataforma

- `docs/FASE1_BACKLOG.md` passou a listar comandos e evidências de saída para Windows x86_64, Linux aarch64 e incidente real, todos ainda `PENDING-HUMAN` sob os waivers existentes. CI permanece sem implementação porque não há repositório remoto; `WVR-003` continua vigente.
- `just e2e` passou a usar `{{python}}`, escolhendo `python` no Windows e `python3` nas demais plataformas. Nenhuma execução Windows/aarch64/incidente real foi declarada.

## Unreleased — 2026-11-18 — skills locais mínimas para mudança e verificação

- Criadas somente `.jcode/skills/dfir-change` e `.jcode/skills/dfir-verify`, com autoridade, edição cirúrgica, gates e evidência. Nenhuma skill genérica foi instalada. `skill_manage list` reconheceu 2 skills e ambas foram carregadas com sucesso.

## Unreleased — 2026-11-18 — supressões amplas de dead code removidas

- Removidos os allows de módulo em `analytical_results.rs` e `nist_profiles.rs` e o allow da view NIST. Helpers usados só por testes ficaram em `cfg(test)`; campos nunca lidos saíram das views sem alterar banco, migrations ou consultas de ordenação. O histórico `supersedes` usado por asserção permanece disponível nos testes.
- Verificação: `just verify` com nextest 432 passed/1 skipped, doc-tests, fmt/check/clippy verdes; resta apenas o allow condicionado por plataforma no sandbox.

## Unreleased — 2026-11-18 — verify sem duplicação de testes

- `just verify` agora executa nextest e `just doctest`, sem repetir toda a suíte com `cargo test`. `just test` permanece como opção explícita do runner padrão.
- Verificação da nova receita: nextest 432 passed/1 skipped, doc-tests executados, fmt/check/clippy verdes; duração total 51,66 s neste ambiente.

## Unreleased — 2026-11-18 — canonicalização JCS consolidada sem mudança de hash

- `src/persistence/jcs.rs` passou a centralizar validação de inteiros e canonicalização usadas por auditoria, custódia e resultados analíticos. A justificativa histórica de duplicação por ausência de Cargo foi removida.
- Vetores capturados antes da extração fixam os hashes `BLAKE3-256-v1` e `BLAKE3-256-v2-JCS`; ambos permaneceram idênticos após a mudança. O gate estático agora exige o helper compartilhado e essa regressão.
- Verificação: persistência 109/109; vetor v1/v2 explícito; check de todos os targets; nextest 432 passed/1 skipped; clippy limpo.

## Unreleased — 2026-11-18 — propriedades críticas do protocolo e remoção de resíduo

- `proptest` passou de dependência não utilizada do app para dev-dependency de `dfir-protocol`. Novas propriedades cobrem profundidade JSON, round-trip de eventos e referências lógicas, com 128 casos cada. O lock manteve os mesmos 612 pacotes.
- Removido `engine/examples/dump_evtx_tmp.rs`, exemplo temporário sem referências que imprimia conteúdo de evidência.
- Verificação: protocolo 29/29; `just verify` com nextest 429 passed/1 skipped, fmt/check/clippy verdes; `just supply-chain` e `just doc-gate` PASS.

## Unreleased — 2026-11-18 — simplificação da governança documental

- `docs/rtm.md` passou a ser a única matriz de tickets; `docs/FASE1_STATUS.md` ficou somente com histórico. README e prompt perderam narrativas datadas; DoR e plano de backlog concluídos foram arquivados; atualizações passam a cobrir apenas documentos afetados e novos relatórios de rodada não são obrigatórios.
- A lista de variáveis internas foi completada, o comando inválido `git diff --delta` foi corrigido e `docs/TOOLCHAINS.md` passou a separar referência de versões realmente observadas, incluindo Python 3.13.5 versus referência 3.14.6.
- Novo `just doctor` (`scripts/doctor.py`) para diagnosticar ferramentas sem instalação; `release-tools` passou a fixar cargo-nextest 0.9.151, igual ao ambiente validado. O doctor terminou com 0 erros e 1 aviso esperado de Python; o pager `delta` ficou explicitamente ausente/opcional.

## Unreleased — 2026-11-18 — upgrade do pin para Rust 1.100.0 e lacunas RF-003/RF-027 fechadas (`EVD-012`)

- **Toolchain:** `rust-toolchain.toml` e `toolchains.toml` 1.99.1 → **1.100.0** (stable de 2026-11-12); `rust-version = "1.100"` nos três `Cargo.toml`. `cargo metadata --locked` ok; nenhum bump forçado pelo lock. `docs/TOOLCHAINS.md`, `docs/SUPPLY_CHAIN.md` e `docs/FASE1_BACKLOG.md` (pista B) atualizados.
- **Código:** 1 fix de lint novo do clippy 1.100 (`needless_bool` em `src/persistence/audit.rs` → `Ok(actual_count == verified_count)`).
- **Testes novos (lacunas do RTM fechadas):**
  - RF-003: `src/service/provenance.rs::import_leaves_the_source_file_byte_for_byte_untouched` — bytes, mtime e SHA-256 do arquivo de origem idênticos antes/depois da importação; cópia preservada com os mesmos bytes.
  - RF-027: `src/main.rs::no_network_listener_exists_in_app_or_engine_source` — guarda estática recursiva sobre `src/`, `engine/src/`, `protocol/src/` (nenhum `TcpListener`/`TcpStream`/`UdpSocket`/`UnixListener`/`UnixStream`/`tokio::net::`/`std::net::*`); complemento em runtime já existente em `engine/tests/sandbox.rs` (socket TCP/Unix = EPERM sob seccomp).
- **Supply chain:** `RUSTSEC-2026-0543` (chrono 0.4.45 "unmaintained", informativo, sem CVE) apareceu no deny após o bump; exceção justificada em `deny.toml` + `.cargo/audit.toml` (uso só de timestamps locais/parsing RFC 3339 interno; revisão 2026-12-31; avaliar jiff).
- **Gates executados em rustc/cargo 1.100.0, Linux x86_64 (`EVD-012`, artefatos em `docs/evidence/2026-11-18-*.txt`):** `fmt --check` ok; `check --workspace --locked --all-targets` ok; `clippy -D warnings` limpo; `nextest 426/426` (1 ignorado); `just e2e tests/fixtures/senai-demo.evtx` **110/110** com engine real e sandbox `required`; `cargo deny --workspace check` + `cargo audit` verdes; `validate_fase0.py` 0/0; `doc_gate.py` PASS.
- **Docs:** `EVD-012` em `docs/EVIDENCE_REGISTER.md`; linhas RF-003/RF-027 e "Atualização 2026-11-18" em `docs/rtm.md`; entrada no histórico de `docs/FASE1_STATUS.md`; nota no cabeçalho de `docs/RELEASE_GATE.md`.

## Unreleased — 2026-11-18 — micro-consolidações de helpers duplicados

- **`persistence/mod.rs`**: `validation_error` consolidada (7 definições idênticas removidas de `analytical_results.rs`, `attack_impact.rs`, `findings.rs`, `ioc_recommendations.rs`, `nist_profiles.rs`, `plan_items.rs`, `source_gaps.rs`). A função vive agora em `persistence::validation_error` com doc comentando o canal de erro (Protocol) que a camada de serviço já traduz.
- **`gui/mod.rs`**: helpers de UI movidos para `crate::gui::{non_empty, str_field, string_list, toggle, split_ids}` e reutilizados por `findings.rs`, `plan.rs`, `gaps.rs`, `attack.rs`, `critical_path.rs`, `impact.rs`, `iocs.rs`, `nist.rs`, `recommendations.rs` (≈133 linhas removidas, 0 risco: funções puras, testadas via UI).
- **`gui/mod.rs::test_harness::drain`**: helper de teste único substitui 11 cópias idênticas em `detection.rs`, `evidence.rs`, `deliverables.rs`, `findings.rs`, `gaps.rs`, `nist.rs`, `incidents.rs`, `plan.rs`, `timeline.rs` (≈77 linhas removidas).
- Gates: `fmt`, `check --workspace --locked --all-targets`, `clippy --workspace --locked --all-targets --all-features -- -D warnings`, `nextest run --workspace --locked` (426 pass, 1 skipped) — comportamento preservado.

## Unreleased — 2026-11-15 — pistas C e D do plano remanescente concluídas (F1-16, F1-19/20/21/23/24/33)

- **Pista C (F1-16) — `EVD-011`:** fixture EVTX sanitizada e determinística versionada em `tests/fixtures/senai-demo.evtx` (SHA-256 `256a655a…d448be`); `just e2e tests/fixtures/senai-demo.evtx` roda sem arquivo externo e passou **110/110**. `docs/OPERATION.md` §5 atualizado.
- **Pista D — `EVD-010`:** F1-19 vínculo evidência↔incidente idempotente com auditoria (`link-evidence`, `incident-evidence`, painel na GUI, vigentes sem janela); F1-20 limites YARA formalizados (`wide ascii` provado em UTF-16LE, timeout por tamanho, sem descarte silencioso); F1-21 ordem/retomada da destruição com testes de falha de banco/I-O e purga completa; F1-23 nenhum `let _ =` em produção (`rollback_loudly`, `discard_temp_*`, `mark_job_failed_loudly`, sends logados); F1-24 custódia/auditoria visíveis na GUI e CLI (`custody`, `audit`); F1-33/AUD-32 corrigido (YARA não sobrescreve a timeline). Cada ticket com teste próprio: nextest **424/424**, suíte `gui::` 62/62, clippy `-D warnings` limpo.
- **Docs:** `docs/rtm.md` e `docs/FASE1_STATUS.md` com os tickets em `VERIFIED` citando `EVD-010`/`EVD-011`; `docs/EVIDENCE_REGISTER.md` ganhou `EVD-010`/`EVD-011`; `docs/FASE1_BACKLOG.md` §7 marca pistas C e D concluídas; `docs/OPERATION.md` §3E (novos comandos CLI) e §5 (fixture versionada).

## Unreleased — 2026-11-15 — planejamento remanescente consolidado e ferramentas preferidas do agente

- **`docs/FASE1_BACKLOG.md` §7 (novo):** plano remanescente com pistas A–F e critério de saída por pista — A verificação externa (Windows/aarch64/incidente real, `WVR-001`/`WVR-002`), B CI automático (`WVR-003`/F1-14), C fixtures EVTX sanitizadas (F1-16), D lacunas parciais (F1-19/20/21/23/24/33), E supply chain recorrente (`WVR-004` até 2026-12-31), F fechamento do release candidate (F1-12).
- **`AGENTS.md` §15 (novo) e `CLAUDE.md`:** ferramentas preferidas do agente — `agentgrep` (grep/find/outline/trace) em vez de `grep -r`; `ast-grep` (`sg` depreciado); `read`/`edit`/`apply_patch` em vez de `cat`/`sed`; `rtk <cmd>` para saída condensada (`rtk proxy` como recuperação); `fd`/`eza`, `zoxide`, `bat -p`, `git diff` com delta; temporários em `$JCODE_SCRATCH_DIR`. As regras de segurança do §6 prevalecem. `jcodemunch` não está instalado no host e não foi recomendado.
- **`docs/FASE1_STATUS.md`:** histórico da rodada da noite (p2–p6) e desta rodada.

## Unreleased — 2026-11-14 (noite) — itens p2–p6 do plano do operador: waivers formais, DoR, verificação real da GUI e políticas operacionais

- **p2 — waivers formais** (`docs/EVIDENCE_REGISTER.md`, formato Apêndice F.14, aprovados pelo operador em 2026-11-14T22:00:30Z, válidos até 2026-12-31): `WVR-001` Runtime/Windows (Job Object só compilado; saída: `engine-smoke` + e2e em Windows), `WVR-002` Runtime/aarch64, `WVR-003` CI automático (F1-14; baseline manual `just verify` = `EVD-005`).
- **p3 — `docs/DEFINITION_OF_READY.md`:** todos os itens marcados `[x]` citando `EVD-001`..`EVD-009` / `WVR-001`..`WVR-004` (regra EVI-01 satisfeita; `just doc-gate` valida).
- **p4 — SEC-02 formalizado como `WVR-004`** (`RUSTSEC-2026-0459`, `libsqlite3-sys 0.37.0`): ponteiros cruzados em `deny.toml` e `.cargo/audit.toml`; revisão até 2026-12-31.
- **p5 — `EVD-009`:** janela real da GUI aberta no Xorg `:0` do host por 25 s; log registrou o adaptador wgpu (Mesa Intel HD Graphics 2000, backend Gl, 2 candidatos) — cobre GUI-01 e GFX-01. Detalhe observado: aviso "No font has 📁" (emoji fora das fontes padrão, sem `system_fonts`) — cosmético, não quebra.
- **p6 — políticas operacionais** em `docs/OPERATION.md`: §3.1.1 retenção de `backups/` (o app nunca apaga; manter as duas últimas; limpeza manual após `backup-verify`) e §5 distribuição de `.evtx` (pacote não distribui amostras; `security.evtx` fornecido pelo operador com SHA-256 na evidência; F1-16 de fixtures sanitizadas segue PLANNED).

## Unreleased — 2026-11-14 (tarde) — upgrade `eframe`/`egui` 0.37.0 + `cargo update` geral (item 1 do plano do operador)

- **`eframe`/`egui`/`egui_extras`/`egui_kittest` 0.36.2 → 0.37.0, `rfd` 0.17.2 → 0.18.0.** `eframe` com `default-features = false` (features `default_fonts`, `wgpu`, `x11`, `wayland`, `clipboard`, `links`): em 0.37 a feature `accesskit` do `egui-winit` é opcional e, desligada, a cadeia D-Bus/AT-SPI (`zbus`, `zvariant`, `zbus_names`, `zvariant_derive`, `atspi`, `accesskit_unix`, `ashpd`, `sctk-adwaita`, `ttf-parser`) **sai do grafo de build** (lock 615 → 612 pacotes). As exceções informativas RUSTSEC-2026-0370/0371/0372 e RUSTSEC-2026-0192 foram removidas do `deny.toml` porque os crates não existem mais no grafo.
- **`cargo update --workspace`:** 0 mudanças pendentes (já nas máximas compatíveis com rustc 1.99); minors aplicados `jiff 0.2.38→0.2.44`, `toml 1.1.8→1.2.0`, `uuid 1.29.0→1.29.2` (`defmt` fora do grafo). 52 pacotes restantes exigem `rust-version > 1.99`.
- **Testes de GUI** adaptados ao `egui_kittest 0.37`: harness com `allow_missing_glyphs()` (o padrão `MissingGlyphPolicy::Panic` quebra no glifo `▸`) e nome acessível obrigatório nos widgets (`labelled_by` nos `TextEdit`, `ComboBox::from_label`); novo `test_harness::builder` em `src/gui/mod.rs`. O binário não precisou de mudança de código.
- **Evidências:** `EVD-005` (`just verify` PASS, nextest 410/410, clippy `-D warnings` limpo), `EVD-006` (e2e 110/110 com `security.evtx`), `EVD-007` (audit 0 vulnerabilidades, `cargo deny --workspace check` ok). Não repetido: Windows, aarch64, janela real da GUI.

## Unreleased — 2026-11-14 — correções do registro de problemas de execução local (BUILD-01, GUI-01, SEC-01, SEC-02, TOOL-01, GFX-01, DOC-01)

**Pedido do operador:** ler o repositório, analisar o "Registro de problemas – execução local do v6.5.1" e corrigir, passo a passo.

- **SEC-01 — `evtx 0.12.3 → 0.15.1`** (`engine/Cargo.toml`, `default-features = false`): resolve `RUSTSEC-2026-0431` (alocação sem limite em `.wevtcache`) e, desde a 0.13, remove o crate sem manutenção `encoding` (`RUSTSEC-2021-0153`) do grafo. A API usada (`from_path`, `serialized_records`, `into_json_value`) não mudou. Resultado funcional idêntico no `security.evtx` de referência: **2 261 eventos**, 6 candidatos, sem avisos do parser.
- **SEC-02 — `RUSTSEC-2026-0459` (`libsqlite3-sys 0.37.0`)**: não há correção possível agora; `sqlx-sqlite 0.9.0` exige `<0.38.0` e `sqlx 0.9.0` é a última versão. Exceção **temporária** em `deny.toml` e `.cargo/audit.toml`, com análise de alcance (só o banco local no volume protegido, WAL, operador único) e revisão até 2026-12-31. Não é prova de inexploração.
- **TOOL-01 — `just supply-chain`** executa `cargo audit` e `cargo deny --workspace check` e ambos passam; `async-executor 1.14.0` (retirada) foi para 1.14.1. `just verify` ganhou o gate de validação estática atualizado.
- **GUI-01 — seletor de arquivos**: `src/gui/file_dialog.rs` roda o `rfd` em thread própria e drena o resultado por canal; a UI não congela e o botão nunca fica "sem fazer nada". Há campo de caminho digitado e arrastar-e-soltar, todos por `request_import` (caminho vazio, pasta e importação concorrente recusados com mensagem; vários arquivos soltos avisam quantos ficaram de fora). Dependências Linux (`xdg-desktop-portal`, `zenity`) documentadas em `docs/OPERATION.md` §3D. Novos testes cobrem os caminhos alternativos.
- **GFX-01 — adaptador de GPU**: a inicialização registra `adapter`, `backend`, `device_type`, `driver` e o número de candidatos; os erros de `wgpu_hal` continuam visíveis como diagnóstico, agora interpretáveis.
- **BUILD-01 — `just e2e`/`just demo`**: usam `cargo build -p dfir-app -p dfir-engine`; o e2e foi executado com engine e EVTX reais (**110/110**). O script não estoura mais `IndexError` quando o arquivo não gera candidatos, os filtros usam os Event IDs presentes e o "EVTX truncado" é realmente menor que o arquivo.
- **DOC-01 — toolchain e referências**: `rust-version = "1.99"` nos três manifests; `docs/TOOLCHAINS.md`, `README.md`, `PROMPT_NOVA_CONVERSA.md`, `docs/SUPPLY_CHAIN.md` e `LICENSES_THIRD_PARTY.md` atualizados (659 pacotes no lock, 656 de terceiros). Menções a 1.97.1 em ADRs/logs históricos foram preservadas.
- **Evidências (`docs/EVIDENCE_REGISTER.md`):** `EVD-001` (`just verify`, app 294 + engine 83 + 7 + protocolo 26), `EVD-002` (e2e 110/110), `EVD-003` (supply chain), em Linux x86_64 e rustc 1.99.1. **Não executado:** Windows, Linux aarch64 e incidente real.

## Unreleased — 2026-10-29 — dependências: `cargo update` completo, `cargo audit` e `cargo deny` reexecutados (F1-15; `docs/SUPPLY_CHAIN.md`)

- `cargo audit` no lock de 2026-10-27 achou **9 vulnerabilidades** (8 do `wasmtime 45.0.3`, 1 do `ctutils`), mais `tokio` *unsound* e `clipboard-win` *yanked*; o `cargo deny` falhava. Avisos posteriores às exceções antigas, não causados pela rodada de 2026-10-27.
- `cargo update` completo: `yara-x 1.22.1`, `wasmtime 48.0.8`, `tokio 1.54.1`, `ctutils 0.4.7`. `cargo audit`: 0 vulnerabilidades. `cargo deny --workspace check`: tudo ok. `deny.toml`: 4 exceções do wasmtime removidas, 3 exceções informativas de UI (`zvariant`/`zbus_names`/`zvariant_derive`) acrescentadas para sua revisão.
- Verificado com o lock novo, no pacote exato: `fmt`, `clippy -D warnings`, `test --workspace` (288 + 83 + 7 + 26), e2e 110/110, engine real sob Landlock + seccomp com o wasmtime 48. Não repetido: Windows, aarch64.

## Unreleased — 2026-10-27 — contenção do engine (F1-22), evento normalizado (F1-26), detecção por regras (F1-28), cópia antes de migration (RF-032); F1-27 adiado (ADR-0024, ADR-0025, ADR-0026)

**Pedido do operador:** concluir as etapas da análise de contenção do engine e finalizar os pedidos e restrições da demanda original, lendo toda a documentação.

- **Toolchain pinado pela primeira vez.** rustc/cargo **1.97.1** (do conda-forge, sem cópias locais de crates). O pacote 2026-10-23 já passava nele (app 240, engine 43, protocolo 11); a documentação dizia o contrário por nunca ter tentado.
- **F1-22 — contenção do engine (ADR-0024).** `env_clear` com ambiente mínimo (`reduced_environment`), **arquivo único imposto** (`DFIR_ENGINE_INPUT_FILE`, antes definida e nunca lida), `kill_on_drop`, e no Linux `no_new_privs` + **Landlock** + **seccomp** + `RLIMIT_CORE=0` aplicados pelo engine antes de qualquer thread e de ler a entrada. Política `DFIR_ENGINE_SANDBOX` (`required` no Linux; valor inválido recusa a partida), estado do sandbox no `PROGRESS`, na auditoria do job e como aviso `SANDBOX_NOT_FULLY_APPLIED`. **Windows:** Job Object (memória, 1 processo, sem interface); só compilado para `x86_64-pc-windows-gnu`, nunca executado. Prova: `engine --sandbox-selftest`, `engine/tests/sandbox.rs` (com controle) e e2e.
- **Defeito real:** o engine entrava em pânico sob seccomp (driver de sinais do tokio usa `socketpair`; só aparece na build do workspace). `socketpair` liberado deliberadamente.
- **F1-26 — evento normalizado.** Campos estruturais opcionais (`event_id`, provedor, canal, host, conta, IPs, processo, linha de comando, ação, resultado, tipo de fonte); esquema do protocolo 4; extrator compartilhado em `dfir_protocol::normalize` (o app recalcula os campos de resultados antigos); migration **0012**; filtro `campo:valor` na timeline e em `dfir-app results --filter`; data inválida é erro, validada na interface antes de enviar.
- **F1-28 — detecção por regras.** Análise `EVTX_DETECTION`, 19 regras versionadas (`2026.10.1`), catálogo em `docs/DETECTION_RULES.md`. Candidatos pendentes com ATT&CK sugerida, eventos de origem, IOCs e o limite do método; **confirmar** cria mapeamento, achado e IOCs (retomável), **rejeitar** exige motivo; decisão final protegida por gatilhos; auditoria; aba "Detecções" na tela de evidência; `dfir-app detect | candidates | candidate-confirm | candidate-reject`; Entregável 1 §4.3 mostra confirmados, rejeitados e **pendentes**.
- **RF-032 automático (ADR-0026).** Cópia `VACUUM INTO` verificada (SHA-256, `integrity_check`, `foreign_key_check`, contagens) antes de migrar um banco com dados; falha fechada; `dfir-app backup-verify`.
- **Defeito real (migration 0012):** `analysis_jobs.kind` tinha `CHECK` com dois tipos; a tabela foi recriada com teste de preservação.
- **Caracteres de direção de texto** (`text_safety`): viram marcador visível `⟦U+202E⟧` na timeline, nos candidatos e nos cinco documentos.
- **Adiado (decisão do operador):** F1-27, importador CSV/JSONL (ADR-0025 D-4). `RF-049` deixa de ser MVP.
- **Reconciliação:** especificação (nota 8, RF-033/036/049; as duas cópias idênticas), RTM, DEMAND_TRACE (linhas de identificação `COBERTO` com o limite dito; fontes de rede/SIEM/dispositivos `ADIADO`, não `PARCIAL`), FASE1_STATUS, FASE1_BACKLOG, README, CONVENTIONS, prompt de continuidade, tickets F1-10, 18, 22, 26, 27 e 28, OPERATION, DEMO, RELEASE_GATE e SUPPLY_CHAIN.
- **Gates:** `test --workspace` (app 288, engine 83 + 7 de integração, protocolo 26), `clippy --workspace --all-targets -D warnings` (inclui 3 avisos antigos de `assertions_on_constants`), `fmt --check`, e2e **110/110** com engine e `security.evtx` reais.

## Unreleased — 2026-10-23 — Etapa 7 do plano de conclusão do protótipo: incidente de demonstração, e2e dos cinco documentos e reconciliação (F1-11 e F1-12 reduzidos, ADR-0023)

**Pedido do operador:** ler toda a documentação e desenvolver e concluir a Etapa 7.

- **Defeito real corrigido (Etapa 6):** `dfir-app analyze` falhava em toda evidência com `Database(ColumnNotFound("source"))`: `analyze_inner` lia só 9 colunas de `EvidenceRow`. Uma lista única (`evidence_columns!`) agora serve às duas consultas; teste de regressão falha com a consulta antiga. Os 230 testes anteriores não pegavam (erro de execução); o e2e pegou.
- **Defeito real corrigido (Etapa 5/6):** depois de descartar a evidência, o Entregável 1 dizia "nenhuma evidência citada" mesmo com IOCs que a citam. Agora a evidência descartada segue listada (hash e custódia com `DESTROYED`), com a nota de descarte; "Escopo e limitações" e a linha do tempo bruta também tratam o caso. Teste de regressão.
- **`dfir-app demo <ARQUIVO.evtx>` (novo, `service/demo.rs`):** importa o EVTX, roda as análises reais (timeline e YARA-X) e registra o cenário BEC sintético (9 técnicas ATT&CK, caminho crítico de 6 etapas, 6 IOCs, 7 achados, 3 causas, 6 recomendações, 13 itens de plano nos 8 domínios, 3 lacunas de fonte, profiles NIST com 10 lacunas) pelas funções de serviço da interface. Tudo sintético é rotulado DEMO; o EVTX não é distribuído. `just demo`.
- **Classe do dado `DEMO` na importação:** `import_evidence_classified` (classes `FORENSIC`, `SANITIZED`, `DEMO`; a interface e a CLI de importação seguem `FORENSIC`). A demo grava `DEMO` e o evento de auditoria `DEMO_DATASET_SEEDED`.
- **e2e dos cinco documentos:** `scripts/e2e_headless.py` de 38 para 79 verificações (contagens, rótulo DEMO, hashes do manifest, tabelas, máscara, geração sem máscara, determinismo byte a byte, descarte, `audit-verify`).
- **Documentação:** `docs/DEMO.md` (roteiro), `docs/OPERATION.md` (instalação de referência, volume protegido, backup e atualização manual); README, backlog, RTM, DEMAND_TRACE, RELEASE_GATE, FASE1_STATUS, ADR-0023, fixtures e prompt de continuidade reconciliados com o código (o "Estado do MVP" do README dizia que Finding, plano, NIST por Function, proveniência e os cinco documentos "ainda não existiam").
- **Verificado:** rustc/cargo 1.91.1 (apt) com cópias locais descartáveis de `emath`/`epaint`/`egui` e, para o engine, de `wasmtime-environ`/`wasmtime`: `cargo test -p dfir-app` 240 ok (1 ignorado), `cargo clippy -p dfir-app --all-targets --all-features -- -D warnings` e `rustfmt --check` limpos; e2e 79/79 com engine e `security.evtx` reais. **Não verificado:** toolchain 1.97.1 pinado; GUI numa janela real; execução com a rede cortada; `cargo test` do engine e do workspace (o clippy 1.91 acusa 3 `assert!(true)` antigos nos testes de `protocol`).
- **Estado:** F1-11 `IMPLEMENTED / PENDING-HUMAN` (reduzido); F1-12 `PARCIAL`. **Pendente:** RF-032 automático (cópia do banco antes de migration), F1-26..F1-28, verificação no toolchain pinado (F1-14).

## Unreleased — 2026-10-18 — Etapa 6 do plano de conclusão do protótipo: proveniência, lacunas e máscara de dados pessoais (F1-31 e F1-32, ADR-0023)

**Pedido do operador:** "continue". Segue a Etapa 6 do plano da ADR-0023.

- **Migration `0011_provenance_and_source_gaps.sql`:** `analytical_results` aceita `SOURCE_GAP` (mesma receita da 0010) e `evidence` ganha `provided_by` e `contains_personal_data`.
- **Proveniência (F1-31):** a importação grava o que o operador declara (origem, quem forneceu, método, ferramenta e versão, data da coleta, autorização, dado pessoal) e `NOT_PROVIDED` ou ausente no resto. **Corrigido:** a importação gravava a data de importação como data de coleta e constantes (`LOCAL_OPERATOR`, `LOCAL_FILE_PICKER`) como origem e método; linhas antigas passam a ser tratadas como não informadas. A aba de evidências mostra "Tratamento" (analisada × somente preservada) e o resumo da proveniência.
- **Lacunas (F1-31):** aba "Lacunas" (`SOURCE_GAP`: fonte esperada, período, motivo, nota) e seção 2.1 do Entregável 1, junto com a proveniência não informada de cada evidência.
- **Máscara (F1-32):** IPv4, IPv6, e-mails e contas `DOMÍNIO\usuário` por padrão de texto, mais "fornecida por" e o responsável da custódia, com marcadores estáveis (`{IP-1}`, `{EMAIL-2}`...) sem gravar o mapeamento. **Ligada por padrão**; exportar sem ela exige confirmação na interface (`--no-redaction` na CLI) e a escolha vai para a auditoria e o manifest, só por contagem. Nomes em texto livre não são detectados.
- **CLI:** `dfir-app import` ganha `--source`, `--provided-by`, `--collection-method`, `--collection-tool`, `--collection-tool-version`, `--collected-at`, `--authorization` e `--personal-data`; `dfir-app deliverables` ganha `--no-redaction`.
- **Defeito real corrigido:** a lista de evidências do Entregável 1 seguia a ordem de entrada, e a saída variava com ela; `render_all` agora normaliza a ordem.
- **Removido:** o wrapper `import_evidence` (a GUI e a CLI usam `import_evidence_with_provenance`).
- **Verificado:** rustc 1.91.1 com cópias locais descartáveis de `emath`/`epaint`/`egui`: `cargo test` 230 ok (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos. **Não verificado:** toolchain 1.97.1 pinado; GUI numa janela real.
- **Estado:** F1-31 e F1-32 `IMPLEMENTED / PENDING-HUMAN`; F1-07 deixa de ser parcial; DEM-ENT-01, RB-001, RB-002 e RB-006 `COBERTO`. Próxima: Etapa 7 (incidente de demonstração e e2e).

## Unreleased — 2026-10-16 — Etapa 5 do plano de conclusão do protótipo: gerador dos entregáveis (F1-07, ADR-0023)

**Pedido do operador:** "continue". Segue a Etapa 5 do plano da ADR-0023.

- **`service/deliverables/` (novo):** cinco documentos Markdown determinísticos (relatório DFIR; vulnerabilidades e ATT&CK; maturidade NIST e recomendações; hardening; criptografia e proteção de dados) e `manifest.json` (SHA-256 e BLAKE3 por documento, ids dos resultados citados, versão da ferramenta, `generated_at`). Nada é inventado: seção sem dados diz "sem registro", cada item cita o resultado de origem, não há pontuação. O Entregável 1 declara que só EVTX é analisado automaticamente.
- **Seguro:** `markdown::esc` e `markdown::code` neutralizam Markdown, HTML, links e caracteres de controle; nomes de arquivo são constantes; a pasta de saída recusa o armazenamento interno, links simbólicos e caminho ocupado; escrita por arquivo temporário e renomeação; a auditoria (`DELIVERABLES_GENERATED`) guarda só nomes e hashes.
- **GUI:** aba "Entregáveis" com pré-visualização (mesma renderização, sem escrita) e "Gerar arquivos…" (seletor nativo de pasta). **CLI:** `dfir-app deliverables <INCIDENTE> [--out PASTA]`.
- **Fora desta etapa:** redação (F1-32) e lacunas de proveniência e retenção (F1-31), na Etapa 6; por isso F1-07 e DEM-ENT-01 ficam parciais.
- **Defeito achado ao ler os documentos gerados:** "Ativos afetados" e "Dados afetados" do Entregável 1 saíam vazios (o payload de impacto traz lista e o código lia só texto). Corrigido e coberto por teste; cabeçalhos `#` e `Severidade \\ Confiança` trocados por `Nº` e `Severidade × Confiança`.
- **Verificado:** rustc 1.91.1 com cópias locais descartáveis de `emath`/`epaint`/`egui`: `cargo test` 200 ok (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos. **Não verificado:** toolchain 1.97.1 pinado; GUI numa janela real.
- **Estado:** F1-07 `IMPLEMENTED / PENDING-HUMAN` (parcial); DEM-ENT-02..06 `COBERTO`, DEM-ENT-01 `PARCIAL`. Próxima: Etapa 6 (proveniência, lacunas de retenção e redação).

## Unreleased — 2026-10-15 — Etapa 4 do plano de conclusão do protótipo: visão NIST por Function (F1-30, ADR-0023)

**Pedido do operador:** "continue". Segue a Etapa 4 do plano da ADR-0023.

- **`persistence/nist_functions.rs` (novo):** agregação pura por Function (GV, ID, PR, DE, RS, RC; sempre as seis, na ordem oficial): Current por status, Target por subcategorias elegidas, gaps vigentes por prioridade. **Só contagens**, sem média, percentual nem nota. `gap_analysis_state` avisa quando os gaps vigentes foram calculados com versões anteriores dos profiles.
- **`service/nist.rs`:** `function_view` (profiles vigentes com Tier e justificativa, Functions e estado do gap).
- **GUI:** seção "Visão por Function" no topo da aba NIST, com nota de que não há score oficial; a visão é pedida ao abrir a aba e depois de criar um profile e de recalcular os gaps. `gui/backend.rs` e `gui/mod.rs` ligam o comando e o evento.
- **Desvio do texto do ticket F1-30:** o Tier é do profile inteiro e não existe por Function; aparece uma vez por profile, com a justificativa.
- **Refatoração só de testes:** `service/test_support.rs` (fixture comum) passa a ser usado pelos testes de achados, plano e NIST.
- **Verificado:** rustc 1.91.1 com cópias locais descartáveis de `emath`/`epaint`/`egui`: `cargo test` 180 ok (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos. **Não verificado:** toolchain 1.97.1 pinado; GUI numa janela real.
- **Estado:** F1-30 `IMPLEMENTED / PENDING-HUMAN`; DEM-NIST-01..06 `COBERTO`. Próxima: Etapa 5 (gerador dos entregáveis, F1-07).

## Unreleased — 2026-10-14 — Etapa 3 do plano de conclusão do protótipo: item de plano e aba "Plano" (F1-29, absorve F1-06, ADR-0023)

**Pedido do operador:** "continue". Segue a Etapa 3 do plano da ADR-0023.

- **`persistence/plan_items.rs` (novo):** `PLAN_ITEM` sobre o envelope analítico, sem tabela nova. Oito domínios (as sete ações da demanda; criptografia em repouso e em trânsito separadas); prioridade, esforço, classe de custo e fase obrigatórios (RB-004); referência regulatória/política interna em campo próprio (RB-005); item sem achado vira recomendação geral (`support_kind`); os domínios de criptografia e chaves exigem os cinco detalhes do Entregável 5 e os demais os recusam; "como verificar" obrigatório; dependências sem ciclo; `domain_coverage` lista os oito domínios, inclusive os vazios.
- **`service/plan.rs` (novo):** `create_plan_item` e `plan_view`.
- **GUI:** aba "Plano" (`gui/plan.rs`) com cobertura, itens por domínio e formulário; comandos e eventos em `gui/backend.rs`; listas de opções em `gui/mod.rs`, com um teste que as mantém iguais ao vocabulário do modelo.
- **Alterações fora dos arquivos novos:** `require_result` (`persistence/findings.rs`) e `non_blank` (`service/findings.rs`) ficaram visíveis ao crate para reuso.
- **Desvio do plano da ADR-0023 (D-1):** o plano previa exigir um achado ou uma recomendação de origem; o ticket F1-29 manda aceitar o item sem achado e marcá-lo como recomendação geral, e foi o ticket que prevaleceu.
- **Verificado:** rustc 1.91.1 com cópias locais descartáveis de `emath`/`epaint`/`egui`: `cargo test` 171 ok (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos. **Não verificado:** toolchain 1.97.1 pinado; GUI numa janela real.
- **Estado:** F1-29 e F1-06 (modelo) `IMPLEMENTED / PENDING-HUMAN`; DEM-PLAN-01..07, RB-004 e RB-005 `COBERTO`. Próxima: Etapa 4 (visão NIST por Function, F1-30).

## Unreleased — 2026-10-14 — Etapa 2 do plano de conclusão do protótipo: Finding, causa-raiz e aba "Achados" (F1-25, ADR-0023)

**Pedido do operador:** "teste usando a SDK de Rust disponível; se rodar bem, continue à próxima etapa, senão, foco em corrigir". A Etapa 1 rodou bem (127 testes) e a Etapa 2 foi feita em seguida.

- **`persistence/findings.rs` (novo):** `FINDING` e `ROOT_CAUSE` sobre o envelope analítico, sem tabela nova. Achado exige título, severidade acima de `NONE` e evidência; causa-raiz definitiva exige confiança HIGH ou CONFIRMED; toda conclusão cita um achado do mesmo incidente; lacunas explícitas (`gap_status`); matriz severidade × confiança sem pontuação.
- **`service/findings.rs`:** `create_finding`, `create_root_cause`, `findings_view`.
- **GUI:** aba "Achados" (`gui/findings.rs`) com matriz, listas e dois formulários; comandos e eventos em `gui/backend.rs`.
- **Alterações fora dos arquivos novos:** `require_evidence_for_high_confidence` passou a `pub(crate)` em `attack_impact.rs` (reuso da regra de F1-04); `#[allow(clippy::unnecessary_cast)]` em `infrastructure/disk.rs` (o clippy acusava dois `as u64` anteriores a esta etapa; os tipos de `statvfs` variam por plataforma).
- **Corrigido na verificação:** 3 `assert!` do teste de `persistence/db.rs` reformatados pelo `rustfmt`; dois testes de interface falharam na primeira execução por defeito do teste (janela baixa; spinner sob `run`).
- **Verificado:** rustc 1.91.1 com cópias locais descartáveis de `emath`/`epaint`/`egui`: `cargo test` 154 ok (1 ignorado), `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check` limpos. **Não verificado:** toolchain 1.97.1 pinado; GUI numa janela real.
- **Estado:** F1-25 `IMPLEMENTED / PENDING-HUMAN`; DEM-ID-02 e DEM-CR-01 `COBERTO`. Próxima: Etapa 3 (PlanItem).

## Unreleased — 2026-10-13 — Etapa 1 do plano de conclusão do protótipo: tipos `FINDING`, `ROOT_CAUSE`, `PLAN_ITEM` e migration 0010 (ADR-0023)

**Pedido do operador**: planejar como, quando e por onde produzir o que falta da demanda para finalizar o protótipo; depois, "comece pela Etapa 1".

- **Migration `0010_result_types_finding_plan_item.sql`:** recria `analytical_results` com o `CHECK` de `result_type` estendido (`FINDING`, `ROOT_CAUSE`, `PLAN_ITEM`). Dentro da transação do sqlx, com `PRAGMA defer_foreign_keys = ON`; preserva linhas, ordem de `sequence`, índices, triggers append-only e a tabela de relações.
- **`AnalyticalResultType`:** três variantes novas com `as_str`/`from_str` e requisitos (incidente obrigatório; confiança obrigatória para `FINDING` e `ROOT_CAUSE`, não para `PLAN_ITEM`). Nenhum comportamento existente muda.
- **Testes escritos:** `persistence/db.rs` (migração sobre banco populado e migrator embutido) e quatro em `persistence/analytical_results.rs`.
- **Escopo:** F1-26, F1-27 e F1-28 ficam fora do plano de conclusão do protótipo (continuam `PLANNED`). F1-06 passa a ser a visão por domínio de F1-29/F1-07, sem código próprio.
- **Verificado:** a 0010 sobre 0001–0009 com SQLite 3.45.1 (11 verificações, inclusive controle sem o pragma) e, depois, os testes Rust em rustc 1.91.1 (127 ok, 1 ignorado). Não verificado no 1.97.1 pinado.

## Unreleased — 2026-10-11 — testes de GUI com `egui_kittest` e observabilidade (ADR-0022)

**Pergunta do operador**: incluir `egui_kittest` e outras formas de teste? Logs/telemetria mais específicos para debug futuro?

- **Testes de GUI sem janela:** `egui_kittest` 0.36.2 (dev-dependency, +6 pacotes no lock). 6 testes em `gui/timeline.rs`, incluindo o que o Xvfb não conseguiu verificar: **digitar o filtro e clicar em "Filtrar"** envia um único pedido da página 0 com o filtro sem espaços. `Backend::for_test`, `TimelinePage::for_test`.
- **Logs em arquivo** (`infrastructure/logging.rs`): `<data_root>/logs/dfir.log`, JSON, rotação 5 × 5 MiB, 0600; `StoragePaths.logs`; configuração lida antes do `tracing`; panics no log.
- **Eventos com id e duração:** importação, análise (sucesso e falha), indexação da timeline, descarte, recuperação de órfãos, página de timeline (`debug`). Sem nome de arquivo nem conteúdo de eventos (verificado com o binário real).
- **`dfir-app diagnostics`:** JSON para suporte (versões, contagens, jobs por estado, 10 últimas falhas por código, disco, logs), sem nomes de arquivo nem mensagens de erro.
- **Não adicionado de propósito:** fuzzing (esperar o importador F1-27), testes de mutação, cobertura e snapshots de GUI (GPU/arquivos binários). Nenhuma telemetria remota (RB-006).
- **Verificado (rustc 1.91.1):** app 134 ok (+1 `#[ignore]`) incluindo 6 de GUI; engine 43, protocolo 11; `cargo deny check` ok.

## Unreleased — 2026-10-11 — riscos da ADR-0020 resolvidos (ADR-0021): timeline no banco, guarda de disco, avisos do parser, arquivo grande, supply chain

**Pedido explícito**: resolver todos os riscos antes de o operador testar a build.

- **Timeline no SQLite, paginada (F1-18).** Migration `0009`; `service::timeline_page` (offset, limite, filtro, total opcional) substitui `job_results`/`ResultsResponse`; índice construído em fluxo ao concluir o job (e na primeira consulta de jobs antigos); filtro no banco com `%`/`_` literais; descarte da evidência apaga o índice. GUI: novo `gui/timeline.rs` (cache de 48 páginas de 200 linhas, tabela virtualizada, linhas de uma linha, severidade em palavras) e a timeline do último job aparece ao selecionar a evidência. CLI: `results --offset --limit --filter`. Itens sem timestamp válido viram o aviso `TIMELINE_ITEMS_SKIPPED` em vez de sumir.
- **Guarda de espaço em disco (F1-10).** `DFIR_MIN_FREE_DISK_BYTES`; recusa importar/analisar e interrompe a análise (`DISK_SPACE_LOW`); Unix (`statvfs`), desconhecido (não bloqueia) nas demais plataformas.
- **Avisos do parser (F1-17, AUD-36/AUD-04).** Engine: `EVTX_TRUNCATED` e `EVTX_RECORDS_MISSING`; app: agrega todas as mensagens não fatais em `analysis_jobs.warning_count/warnings`; GUI e CLI os exibem.
- **Arquivo grande (F1-20).** `RLIMIT_AS` agora inclui o tamanho do arquivo de entrada; prazo da varredura YARA cresce com o tamanho.
- **Supply chain (F1-15).** `deny.toml`: licenças `BSD-2-Clause`, `ISC`, `BSL-1.0`; fontes do egui só para `epaint_default_fonts`; exceções justificadas para RUSTSEC-2026-0316/0327 (wasmtime 45, componentes desabilitados) e 0192 (`ttf-parser` não mantido); `yoke-derive` 0.8.3 (retirada) → 0.8.4 no `Cargo.lock`. `cargo deny --workspace check`: advisories, bans, licenses e sources ok.
- **Verificado por execução (rustc 1.91.1, não o pinado):** engine 43 ok, protocolo 11 ok, app com GUI 123 ok (+1 teste de desempenho `#[ignore]`); ponta a ponta com engine real e EVTX real **38/38**; GUI real renderizada sob Xvfb (software) e verificada em captura: lista, seleção, timeline carregada do banco, rolagem com páginas sob demanda, avisos do EVTX truncado.
- **Achados corrigidos pelo teste real nesta rodada:** deadlock do SQLite ao indexar dentro da transação de conclusão; `RLIMIT_AS` e prazo fixo de 60 s reprovavam arquivo de 1,1 GiB.

## Unreleased — 2026-10-04 — sem limite de saída nem truncamento (ADR-0020) e refinos validados por execução

**Pedido explícito**: remover o limite de saída e o truncamento; refinar o projeto enquanto o operador não pode testar no desktop Linux.

**Limites e truncamentos removidos** (ver ADR-0020 para o que foi mantido e por quê)
- `dfir-protocol`: `MAX_OUTPUT_BYTES`/`MAX_ITEMS` = 2^53 − 1 (sem limite prático; `i64::MAX` quebrava a canonicalização RFC 8785 da auditoria — teste de regressão); `MAX_LINE_BYTES` 64 KiB → 8 MiB (também o padrão de `DFIR_ENGINE_MAX_LINE_BYTES`).
- `service`: removidos `MAX_TIMELINE_EVENTS` (1000), `MAX_RESULT_LINES` (2000) e `MAX_PERSISTED_YARA_MATCHES` (1000); `LIMIT 100` do catálogo de evidências; `ResultsResponse.truncated` removido. A timeline é lida em fluxo, ordenada por UTC e devolvida inteira (100 mil eventos testados).
- `persistence`: `LIMIT 500` de `list_by_evidence`/`list_by_incident` removido (corrige a janela de AUD-08); nova `list_current_by_evidence`.
- `engine/yara`: sem `max_scan_size` (o YARA-X **truncava a varredura em silêncio** em 256 MiB; arquivo esparso de 300 MiB com marcador no fim agora é encontrado) e contagem exata de ocorrências por padrão, com amostragem explícita (`pattern_match_counts`, `matches_truncated`).
- GUI: removido o aviso "(truncado)"; filtro da timeline com cache de índices (a tabela já era virtualizada).

**Refinos**
- **AUD-07 corrigido:** reexecutar o scan YARA substitui o resultado vigente (`supersedes`) em vez de duplicá-lo.
- **AUD-03 (b–d) parcialmente corrigido:** `recover_orphaned_jobs` marca como `FAILED/ORPHANED` os jobs deixados `PROCESSING` por um app que fechou no meio da análise (auditado), ligado à partida da GUI e destrava o descarte da evidência.
- `scripts/e2e_headless.py`: agora também importa arquivos que realmente casam regras YARA (ASCII e UTF-16LE) e confere a idempotência.

**Verificado por execução (rustc 1.91.1, não o pinado):** engine 38 ok, protocolo 11 ok, app com GUI 112 ok; ponta a ponta com engine real e EVTX real **29/29** (2261 eventos, nenhum descartado; Mimikatz em ASCII e UTF-16 detectado, sem duplicar na reexecução). **Defeito novo, aberto:** AUD-36 — EVTX truncado termina `COMPLETED` com menos eventos e sem aviso (ticket F1-17).

## Unreleased — 2026-10-03 — CLI headless fina e primeiro teste ponta a ponta com engine real (ADR-0019)

**Pedido explícito**: guardar a demanda original; avaliar CLI/TUI "majoritariamente para testes mais certeiros", não como uso adicional completo.

- **Decisão (ADR-0019):** CLI **headless não interativa**, adaptador fino sobre `service::*`; **sem TUI**; nenhuma dependência nova. Subcomandos `import`, `evidence-list`, `analyze`, `results`, `findings`, `destroy` (JSON em stdout, logs em stderr; `destroy` exige `--confirm-filename`). Ticket F1-34.
- `scripts/e2e_headless.py` + `just e2e <arquivo.evtx>`: roda o binário real com o engine real num diretório temporário e confere 21 verificações (hash, formato, análise, timeline, YARA, auditoria/custódia, descarte).
- **Achados do primeiro teste real** (EVTX real de 2,1 MB, 2261 eventos):
  - **AUD-35 (novo, corrigido):** `import` abortava com `stack overflow` na thread principal em build debug. Causa: dois `[0_u8; 1 MiB]` dentro de funções `async` (`service/mod.rs`) viram parte do future e estouravam a pilha. Trocados por `vec![0_u8; 1 MiB]` (heap). A CLI também passou a rodar numa thread com pilha de 64 MiB e future na heap.
  - **AUD-05 confirmado (era `[P]`):** a saída do engine tem ~1,2 KB por evento; a análise desse arquivo já ocupa 2,7 MB do teto de 4 MiB, que comporta ~3,5 mil eventos. EVTX típicos de produção têm dezenas de milhares: passam a falhar com `OUTPUT_LIMIT`. Tratado em F1-18/F1-16 (sobe na prioridade).
  - **AUD-06 confirmado:** a timeline exibe 1000 de 2261 eventos (`truncated=true`); a verificação correspondente do script falha de propósito até o F1-18.
- `docs/source/demanda-12412-impressao-2026-04-23.txt`: cópia literal do texto da demanda original (SHA-256 em `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md`). O PDF segue **PENDING-HUMAN**.
- Logs JSON de `tracing` agora vão para **stderr** (antes stdout), para não misturar com a saída JSON dos comandos.

## Unreleased — 2026-10-02 — auditoria completa e realinhamento à demanda original (spec 6.5.0, ADR-0018)

**Pedido explícito**: auditar documentação e código contra a **demanda original** (SAGA SENAI 12412), corrigir a documentação e o projeto, cortar o desnecessário e acrescentar o necessário.

**Documentação e governança**
- **Demanda versionada no repositório**: `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md` (transcrição; o SHA-256 do PDF original é **PENDING-HUMAN**) e `docs/DEMAND_TRACE.md` (demanda → RF → ticket → estado). `docs/DOCUMENT_AUTHORITY.md` e `AGENTS.md` §1 apontam para eles.
- **Especificação v6.5.0** (`docs/PROJECT_SPEC_v6_5_0.md` + cópia raiz idêntica; v6.4.5 arquivada em `docs/archive/`): `.dfirpkg` (`RF-025`), atualização offline (`RF-026`), Ansible (`RF-012`) e benchmark (`RF-045`) passam para a Fase 4, alinhando a tabela de RFs ao roadmap §38 que já os colocava lá; `RF-041` (retry/DLQ) substituído por falha inspecionável; perfis executivo/técnico removidos de `RF-046`; `RF-016`/`RF-030`/`RF-044` perdem cláusulas de XSS/HTTP; RB-004 relida como orçamento **do cliente**; novos `RF-048`..`RF-055`. IDs anteriores não foram reutilizados (§39.1 lista as mudanças).
- **ADR-0018** (realinhamento) e adendo à **ADR-0017**: a troca para egui foi decisão do operador, não requisito da demanda (a demanda não menciona tecnologia de interface).
- **Backlog reescrito**: F1-13..F1-33 novos, F1-06/07/10/11/12 reescritos, F1-08/F1-09 adiados (Fase 4). RTM: linhas de baseline com evidência da era HTTP marcadas `STALE`; correção do `RF-011` (era citado como "descarte", é ATT&CK).
- Versões `0.33.3` → `0.36.2` nos documentos de estado atual; `App::update()` → `App::ui()` em `docs/NATIVE_UI_ARCHITECTURE.md`.
- Removidas 3 cópias byte-idênticas de especificações arquivadas (`nist-senai-*_v6_3_13`, `_v6_4_1` em `docs/archive/` e a cópia raiz da v6.4.5).

**Código** (todas as mudanças têm teste que reproduz o defeito; ver `docs/FASE1_STATUS.md` para o que foi de fato executado)
- `src/persistence/nist_profiles.rs`: **erro de compilação E0505** em `previously_open_gaps` (introduzido pela correção de fechamento de gap de 2026-09-26, que nunca foi compilada). Corrigido copiando o sufixo para `String` própria.
- `src/service/mod.rs`:
  - `destroy_evidence` agora **registra a destruição (banco + auditoria + custódia) antes de apagar o arquivo**; falha de I/O depois do registro devolve erro e a chamada seguinte conclui a limpeza (AUD-02). Núcleo extraído em `destroy_evidence_core` para ser testável.
  - `analyze` marca o job como `FAILED` quando `begin_immediate` falha nos dois caminhos (sucesso e erro do engine), em vez de deixá-lo `PROCESSING` (AUD-03a).
  - Timeline **ordena por UTC antes de truncar** (`build_timeline`), em vez de cortar os 1000 primeiros registros na ordem do arquivo (AUD-06).
  - Falha ao registrar `EVIDENCE_INTEGRITY_FAILURE` não é mais engolida por `let _ =`: o erro devolvido ao operador diz explicitamente que o registro de auditoria falhou (AUD-25).
  - `AnalyzeResponse` ganhou `analysis_kind`.
- `src/gui/evidence.rs`: executar YARA não sobrescreve mais o painel de timeline da evidência (AUD-32).
- `src/gui/mod.rs`, `src/gui/incidents.rs`: portados para a API do egui 0.36 (`SidePanel` → `Panel`, `show_inside` → `show`); a árvore nunca tinha sido compilada com 0.36.2.
- `engine/rules/dfir_mvp_ruleset.yar` + `engine/src/yara_scan.rs`: `wide ascii` nos padrões de PowerShell, Mimikatz e nota de resgate (EVTX é UTF-16LE), `rule_version = "2"`, `RULESET_VERSION` 1.1.0 e 4 testes novos (AUD-09).
- `Cargo.lock` regenerado: alterou apenas `eframe`/`egui`/`egui_extras` (0.33.3 → 0.36.2) e a árvore de UI correspondente (AUD-01).
- `rustfmt` aplicado a `nist_profiles.rs` e `service/nist.rs`.

**Gates**: `scripts/validate_fase0.py` ganhou coerência manifesto × lock e a verificação de que todo ticket citado em `DEMAND_TRACE.md`/ADR-0018 existe.

**Não alterado de propósito (viram ticket)**: `env_clear`/isolamento do engine (F1-22), política de limite de tamanho da YARA (F1-20, parte pendente), YARA idempotente e vigentes sem janela (F1-19), erros de parsing persistidos (F1-17), recovery de jobs na partida (F1-10): dependem de integração com o engine e de fixtures EVTX que o projeto ainda não tem.

## Unreleased — 2026-09-26 — eframe/egui 0.36.2 e correção de bug de ordem no reject_symlink

**Pedido explícito**: ler a documentação do projeto, atualizar eframe/egui/egui_extras para a versão mais recente, e revisar o código com atenção a bugs.

- **`eframe`/`egui`/`egui_extras`: 0.33.3 → 0.36.2** (`rfd` já estava em 0.17.2, a versão mais recente; sem mudança). Breaking change do eframe 0.34.0 ("Replace App::update with fn logic and fn ui"): `fn update(&mut self, ctx: &Context, frame)` deixou de ser o método obrigatório de `eframe::App` — virou provido/depreciado — e `fn ui(&mut self, ui: &mut Ui, frame)` passou a ser o obrigatório. `src/gui/mod.rs` só implementava `update`; sem essa migração o crate não compilaria contra 0.36. Migrado para `fn ui`, painéis (`SidePanel`/`CentralPanel`) trocados de `.show(ctx, …)` para `.show_inside(ui, …)` — mesmo padrão já usado em `gui/incidents.rs` — e `request_repaint_after` movido para `ui.ctx()`. Nenhuma outra chamada de API do egui usada em `src/gui/*.rs` (TableBuilder, ComboBox::from_id_salt, ScrollArea, RichText, DatePickerButton etc.) tem mudança de assinatura entre 0.33 e 0.36.
- **`Cargo.lock` não regenerado nesta sessão**: sem toolchain Rust utilizável neste ambiente (apt só oferece rustc/cargo 1.75, anterior à edition 2024 do workspace e ao MSRV 1.95 do egui 0.36). Rodar `cargo update -p egui -p eframe -p egui_extras` (ou um `cargo build` normal) localmente para regenerar as entradas transitivas afetadas (winit, wgpu, glow etc.).
- **Bug de segurança corrigido (2 ocorrências do mesmo padrão)**: `reject_symlink` estava sendo chamado sobre o caminho **já canonicalizado** em `src/application/process.rs::launch_process` e em `engine/src/input.rs::resolve_input`. Como `std::fs::canonicalize` já resolve symlinks, o caminho resultante nunca pode, por definição, "parecer" um symlink para `symlink_metadata` — a checagem era um no-op permanente exatamente contra o caso que a própria documentação da função descreve ("a pre-planted symlink sitting at the path when we look"). Um symlink cujo alvo permanecesse dentro do root confiável passava incólume pelas duas checagens (containment via `starts_with` após canonicalize, e o `reject_symlink` inerte). Corrigido nos dois locais checando o caminho **antes** de canonicalizar, mantendo a checagem de containment sobre os caminhos canonicalizados como já era. Nenhum teste existente cobria esse caso — o teste `..._escapes_the_root` só exercita a checagem de containment, nunca depende do `reject_symlink`; adicionado `resolve_input_rejects_a_symlink_whose_target_stays_inside_the_root` em `engine/src/input.rs` para cobrir a lacuna. Não foi adicionado um teste equivalente para `launch_process` nesta sessão porque esse caminho depende de um subprocesso real (`dfir-engine`); a função `reject_symlink` em si já era testada diretamente e continua sendo.
- **Bug de gap analysis corrigido** (achado nesta sessão e corrigido a pedido explícito em seguida): `persistence/nist_profiles.rs::run_gap_analysis` nunca fechava um gap que deixasse de existir entre duas rodadas — se o current profile evoluísse a ponto de atender o target (ou o target deixasse de exigir a subcategoria), nada superava o `NIST_GAP` antigo, e `current_for_correlation_key`/`service::nist::current_gap_analysis` continuavam devolvendo-o como vigente para sempre. Corrigido: a cada rodada, `run_gap_analysis` agora também descobre (via `analytical_results::distinct_correlation_keys`, nova, sem o corte de `MAX_LISTED_RESULTS` de `list_by_incident` — necessário para não perder um gap parado há muitas rodadas) todo `NIST_GAP` vigente cuja subcategoria não apareça mais em `gaps` nesta rodada, e o fecha com um registro que o supersede e traz `"status": "CLOSED"` e `"closed_reason"` (`target_met` ou `no_longer_targeted`) no payload — sem introduzir um novo `AnalyticalResultType` só para isto. `service::nist::current_gap_analysis` passa a filtrar esse `status` antes de devolver os gaps "vigentes" à GUI (`gui/nist.rs` não mudou — continua só listando o que o serviço devolve). Reaberto normalmente se a subcategoria voltar a ser gap depois (o encadeamento de `supersedes` já cobria isso sem mudança extra). Dois testes novos em `persistence/nist_profiles.rs` cobrem os dois motivos de fechamento e confirmam que recalcular sem mudança não fecha o mesmo gap de novo.

**Evidência de execução**: nenhuma. Ambiente sem `cargo`/`rustc` utilizável (ver acima) — revisão inteiramente por leitura manual do código (todo `dfir-app/src`, `engine/src`, `protocol/src`) contra a documentação das mudanças de API do eframe/egui 0.34–0.36. Sem `cargo check`/`test`/`clippy`/`fmt` rodados nesta sessão. Necessário rodar os gates de sempre localmente antes de considerar isto fechado.

**Não verificado / pendente**: `Cargo.lock` desatualizado (ver acima); GUI nunca executada visualmente nesta sessão; nenhum dos testes novos (`engine/src/input.rs`, `persistence/nist_profiles.rs`) foi de fato rodado, só escrito e revisado manualmente linha a linha contra os tipos/assinaturas reais do código.

## Unreleased — 2026-09-25 — simplificação e indexação da documentação para LLM

**Pedido explícito**: revisar a documentação completa do projeto, simplificar mantendo só o necessário, e (se fizer sentido) estender a tag de indexação para LLM já usada no `PROJECT_SPEC`.

- **Estrutural, sem mudança de decisão/requisito**: removidos cabeçalhos duplicados/órfãos (`CHANGELOG.md` tinha dois "# Changelog" órfãos no meio do arquivo e uma seção `[Unreleased]` vazia; `AGENTS.md`/`PROMPT_NOVA_CONVERSA.md` tinham um "### Desenvolvimento local" sem conteúdo no fim do arquivo).
- **`docs/PROJECT_SPEC_v6_4_5.md`** (e cópia raiz, mantida byte-a-byte igual): 7 subseções sem conteúdo (`6.3.2`, 3 itens de `41.2`, `R-004`, `R-008`, `F.8.1`) receberam uma nota explícita — "sem conteúdo, tópico obsoleto" para as que já não se aplicam (ex.: middleware HTTP, YARA-Forge agregado), ou "PENDING" com ponteiro para onde a informação equivalente já existe, para as que ainda são relevantes. Nenhum texto normativo existente foi alterado ou apagado.
- **Documentos arquivados** (movidos para `docs/archive/`, conteúdo preservado): `REVIEW_2026-08-17.md`, `IMPLEMENTATION_REVIEW_2026-08-19.md`, `FINAL_REVIEW_2026-08-18/19/20/20-deep.md` — revisões pontuais já incorporadas ao estado corrente. Aviso "HISTÓRICO/SUPERSEDED" padronizado nas 3 que não tinham.
- **Staleness corrigida**: `docs/FASE0_STATUS.md` ainda listava a UI web local-server-rendered e o frontend TypeScript como `IMPLEMENTED` — corrigido para `RETIRADO` (ADR-0017), com linha nova para a interface nativa. `docs/RELEASE_GATE.md` ainda dizia "exposta somente em loopback" — corrigido para "não expõe porta de rede alguma". `ADR-0005` e `ADR-0009` já eram citados como superseded por ADR-0017 em `rtm.md`/no próprio ADR-0017, mas não traziam esse aviso nos seus próprios arquivos — corrigido, no mesmo formato de banner já usado em ADR-0001/0003/0004.
- **Tag de indexação para LLM**: adicionada a linha `<!-- doc-type: ... | status: ... -->` no topo de todo documento fora da especificação canônica (README, AGENTS, CONVENTIONS, CONTRIBUTING, status/gates/backlog, as 17 ADRs, docs arquivados nesta sessão). Convenção registrada em `docs/DOCUMENT_AUTHORITY.md` (seção "Staleness", que estava vazia). Não é uma segunda fonte de verdade — mesmo princípio dos blocos XML já usados no `PROJECT_SPEC`.
- **Não alterado de propósito**: o corpo normativo do `PROJECT_SPEC` (decisões, requisitos, notas de superseded existentes) — o documento já preserva histórico via anotação de supersessão em vez de reescrita, e esta sessão seguiu o mesmo princípio. Nenhum bump de versão do spec (segue v6.4.5); é o autor do projeto quem decide se as correções acima justificam v6.4.6.

## Unreleased — 2026-09-22 — interface nativa eframe/egui substitui Axum + frontend TypeScript (ADR-0017)

**Requisito explícito da demanda**, não uma sugestão interna: substituir a interface HTTP/navegador por uma interface nativa em Rust com eframe/egui/egui_extras. Decisão completa, incluindo por que isso supera o contrato de frontend do Apêndice F.3.1 do spec: `docs/adrs/ADR-0017-interface-nativa-eframe-egui.md`.

- **Removido por inteiro**: `src/web/` (handlers Axum), `templates/` (HTML + template Askama de CLI), `frontend/` (TypeScript/Node/npm). Dependências removidas: `axum`, `tower`, `tower-http`, `askama`. `AppConfig::bind`/`DFIR_BIND`/`DFIR_BIND_ALLOW_NON_LOOPBACK` removidos — não há mais porta nenhuma.
- **Novo `src/service/`** (`mod.rs`, `nist.rs`, `findings.rs`): a lógica de negócio dos handlers Axum portada linha a linha (mesmas transações `BEGIN IMMEDIATE`, mesma auditoria/custódia, mesma validação), agora `async` puro sem HTTP. Única mudança de comportamento real: importação de evidência lê um arquivo local escolhido via diálogo nativo (`rfd`) em vez de multipart de upload de navegador.
- **Novo `src/gui/`** (eframe 0.33.3 + egui 0.33.3 + egui_extras 0.33.3 + rfd 0.17.2): telas Evidência (lista, importar, destruir, analisar EVTX/YARA, timeline filtrável) e Incidentes (NIST CSF, ATT&CK, caminho crítico, impacto, IOCs, recomendações, exportação para arquivo JSON local) — paridade funcional com as telas de `frontend/src/app.ts`/`incident-ui.ts`/`timeline.ts`, removidas. Arquitetura: `src/gui/backend.rs` roda um runtime Tokio numa thread dedicada, ligado à thread da UI (síncrona, um `update()` por quadro) por canais `Command`/`Event` — nenhuma chamada assíncrona acontece na thread da UI. Detalhe: `docs/NATIVE_UI_ARCHITECTURE.md`.
- `src/main.rs`: sem subcomando, abre a GUI (antes: servidor Axum). `audit-verify`/`engine-smoke` continuam subcomandos de CLI.
- `docs/API.md` arquivado em `docs/archive/API_HTTP_v1_superseded.md` (não há mais rota alguma para documentar como contrato ativo).
- `scripts/validate_fase0.py` e `scripts/doc_gate.py` atualizados: paravam de fazer sentido sem `frontend/`/Askama. `doc_gate.py` já falhava antes desta sessão (checava o pin `1.3.14` do Bun, removido pela ADR-0009, e nunca corrigido — ver a entrada de 2026-09-21 abaixo); como a checagem de frontend inteira sai nesta mudança, o bug some junto.

**Evidência de execução** (ambiente sem o toolchain 1.97.1 pinado; `rustc`/`cargo` 1.91.1 via `apt`, usados com `--ignore-rust-version`): `cargo check -p dfir-app` limpo (0 erros, 0 avisos); `cargo test -p dfir-app --bin dfir-app` **93/93 verdes**; `cargo fmt --all -- --check` limpo; `cargo clippy -p dfir-app --all-targets --all-features -- -D warnings` — ver `docs/FASE1_STATUS.md` para o resultado. `cargo check --workspace` **não** passa neste ambiente, mas por um motivo alheio a esta mudança: `dfir-engine`→`yara-x 1.20.0`→`wasmtime-environ 45.0.3` usa a feature de biblioteca instável `debug_closure_helpers`, indisponível no rustc 1.91 — confirmado reproduzindo o mesmo erro no `dfir-engine` original, sem nenhuma alteração desta sessão. `cargo nextest`/`cargo audit`/`cargo deny` não executados (ferramentas não instaladas neste ambiente).

**Não verificado / pendente**: nenhum gate rodou no toolchain 1.97.1 real; a GUI nunca foi executada visualmente (sem display neste ambiente — só `cargo check`/`test`/`clippy`, que não renderizam janela). Ver `docs/FASE1_STATUS.md`.

## Unreleased — 2026-09-21 — supply chain do yara-x, cap de sightings, engine_path, engine em módulos, UI de incidentes e API.md

**Primeira execução real dos gates Rust** (sandbox Linux com rustc 1.97; `cargo nextest` não rodou — foi usado `cargo test`):

- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test` (engine 30, protocol 10, app 103, frontend 28) e `cargo deny --workspace check` **verdes**. O `Cargo.lock` foi regenerado com o `cargo` (186 pacotes do `yara-x` adicionados, nenhum pin existente alterado; 492 → 407 pacotes depois do corte de features). Já a formatação: 7 arquivos vinham fora do `rustfmt` (nunca formatados sem `cargo`); o `cargo fmt` os normalizou — mudança só de formatação.
- Erros latentes que só um build real revelaria, corrigidos: tipo do bloco `async` em `src/web/mod.rs` (E0282); import `CommandExt` sem uso em `src/application/process.rs`; fixture ATT&CK com `sub_technique_id: "001"` (o formato exigido é `T1059.001`); dois testes de `src/config.rs` comparavam caminho do Windows com o separador do host e só passavam no Windows.
- Verificação de ponta a ponta com os binários reais: `dfir-app` sem `DFIR_ENGINE_PATH` executou um `YARA_SCAN` real pelo `dfir-engine` vizinho (EICAR detectado); todas as famílias de rotas de incidente foram exercitadas com os corpos de `docs/API.md`; o bundle da UI foi carregado em `jsdom` contra o servidor real (fluxos de todas as abas).

**1. Supply chain do `yara-x` (ADR-0010, adendo; `docs/SUPPLY_CHAIN.md`)**
- Descoberta: `cargo deny check` sem `--workspace` só examinava o pacote raiz `dfir-app` — o grafo do `dfir-engine` (`evtx`, `yara-x`, `wasmtime`) nunca foi verificado. `just licenses` agora usa `--workspace`.
- `engine/Cargo.toml`: `yara-x` com `default-features = false` e só `constant-folding`, `exact-atoms`, `fast-regexp` (o ruleset não usa `import`). Some a pilha de criptografia, inclusive o advisory RUSTSEC-2023-0071 (`rsa`, sem correção).
- `deny.toml`: `Apache-2.0 WITH LLVM-exception` na allow-list (24 crates do wasmtime/cranelift; licença permissiva) e três exceções de advisory justificadas — `wasmtime 45.x` (RUSTSEC-2026-0222/-0269; o `yara-x 1.20.0` fixa `^45.0.3` e a correção só existe em >=46.0.2) e `bincode` (RUSTSEC-2025-0141, sem manutenção). Reavaliar quando o `yara-x` subir para `wasmtime >= 46.0.3`.

**2. Teto de `sightings[]` do IOC (F1-05; ADR-0013, adendo)**
- `src/persistence/ioc_recommendations.rs`: o payload guarda as 100 ocorrências mais recentes (`MAX_STORED_SIGHTINGS`); `sighting_count` passa a ser o total real e `sightings_dropped` informa o corte; `first_seen`/`last_seen` são preservados do payload anterior. As ocorrências descartadas continuam nas versões anteriores (append-only). Payloads antigos sem os novos campos são aceitos.

**3. `engine_path` a partir de `current_exe()` (ADR-0015)**
- `src/config.rs`: `DFIR_ENGINE_PATH` → `dfir-engine[.exe]` ao lado do `dfir-app` → `/usr/local/bin/dfir-engine` (Unix). Função pura `select_engine_path` testável por plataforma. O caminho resolvido é logado na inicialização (`src/main.rs`). `scripts/setup.ps1` deixa de copiar o engine para `storage\runtime\`; README atualizado.
- **Mudança de comportamento**: o local antigo do Windows (`storage\runtime\dfir-engine.exe`) deixa de ser procurado por padrão.

**Engine em módulos e `NormalizedEvent` (ADR-0016)**
- `dfir-protocol`: novo `NormalizedEvent` (`into_item`/`from_item`), formato no fio inalterado. `timeline_event_from_item` (app) usa o tipo em vez de duck-typing — e por isso passa a **ignorar** itens sem `evidence_id`/`raw_reference`/`data` em vez de inventar `"unknown-evidence"`.
- `engine/src/main.rs` dividido em `runner` (`EngineRunner`), `input`, `limits`, `output`, `evtx_timeline` e `error`, sem mudança de comportamento. **Não** foi criada trait `Parser`/`EvtxParserAdapter` (só existe um parser iterativo; retomar junto com o PCAP).

**4. Interface de incidentes (`frontend/`; `docs/FRONTEND_ARCHITECTURE.md`)**
- Nova seção Incidentes (criar/listar/abrir) e workspace com abas: NIST CSF, ATT&CK, caminho crítico, impacto, IOCs, recomendações e exportação. Módulos `dom.ts`, `incident-model.ts` (puro; 20 testes novos) e `incident-ui.ts`. `frontend/dist/` regenerado. Checklist manual em `tests/frontend/fase0.md`.
- `src/error.rs`: erros de validação da camada de persistência (`sqlx::Error::Protocol`) passam a responder **`400` com o motivo** em vez de `500 database operation failed` — sem isso a interface não conseguiria dizer ao analista o que corrigir. Mudança de comportamento da API.

**5. `docs/API.md`**
- Contrato de todas as rotas (24 caminhos), enumerações, formato de erro, o teto de sightings e os limites conhecidos — entre eles, o fato de a chave de versionamento do mapeamento ATT&CK ser incidente + evidência + técnica (a mesma técnica sob duas táticas colide).

**Não verificado / pendente**: os testes de `reject_symlink` e do engine rodaram só em Linux; o navegador real (CSP, acessibilidade) não foi exercitado — só `jsdom`. `scripts/doc_gate.py` já falhava antes (exige o pin `1.3.14` do Bun, removido pela ADR-0009) e não foi alterado.

## Unreleased — 2026-08-28 — default de data_root por plataforma (evita pastas sincronizadas na nuvem)

- `src/config.rs`: `default_data_root()` agora resolve um diretório de dados de usuário estável em toda plataforma, não só no Windows. macOS ganha `~/Library/Application Support/DFIR/storage`; Linux ganha `$XDG_DATA_HOME/dfir/storage` (ou `~/.local/share/dfir/storage`). Antes, Unix caía sempre em `./storage` (relativo ao diretório de trabalho no momento em que o binário é iniciado).
- Motivação: `./storage` relativo ao CWD é um risco de exposição real para uma ferramenta forense — iniciar o binário a partir da Área de Trabalho/Documentos (pastas comumente sincronizadas por OneDrive Known Folder Move / iCloud Drive) sincronizaria evidência para a nuvem silenciosamente. `LOCALAPPDATA`/`Application Support`/`XDG_DATA_HOME` são justamente as pastas que essas ferramentas não redirecionam por padrão.
- Lógica de seleção refatorada para uma função pura (`select_data_root`, despachada por uma enum `HostPlatform` explícita) testável para as quatro combinações de plataforma a partir de qualquer host de CI, sem depender de qual SO roda a suíte nem mutar env vars reais.
- `src/main.rs`: o `data_root` resolvido passa a ser logado na inicialização, para que o caminho de armazenamento nunca fique implícito.
- Decisão completa: `docs/adrs/ADR-0014-data-root-por-plataforma.md`.
- 6 testes novos/reescritos em `src/config.rs`; `cargo fmt`/`check`/`clippy`/`nextest` não executados neste ambiente (sem `cargo`/`rustc`).

## Unreleased — 2026-08-28 — F1-05: IOCs e recomendações operacionais (fecha o backlog original)

- Novo módulo `src/persistence/ioc_recommendations.rs`, sem migration nova.
- `record_ioc_sighting`: duplicata do mesmo IOC (mesmo incidente+tipo+valor normalizado) **acumula** no agregado via `supersedes` (semântica diferente de Gap/ATT&CK/impacto, onde `supersedes` é substituição) — nenhuma ocorrência é perdida, `first_seen`/`last_seen` recalculados a cada nova ocorrência. Validação de IP via `std::net::IpAddr` da biblioteca padrão (zero dependência nova); checagens estruturais leves para domínio/hostname/email/URL/hashes.
- `insert_recommendation`: `supersedes` explícito e opcional (sem chave natural de deduplicação para recomendações); `supporting_result_ids` obrigatório, validado contra achados existentes, e ligado via `RelationType::DerivedFrom` (mecanismo de F1-01/04).
- RF-022 ("exportação preserva a mesma origem usada na UI") implementado literalmente: `GET /api/incidents/{id}/export` reusa a mesma função (`current_results_for_incident`) que as rotas de listagem.
- Novas rotas: `POST/GET /api/incidents/{id}/iocs`, `POST/GET /api/incidents/{id}/recommendations` (`?category=IMMEDIATE|STRATEGIC`), `GET /api/incidents/{id}/export`.
- `docs/adrs/ADR-0013-ioc-recomendacoes.md`, `docs/rtm.md` e `docs/FASE1_STATUS.md` atualizados.
- 13 testes novos; `cargo fmt`/`check`/`clippy`/`nextest` não executados neste ambiente (sem `cargo`/`rustc`) — status `IMPLEMENTED / PENDING-HUMAN`.
- **Com este ticket, todo o backlog de `.scratch/fase1-backlog/issues/` fornecido pelo operador está implementado.**

## Unreleased — 2026-08-28 — F1-04: ATT&CK mapping, caminho crítico e matriz de impacto

- Novo módulo `src/persistence/attack_impact.rs`, sem migration nova — ATT&CK mapping, estágio de caminho crítico e avaliação de impacto reusam `analytical_results` (F1-01) diretamente.
- `AttackTactic` (14 táticas oficiais, hardcoded — estrutura estável) + validação estrutural de `technique_id`/`sub_technique_id` (`T####`/`T####.###`); catálogo de ~600+ técnicas do MITRE **não** embutido, mesma decisão de risco de transcrição da ADR-0011.
- "Relações não suportadas por evidência são marcadas como incerteza, não como fato" implementado como validação: confidence HIGH/CONFIRMED sem uma `basis` de evidência não-vazia é rejeitada; payload grava `evidence_status: GAP|DOCUMENTED` explicitamente.
- Caminho crítico: estágios são resultados independentes ligados por `RelationType::SequenceNext` (suporta ramificação, não só cadeia linear); novo helper genérico `analytical_results::insert_relation` em F1-01 para ligar estágios após criados.
- `ImpactLevel` (NONE/LOW/MEDIUM/HIGH/CRITICAL); `financial_impact` nunca fabricado quando não há dados.
- Novas rotas: `POST/GET /api/incidents/{id}/attack-mappings`, `GET /api/incidents/{id}/critical-path` (+`/stages`, `/links`), `GET/POST /api/incidents/{id}/impact-assessment`.
- `docs/adrs/ADR-0012-attack-caminho-critico-impacto.md`, `docs/rtm.md` e `docs/FASE1_STATUS.md` atualizados.
- 15 testes novos em `attack_impact.rs` + 2 em `analytical_results.rs`; `cargo fmt`/`check`/`clippy`/`nextest` não executados neste ambiente (sem `cargo`/`rustc`) — status `IMPLEMENTED / PENDING-HUMAN`.

## Unreleased — 2026-08-27 — F1-03: NIST Current/Target Profile e Gap Analysis reproduzível

- Novas tabelas `nist_profiles`/`nist_profile_entries` (`migrations/0008_nist_profiles.sql`), append-only, mesma disciplina de versionamento via `supersedes` de F1-01.
- Novo módulo `src/persistence/nist_profiles.rs`: `compute_gaps` (função pura, determinística, sem acesso a banco) compara Current vs Target por `implementation_status` e persiste cada gap como `AnalyticalResultType::NistGap` (F1-01) via `run_gap_analysis`.
- Decisão deliberada: o catálogo oficial de ~106 subcategorias do NIST CSF 2.0 **não** é embutido (risco de transcrição incorreta em conteúdo de conformidade real); `subcategory_id` é validado apenas na forma estrutural (`FUNCTION.CATEGORY-NN`), a descrição vem do analista. Ver `docs/adrs/ADR-0011-nist-profile-gap.md`.
- Tier (seção 13.2 da spec) é registrado como contexto do profile inteiro, nunca por subcategoria, conforme a spec exige explicitamente.
- Pré-requisito descoberto durante a implementação: não existia CRUD de `incidents` na API — adicionado (`POST/GET /api/incidents`, `GET /api/incidents/{id}`) nesta mesma rodada.
- Novas rotas: `POST/GET /api/incidents/{id}/nist/profiles`, `POST/GET /api/incidents/{id}/nist/gap-analysis`.
- `docs/rtm.md` e `docs/FASE1_STATUS.md` atualizados.
- 12 testes escritos (7 puros sobre `compute_gaps`, 5 de integração); `cargo fmt`/`check`/`clippy`/`nextest` não executados neste ambiente (sem `cargo`/`rustc`) — status `IMPLEMENTED / PENDING-HUMAN`.

## Unreleased — 2026-08-27 — correções de build reportadas pelo operador (Windows + rust-analyzer)

- **Bug real de Windows em `scripts/verify_frontend.py`**: invocava `node_modules/.bin/esbuild`/`.bin/vitest` diretamente, o que falha no Windows (`WinError 193` — `.bin/esbuild` sem extensão é o script Unix, não um executável Win32; `subprocess` sem shell não interpreta `.cmd`). Corrigido: tudo passa a rodar via `node <script>.mjs`, nunca via shim `.bin`. Novo `frontend/verify-build.mjs` (standalone, separado de `build.mjs` de propósito) para a verificação independente do bundle.
- Confirmado por execução real no Windows do operador (`just frontend-verify`): `npm run build`, `npm test` (vitest 8/8) e `npm run typecheck` (`tsc --noEmit`) passam sem alteração — a migração de Bun para Node.js/npm (ADR-0009) funciona no ambiente de destino real, não só no sandbox Linux usado para desenvolver.
- **Avisos `dead_code` (rust-analyzer) em `src/persistence/analytical_results.rs`**: variantes de `AnalyticalResultType`/`RelationType` e funções de leitura (`get`, `current_for_correlation_key`, `history_for_correlation_key`, `list_by_incident`, `related_to`, `insert`, `from_str`) ainda não têm chamador de produção — são API pública de F1-01 para F1-03/04/05 consumirem. Adicionado `#![allow(dead_code)]` de nível de módulo, com justificativa documentada, em vez de suprimir item a item.
- **Avisos `dead_code` em `engine/src/yara_scan.rs`**: `RULESET_LICENSE`/`RULESET_SOURCE_ORIGIN` declaradas mas nunca usadas. Corrigido de verdade (não suprimido): passam a ser incluídas no item `YARA_MATCH` emitido (`ruleset_license`, `ruleset_source_origin`) e no `provenance` persistido em `analytical_results` — reforça a exigência de "origem" do próprio ticket F1-02 em vez de só calar o aviso.
- `docs/adrs/ADR-0009-substituicao-bun-por-node-npm.md` recebeu um adendo com a causa raiz e a correção do bug de Windows.

## Unreleased — 2026-08-25 — F1-02: YARA-X e resultados estruturados

- Ruleset YARA-X curado e original (`engine/rules/dfir_mvp_ruleset.yar`, 4 regras: EICAR, PowerShell codificado, palavras-chave de ransom note, comando Mimikatz) — nenhuma regra externa importada.
- Novo módulo `engine/src/yara_scan.rs`: compila o ruleset embutido, varre evidência, nunca expõe bytes casados (apenas offset/comprimento), SHA-256 do ruleset recalculado em runtime.
- `dfir-protocol`: novo campo obrigatório `Request.analysis_kind` (`EVTX_TIMELINE`/`YARA_SCAN`); `SCHEMA_VERSION` "2"→"3".
- `engine/src/main.rs`: `process_request` roteia por `analysis_kind`; lógica EVTX extraída para `process_evtx_request` sem mudança de comportamento.
- `migrations/0007_analysis_job_kind.sql`: coluna `kind` em `analysis_jobs`.
- `src/web/mod.rs`: handler `analyze` aceita `kind`; matches YARA persistidos em `analytical_results` (F1-01) na mesma transação de conclusão do job; novo `GET /api/evidence/{id}/analytical-results`.
- `docs/adrs/ADR-0010-yara-x-ruleset-curado.md`, `docs/FASE1_STATUS.md` e `docs/rtm.md` atualizados.
- Corrigido erro de build real reportado pelo operador: lint `SqlSafeStr` do sqlx 0.9 em `src/persistence/analytical_results.rs` (5 queries usavam `format!()` para montar SQL; trocadas por strings literais com `LIMIT` como bind parameter).
- `cargo fmt`/`check`/`clippy`/`nextest` não executados neste ambiente (sem `cargo`/`rustc`) — status `IMPLEMENTED / PENDING-HUMAN`. `Cargo.lock` precisa de `cargo build` sem `--locked` uma vez para resolver `yara-x`/`sha2` antes de qualquer gate `--locked`.

## Unreleased — 2026-08-25 — F1-01: modelo analítico comum e versionamento de resultados

- Nova tabela `analytical_results` (`migrations/0006_analytical_results.sql`), append-only (mesmo padrão de `audit_events`/`chain_of_custody`), compartilhada por F1-02 (YARA-X), F1-03 (gap NIST), F1-04 (ATT&CK/caminho crítico/impacto) e F1-05 (IOCs/recomendações): identificadores, evidência/incidente de origem, confiança, provenance e versão de regra/modelo, com histórico preservado via `supersedes` (nunca `UPDATE`).
- Nova tabela `analytical_result_relations` (`DERIVED_FROM`, `SEQUENCE_NEXT`) para caminho crítico e recomendações que referenciam achados.
- Novo módulo `src/persistence/analytical_results.rs` com validação de campos obrigatórios por tipo, inserção transacional com evento de auditoria, consultas (`current_for_correlation_key`, `history_for_correlation_key`, `list_by_evidence`, `list_by_incident`, `related_to`) e 12 testes.
- `src/domain/` (diretório vazio, órfão, sem `mod domain;`) removido.
- `docs/adrs/ADR-0008-modelo-analitico-comum.md`, `docs/FASE1_STATUS.md` (novo) e `docs/rtm.md` atualizados.
- `cargo fmt`/`check`/`clippy`/`nextest` não executados neste ambiente (sem `cargo`/`rustc`) — status `IMPLEMENTED / PENDING-HUMAN`, não `VERIFIED`.

## Unreleased — 2026-08-25 — substituição de Bun por Node.js/npm no frontend

- Bun removido da toolchain de build/test do frontend por não ser instalável em todos os ambientes de execução do projeto (rede restrita, sem acesso a `bun.sh`); ver `docs/adrs/ADR-0009-substituicao-bun-por-node-npm.md` e adendo à ADR-0005.
- `frontend/build.ts` (`Bun.build`) substituído por `frontend/build.mjs` (`esbuild`); `bun:test` substituído por `vitest` em `app.test.ts` (troca de import de uma linha); `frontend/tsconfig.json` adicionado com um gate novo `npm run typecheck`.
- `scripts/verify_frontend.py` reescrito para rebuild independente via `esbuild` e checagem de texto Deno/Bun obsoleto; `scripts/validate_fase0.py`, `justfile`, `toolchains.toml` e todos os documentos normativos que citavam Bun foram atualizados.
- Especificação canônica (`docs/PROJECT_SPEC_v6_4_5.md` + cópia raiz, mantidas byte-idênticas): nova nota de implementação superseded (3), decisão `D45` marcada `SUPERSEDED`, nova decisão `D46`.
- Executado nesta árvore, com evidência real: `npm install`, `npm run typecheck`, `npm run build` (`dist/app.js` 11.9kb, IIFE válido; CSS byte-idêntico), `npm test` (8/8), `scripts/verify_frontend.py` (PASS), `scripts/validate_fase0.py` (0 erros, 1 warning — apenas Cargo indisponível neste ambiente). Cargo/Rust permanecem `PENDING-HUMAN` neste ambiente.

## Unreleased — 2026-08-24 — transição Fase 0 → Fase 1

- Registrada a Definition of Ready da transição como satisfeita, conforme declaração do responsável de que todos os gates/testes da Fase 0 foram concluídos.
- Criado `docs/FASE1_BACKLOG.md` com o backlog final do MVP DFIR em fatias verticais e dependências explícitas.
- Criados tickets locais em `.scratch/fase1-backlog/issues/` para as doze fatias da Fase 1.
- Atualizada `docs/rtm.md` para rastrear o backlog de Fase 1 sem reabrir a base já aceita da Fase 0.
- RF-021 (Sigma) e RF-023 (Velociraptor) permanecem fora do caminho crítico por serem Fase 4.

## Unreleased — 2026-08-24 — correções operacionais da timeline

- Corrigido `frontend/dist/app.js` para alinhar o artefato servido ao contrato atual da timeline; `record_offset` não é mais exibido na UI.
- O parser frontend agora remove campos fora do contrato ao normalizar eventos.
- Corrigido o filtro da timeline para pesquisar o conjunto de eventos da análise atual, incluindo Event ID, provider, channel, descrição e referências de evidência.
- Adicionado descarte explícito de evidência local: arquivo importado e saídas de análise associadas são removidos, a evidência fica `DESTROYED` e as trilhas de auditoria/custody permanecem preservadas.
- Evidências destruídas são ocultadas do catálogo e não podem ser reanalisadas.
- Build/test Rust e Bun permanecem pendentes de execução no Windows; verificações estáticas/documentais e smoke independente do bundle passaram neste ambiente.

## Unreleased — 2026-08-24 (correção da timeline EVTX)

- Removida a exposição de `binxml_offset` como `record_offset`: o campo do `evtx 0.12.2` é interno à representação do registro/chunk e não é um offset físico global adequado para o contrato da timeline.
- A saída do engine preserva `raw_reference=record:<EventRecordID>` conforme a especificação, sem inventar uma segunda referência física.
- A API da timeline passa a ordenar cronologicamente por timestamp UTC e deduplicar por `evidence_id + raw_reference`, mantendo descrição e metadata do evento.
- O frontend passou a renderizar a descrição do evento, referência da evidência e timestamp explicitamente em UTC.
- Adicionados testes de regressão para identidade, ordenação, deduplicação e descrição.
- `cargo check` e Bun continuam pendentes neste ambiente; a evidência anterior de `cargo check` foi invalidada como `STALE` após a reprodução das falhas no Windows.

## Unreleased — 2026-08-22 (remoção de containerização)

Solicitado pelo usuário: remoção completa do empacotamento em container. Build/test **não executados** neste ambiente (sem toolchain Cargo/Bun disponível); gates marcados como PENDING conforme a convenção deste projeto.

- Removidos `containers/Containerfile.app`, `quadlet/dfir-app.container` e `compose.yaml` do repositório. Não existe mais artefato de container no produto.
- Removido `scripts/release_gate.py` e o alvo `release-gate` do `justfile`: o gate existia exclusivamente para validar o digest da imagem de container contra o Quadlet, e não tem mais objeto sem containerização.
- Atualizados `scripts/validate_fase0.py`, `tests/security/fase0_config.py` e `tests/security/rejected_external_tools.py` para parar de exigir/ler os arquivos removidos; as checagens de segurança que independem de container (bind loopback por padrão, `DFIR_BIND_ALLOW_NON_LOOPBACK` explícito) permanecem inalteradas.
- `docs/PROJECT_SPEC_v6_4_5.md` (e o espelho `nist-senai-proj_integrador_v6_4_5.md`, mantidos byte-idênticos conforme `doc_gate.py`): Seção 28 ("Containerização e isolamento") reescrita para registrar a remoção; tabelas de decisão, árvore de diretórios e entradas XML normativas (D02/D15, `runtime-container`, `deployment`) atualizadas. Menções remanescentes em prosa não normativa (notas de DR, ressalvas de WSL2, tabela de versões históricas) permanecem como estavam e são cobertas por nota de supersessão no topo do documento — mesmo padrão já usado para a decisão de sandbox.
- Adendo formal em `docs/adrs/ADR-0007-local-forensic-scope-simplification.md`: o que a ADR já classificava como "Podman opcional" passa a "removido por completo".
- `README.md`, `docs/FASE0_STATUS.md`, `docs/RELEASE_GATE.md`, `tests/security/fase0.md`, `src/config.rs` (mensagem de erro e comentário): referências a container/Podman/Quadlet/Compose removidas ou generalizadas para descrever o deployment sem assumir container.

## Unreleased — 2026-08-21 (auditoria de arquitetura externa)

Revisão de arquitetura solicitada pelo usuário (escopo: repositório completo). Build/test **não executados** neste ambiente (sem toolchain Cargo/Bun disponível); gates marcados como PENDING conforme a convenção deste projeto — revisão estática apenas.

- Removida a instalação de `bubblewrap` e a variável `DFIR_BWRAP_PATH` de `containers/Containerfile.app`: nenhum código em `src/`ou `engine/` jamais invocou `bwrap`. Essa configuração ficou órfã depois que o sandbox de processo baseado em bind-mount foi substituído por validação/canonicalização de caminhos (nota de implementação em `docs/PROJECT_SPEC_v6_4_5.md`, ADR-0007); mantê-la só adicionava superfície de ataque e sugeria falsamente isolamento ativo.
- Endurecida a resolução de caminho de entrada contra o TOCTOU canonicalize-then-open: `dfir-app::process::launch_process` e `dfir-engine::resolve_input` agora rejeitam symlinks/reparse points no componente final do caminho (em Windows, via bit `FILE_ATTRIBUTE_REPARSE_POINT`, cobrindo tanto symlinks quanto junctions, já que junctions não exigem modo desenvolvedor). Isso estreita, mas não elimina, a janela de corrida entre a checagem e o `open` real feito pelo `dfir-engine` em processo separado; fechar completamente exigiria verificação de identidade de handle já aberto (`/proc/self/fd` no Linux, `GetFinalPathNameByHandleW` no Windows), fora de escopo enquanto o modelo de ameaça permanecer o operador local único (ADR-0007).
- Corrigido `AppConfig::data_root`: antes usava `./storage` (relativo ao CWD) em qualquer plataforma, contradizendo o `%LOCALAPPDATA%\DFIR\storage` já documentado no README para Windows. Adicionado `default_data_root()`/`windows_data_root()` para que o padrão real bata com o documentado.
- Removidos `package.json`, `tsconfig.json`, `index.ts` e `bun.lock` da raiz do repositório: scaffold de `bun init` nunca referenciado por `justfile`, pelos scripts de gate ou pela árvore canônica em `docs/PROJECT_SPEC_v6_4_5.md` (que só lista `frontend/package.json`). O frontend real (`frontend/`) não depende deles.
- Adicionado `TimeoutLayer` (tower-http) como backstop de requisição em `src/web/mod.rs`, usando `engine_timeout` configurado + margem de 120s: a feature `"timeout"` do `tower-http` já era dependência declarada mas nunca usada, deixando uploads travados (`upload_evidence` não tem prazo por chunk) sem limite algum.
- Consolidados espaçamentos de borda (`border-radius`) e tamanhos de fonte pequenos em `frontend/src/app.css` em uma escala de tokens (`--radius-sm/md/lg`, `--text-2xs/xs/sm`) em vez de seis e cinco valores quase-duplicados respectivamente; sem mudança de layout, apenas arredondamento de décimos de rem.

- Corrigido o fluxo do frontend após a remoção do sandbox de processo da arquitetura ativa: a interface agora descreve corretamente o engine como subprocesso separado e supervisionado.
- Regenerado o bundle estático do frontend para manter `frontend/dist` alinhado ao TypeScript/CSS versionado.
- Melhorado o diagnóstico de falhas de comunicação: erros de rede agora informam explicitamente quando o `dfir-app` local não respondeu, em vez de expor apenas `TypeError: Failed to fetch`.
- A interface verifica `/health/live` ao iniciar e sinaliza o estado do serviço local.


## 2026-08-20 — MVP evidence pipeline

- Added local evidence import with streaming BLAKE3 and SHA-256 hashing, UUID-backed storage references and audit event `EVIDENCE_IMPORTED`.
- Added persisted analysis jobs, EVTX execution, result retrieval and job status endpoints.
- Completed the browser MVP workflow: evidence selection/import, evidence catalog, EVTX analysis action, job summary and timeline rendering/filtering.
- SHA-256 is retained only for evidence interoperability as required by the current specification; the audit chain remains BLAKE3/JCS.
## [Unreleased] — Windows runtime gate compatibility

- Corrigido `scripts/run_runtime_test.py` para iniciar o teste PowerShell nativo do Windows com `-ExecutionPolicy Bypass` apenas no processo filho, evitando `AuthorizationManager check failed` em execução não interativa sem alterar a policy persistente do host.
- Documentado que o bypass pertence exclusivamente ao tooling de QA e não ao runtime do produto.

## [Unreleased] — deep audit hardening

- Corrigido um erro de mapeamento entre `input_reference` lógico (`evidence/...`) e a raiz física montada no engine: a análise agora remove somente o namespace lógico antes de resolver o arquivo dentro da raiz de entrada.
- Endurecido o sandbox por job para análise normal: Linux monta somente o arquivo de evidência solicitado em modo somente leitura, e Windows concede leitura somente ao arquivo solicitado, mantendo traversal mínimo dos diretórios-pai. O modo smoke continua usando uma raiz de diretório dedicada.
- Adicionados testes de regressão para namespaces não montados e para a resolução do namespace lógico `evidence/`.

- Tornada atômica a importação de evidência: registro em `evidence`, evento `EVIDENCE_IMPORTED` e primeira entrada de chain of custody agora compartilham uma única transação SQLite `BEGIN IMMEDIATE`, evitando evidência órfã ou sem proveniência imutável.
- Tornadas atômicas as transições de início/conclusão/falha de jobs com seus respectivos eventos de audit e custody; o estado persistido do job não avança sem os registros de proveniência correspondentes.

## [Unreleased] — audit hash simplification

- Removido o suporte SHA-256 da cadeia de auditoria: novos eventos usam `BLAKE3-256-v2-JCS` e eventos `BLAKE3-256-v1` continuam verificáveis.
- A dependência `sha2` permanece somente para o hash obrigatório das evidências importadas; ela não participa da audit chain.


- Corrigido `scripts/engine_protocol_smoke.py`: o teste não fixava `DFIR_ENGINE_INPUT_ROOT`, então herdava o default `/input` do engine (inexistente fora do sandbox), fazendo o canonicalize da *raiz* falhar antes de sequer olhar o arquivo EVTX ausente. Como raiz-ausente e arquivo-ausente compartilham o código `INPUT_UNAVAILABLE`, o teste reportava PASS sem nunca exercitar o caminho que afirma testar. Agora usa uma raiz temporária real e valida o texto específico da mensagem, não só o código. Corrigida também a decodificação implícita de locale do `subprocess` (mojibake em mensagens de erro localizadas do SO em hosts não-UTF-8, ex. Windows).
- Removidos do runtime local login, sessões, RBAC, autorização por incidente, bootstrap administrativo, CSRF e rate limiting HTTP.
- Mantidos preservação de evidências, chain of custody/audit trail, engine separado e sandbox obrigatório.
- Podman passa a ser empacotamento/hardening opcional; o produto continua local/offline.
- Auditoria usa BLAKE3-256-v2-JCS para novos eventos e mantém verificação de BLAKE3-256-v1; suporte SHA-256 legado foi removido por não haver eventos históricos de produção nesta árvore.
- Protocolo JSONL passou a streaming por batches com limites explícitos e teardown do engine em timeout/erro.
- Frontend migrado para Bun como build/test toolchain, sem APIs Bun no código executado pelo browser.
- Migration 0003 remove o schema histórico de autenticação/RBAC de bancos existentes sem alterar os checksums das migrations históricas.
- Registrada a evidência de `cargo check --workspace --locked --all-targets` com sucesso em Windows.
- Mantidos testes, Clippy, Nextest, supply chain e runtime como gates pendentes para execução no ambiente de destino.

## 2026-08-20 — Simplificação para ferramenta forense local

- Removidos do runtime ativo login, sessões, RBAC, autorização por incidente, bootstrap administrativo, CSRF e rate limiting HTTP.
- Criada migration `0003_local_forensic_scope.sql` para remover essas estruturas de bancos já migrados sem reescrever migrations históricas.
- Mantidos processo separado do engine, sandbox por job, audit chain, storage local e controle de recursos.
- Registros históricos de autenticação permanecem apenas em migrations/ADRs arquivados para preservar rastreabilidade.


## 2026-08-19 — revisão final da Fase 0

- Corrigida autenticação para tratar senhas curtas/longas como falhas normais, mantendo a verificação dummy para entradas excessivas.
- Corrigida contagem de `max_output_bytes` para considerar apenas `RESULT` e incluir o delimitador JSONL de forma consistente entre app e engine.
- Streaming do engine agora divide batches dinamicamente para respeitar `MAX_LINE_BYTES`, inclusive para eventos individuais grandes.
- Cadeia de auditoria passou a usar `BLAKE3-256-v1` com encoding length-prefixed inequívoco; hashes SHA-256 legados continuam verificáveis.
- O gate estático/documental continua sem erros ou warnings.

## Unreleased — JSONL stack-overflow hardening

- Investigated `STATUS_STACK_OVERFLOW` observed in `dfir-app` while analyzing EVTX evidence.
- Added an iterative JSON nesting-depth check before `serde_json` deserialization on the app↔engine JSONL boundary and stored result reader.
- Centralized `MAX_JSON_DEPTH=256` in `dfir-protocol` and added regression coverage.
- Removed unnecessary full `ResultMessage` deserialization in the app; RESULT status is read from the already-parsed envelope.
- Configured explicit 8 MiB Tokio worker stacks for the app and engine as a defensive margin; this is not the primary safety control.
- Verified the supplied `DE_RDP_Tunneling_4624.evtx` metadata: one chunk and 19 records, so the observed crash is not explained by an oversized evidence file alone.


## 6.4.5 — 2026-08-18 — correções de compilação e limpeza de código

- corrigido o `query_as` de autenticação para refletir exatamente as quatro colunas selecionadas (`id`, `password_hash`, `role`, `locked_until`);
- removido o import `serde::Serialize` não utilizado do engine;
- revisão orientada pelos diagnósticos do rustc/rust-analyzer recebidos no Windows;
- preservada a regra de não silenciar `dead_code`/`unused_imports`: código sem consumidor foi removido ou ligado a um caminho real quando aplicável;
- documentação/RTM/status atualizados para registrar esta correção como manutenção de compilabilidade;

## 6.4.3 — hardening preventivo antes do congelamento

- correção do backoff de autenticação: bloqueio agora impede tentativas enquanto `locked_until` estiver ativo;
- verificação dummy é usada também para contas temporariamente bloqueadas;
- proteção de contexto `<script type="application/json">` para dados da timeline;
- páginas de login/painel marcadas com `Cache-Control: no-store`;
- login só cria sessão depois da auditoria de sucesso; falha de auditoria não deixa sessão utilizável pendente;
- frontend versionado é comparado contra transpile independente no gate de verificação;
- Deno de referência atualizado para 2.9.4;
- HTTP inseguro fica restrito a binds loopback;
- base da imagem atualizada para Alpine 3.24.0 / Rust 1.97.1-alpine3.24;
- teste de regressão adicionado para conta bloqueada rejeitar a senha correta;

## 6.4.0 — 2026-08-17

### Arquitetura

- runtime de produção formalmente reduzido a Rust + assets frontend estáticos;
- TypeScript adotado como frontend compilado, sem runtime Node/Bun/Deno;
- Deno 2.9 LTS adotado como toolchain de frontend/build;
- CPython 3.14.6 + Shell formalizados como tooling de engenharia/QA;
- HTTP/JSON definido como transporte browser→app;
- SSE reservado a eventos unidirecionais de jobs quando necessário;
- WebSocket removido do MVP;
- timeline Fase 0 usa renderer local sem dependência externa obrigatória;
- `vis-timeline`/D3 condicionados a necessidade demonstrada.

### Frontend

- `frontend/src/app.ts` criado;
- `frontend/src/app.css` criado;
- artefatos `frontend/dist/app.js` e `frontend/dist/app.css` versionados;
- `dfir-app` passa a servir assets estáticos incorporados ao binário;
- Alpine.js e HTMX removidos da arquitetura ativa da Fase 0 para reduzir dependências; HTML server-rendered + TypeScript compilado são suficientes para a interface inicial.

### Documentação

- especificação promovida para v6.4.0;
- v6.3.13 arquivada como baseline histórico;
- nova ADR-0005 para a arquitetura híbrida por classes de runtime;
- manifesto de autoridade, Definition of Ready, RTM, status e release gate atualizados;
- Apêndice F ampliado para runtime/build/tooling e contrato frontend.

## 6.3.13 — histórico

Mantido em `docs/archive/` para rastreabilidade. O baseline anterior permanece disponível para comparação, mas não é a autoridade normativa do projeto atual.

## 2026-08-18 — Portabilidade do justfile

- `justfile` agora usa PowerShell nativo no Windows e Bash no Unix/WSL2.
- Receitas compostas com semântica Unix foram substituídas por wrappers Python portáveis.
- `validate_fase0.py` passou a reconhecer a verificação independente de frontend implementada por `scripts/verify_frontend.py`.

## [Unreleased] — Aider tooling — Removido em 2026-11 por ADR-0027

- adicionados `.aider.conf.yml`, `.aiderignore` e `CONVENTIONS.md`;
- Aider configurado sem commits automáticos e com contexto normativo versionado;
- evidências, bancos, secrets e artefatos gerados excluídos do contexto automático do Aider;
- README atualizado com as regras de uso do Aider.

## Unreleased — local runtime hardening

- Replace hand-written JSONL framing with `tokio-util` `LinesCodec` and bounded framing.
- Bump JSONL `schema_version` from `1` to `2` to make batched `RESULT` streaming explicit and reject mixed-version peers.
- Stream EVTX results in bounded protocol batches instead of accumulating the full result set.
- Add an application-level engine concurrency semaphore, configurable by `DFIR_ENGINE_MAX_CONCURRENT_JOBS`.
- Add periodic cleanup of expired sessions.

BLAKE3 is intentionally not substituted into the audit chain in this change: the current audit chain is an existing persisted format, and changing its hash construction requires an explicit format migration and compatibility plan.

### Deferred after toolchain-enabled validation

- `tower_governor` login rate limiting and Prometheus metrics are intentionally not included in this patch because the available execution environment has no Cargo/Rust toolchain and cannot regenerate/verify their larger lockfile changes safely. The existing account lockout/backoff remains in place.

## Unreleased — Windows sandbox + Bun toolchain

- Migrated frontend build/test tooling from Deno to Bun 1.3.14.
- Added pure browser-layer tests for timeline input validation.

### Timeline EVTX: timestamp e provider — 2026-08-24
- Corrigida a fonte do timestamp semântico: `Event/System/TimeCreated/#attributes/SystemTime` agora tem precedência sobre `EvtxRecord.timestamp`.
- Corrigida a extração do provider a partir de `Event/System/Provider/#attributes/Name`.
- Adicionados testes de regressão para timestamp semântico e provider do formato JSON normalizado pelo parser EVTX.
