<!-- doc-type: review | status: superseded -->
# Revisão 2026-10-02 — auditoria de documentação e código contra a demanda original

> **HISTÓRICO.** Registro da auditoria que motivou a ADR-0018. A numeração de tickets proposta na Parte 1 (F1-13..F1-28) e as propostas F1-29..F1-35 da Parte 2 foram **substituídas** pelo backlog vigente em `docs/FASE1_BACKLOG.md` (F1-13..F1-33; ver ADR-0018 e `docs/DEMAND_TRACE.md`). Os identificadores AUD-xx continuam válidos e são citados nos tickets. Achados `[P]` (prováveis) permanecem pendentes de confirmação com execução. AUD-33 e AUD-34 não constam nas partes abaixo: AUD-34 é o erro de compilação E0505 de `nist_profiles.rs`, encontrado depois, ao compilar (ver `docs/FASE1_STATUS.md`).

---

# Parte 1 — Auditoria técnica e backlog inicial

# Auditoria técnica e backlog — DFIR/NIST (pacote egui 0.36.2, 2026-09-26)

Data da auditoria: 2026-10-01. Escopo: toda a documentação (README, AGENTS, ADR-0001..0017, status, RTM, spec v6.4.5, backlog Fase 1) e todo o código (`src/`, `engine/`, `protocol/`, `migrations/`, `scripts/`).

## 0. Método e limites

- **Leitura estática** de todo o código-fonte e da documentação normativa.
- **Executado:** `validate_fase0.py`, `doc_gate.py`, `tests/security/fase0_config.py` e `rejected_external_tools.py` (todos PASS); comparação `Cargo.toml` × `Cargo.lock` via `tomllib`.
- **Não executado:** `cargo fmt/check/clippy/test/nextest/deny/audit/vet`. O ambiente não tem o toolchain 1.97.1. Nada aqui é "VERIFIED" no sentido do AGENTS §10.
- **Certeza dos achados:** `[C]` = confirmado lendo o código/arquivos; `[P]` = provável, depende de comportamento de runtime ou de crate externa e precisa de teste para confirmar.

---

## 1. Veredito: o projeto segue a demanda original?

**A fundação e o núcleo analítico seguem a demanda. A camada de entrega (o que a SENAI/Avivatec efetivamente recebe) ainda não existe.**

| Requisito de entrega da demanda | Estado | Observação |
|---|---|---|
| 1. Relatório forense completo | ✖ Ausente | Sem relatório, sem preview, sem perfis executivo/técnico (F1-07 não iniciado). Faltam também entidades para causa-raiz, hipóteses e lacunas. |
| 2. Vulnerabilidades exploradas + ATT&CK | ◐ Parcial | ATT&CK mapping existe (manual, com evidência e confiança). **Não existe entidade Finding/Vulnerabilidade, severidade nem matriz de risco** (AUD-12). |
| 3. Maturidade NIST CSF 2.0 | ◐ Quase | Current/Target/Gap/Tier persistidos e reproduzíveis. Catálogo CSF não embutido (decisão consciente, ADR-0011). Sem documento gerado. |
| 4. Recomendações | ◐ Quase | Imediato/estratégico persistidos. Entrada manual; export parcial. Sem documento gerado. |
| 5. Hardening | ✖ Ausente | F1-06 não iniciado (Ansible, CIS, dry-run). |
| 6. Criptografia e proteção de dados | ✖ Ausente | Nenhum ticket cobre o conteúdo (classificação, trânsito, repouso, chaves, rotação, recuperação). RF-030 só pede doc de volume. |

**Cobertura dos 43 RFs de fase MVP** (spec §39; avaliação por leitura, não VERIFIED):

| Estado | Qtde | RFs |
|---|---|---|
| ✔ Implementado, sem defeito achado | 16 | 001, 002, 009, 010, 011, 015, 018, 020, 022, 027, 028, 029, 035, 036, 038, 044 |
| ◐ Implementado com defeito/lacuna | 14 | 003, 004, 005, 006, 007, 008, 014, 016, 019, 033, 034, 037, 046, 047 |
| ✖ Não implementado | 13 | 012, 013, 017, 025, 026, 030, 032, 039, 040, 041, 042, 043, 045 |

**Desvios de escopo/governança** (detalhe em DOC-01..05): a troca para egui é decisão legítima do operador, mas foi registrada como "requisito explícito da demanda SENAI 12412", e a demanda não menciona tecnologia de UI. Isso deveria ser uma decisão de arquitetura (nível 3) com bump de spec, não uma precedência nível 1.

**Pontos fortes que devem ser preservados:** audit chain BLAKE3-JCS com âncora de contagem+head; triggers append-only; protocolo JSONL com limites e profundidade; falha fechada no spawn do engine; `reject_symlink` corrigido; modelo `supersedes` coerente; ADRs honestos sobre o que não foi verificado.

---

## 2. Achados

### P0 — Bloqueia gates

| ID | C | Achado | Local |
|---|---|---|---|
| AUD-01 | C | **`Cargo.lock` dessincronizado.** `Cargo.toml` pede `eframe/egui/egui_extras 0.36.2`; o lock tem `0.33.3`. Todo alvo do `justfile` usa `--locked`, então `check/test/clippy/build/run` falham. O CHANGELOG admite o lock não regenerado, mas `validate_fase0.py` não detecta a divergência. | `Cargo.toml`, `Cargo.lock`, `justfile` |

### P1 — Alto (integridade forense, corretude, MVP)

| ID | C | Achado | Local |
|---|---|---|---|
| AUD-02 | C | **Destruição de evidência não é atômica.** O arquivo e os diretórios de staging são apagados **antes** da transação que registra `EVIDENCE_DESTROYED`. Se a transação falhar, o arquivo se perdeu, o banco diz `ACTIVE` e a auditoria não registra nada. | `service::destroy_evidence` |
| AUD-03 | C | **Jobs presos em `PROCESSING`.** (a) Se `begin_immediate` falhar depois do engine, `analyze` retorna sem marcar o job (dois pontos). (b) Fechar a janela durante a análise deixa o job `PROCESSING`. (c) Não há recovery na partida. (d) `destroy_evidence` recusa para sempre evidência com job `PROCESSING`. | `service::analyze`, `gui/backend.rs` |
| AUD-04 | C | **Erros de registro EVTX descartados pelo app (viola RF-006).** O engine emite `PARSER_RECORD_ERROR` (não fatal), mas o app só grava mensagens `RESULT` em `results.jsonl`. Esses erros não vão para arquivo, banco nem auditoria. | `engine/runner.rs`, `application/engine.rs` |
| AUD-05 | P | **Análise de EVTX real provavelmente estoura `OUTPUT_LIMIT`.** O teto é `MAX_OUTPUT_BYTES = 4 MiB` e cada evento carrega o `data` completo do registro. Pela ordem de grandeza (1–2 KB/evento) o limite chega em poucos milhares de eventos. Confirmar com um EVTX de produção. | `protocol::MAX_OUTPUT_BYTES`, `service::analyze` |
| AUD-06 | C | **Timeline trunca antes de ordenar.** Pega os primeiros 1000 eventos na ordem do arquivo e só depois ordena por UTC, então os eventos mais antigos podem não aparecer. Resultados vivem só em JSONL de staging (sem banco, sem paginação, leitura limitada a 2000 linhas). | `service::job_results` |
| AUD-07 | C | **Reexecutar YARA duplica resultados.** `correlation_key = yara:<regra>:<evidência>` com `supersedes: None` e sem unicidade. Cada scan acrescenta linhas "novas", e `list_by_evidence` mostra o histórico como se fosse vigente. Viola RF-039 e o contrato de F1-01. | `service::analyze`, `analytical_results::list_by_evidence` |
| AUD-08 | C | **"Vigentes" calculado sobre janela de 500 linhas.** `current_results_for_incident` usa `list_by_incident` (`LIMIT 500` por `created_at`). Achados vigentes antigos (ex.: IOC visto uma vez) somem da UI **e da exportação**, sem aviso. O próprio F1-03 já resolveu isso com `distinct_correlation_keys`; os demais tipos não. | `service/mod.rs`, `analytical_results.rs` |
| AUD-09 | P | **Regras YARA sem `wide`.** Strings de BinXML em EVTX são UTF-16LE; padrões ASCII de PowerShell/Mimikatz provavelmente nunca casam em `.evtx` (falso negativo silencioso no cenário BEC). EICAR funciona porque é ASCII em texto. | `engine/rules/dfir_mvp_ruleset.yar` |
| AUD-10 | C | **Evidência e incidente nunca são vinculados.** A coluna `evidence.incident_id` existe (migration 0005), mas o INSERT de importação não a preenche e a GUI não tem seletor. IOC/ATT&CK/caminho crítico pedem `evidence_id` **digitado à mão**; erro de FK vira "database operation failed". | `service::import_evidence`, `gui/iocs.rs`, `gui/attack.rs` |
| AUD-11 | C | **Robustez do MVP inexistente (RF-039/041/042/043).** Sem retry/backoff, sem `PERMANENTLY_FAILED` (o CHECK do schema só aceita PENDING/PROCESSING/COMPLETED/FAILED), sem recovery, sem SIGTERM/SIGINT/shutdown, sem checagem de espaço livre, sem idempotência. | `migrations/0004`, `service` |
| AUD-12 | C | **Modelo de dados insuficiente para os Entregáveis 1, 2 e 5.** `result_type` só admite YARA/NIST/ATT&CK/caminho/impacto/IOC/recomendação. Faltam Finding/Vulnerabilidade (severidade, impacto, matriz de risco), causa-raiz/hipótese/lacunas, movimentação lateral/escalonamento/exfiltração e classificação de dados/chaves. | `migrations/0006` |
| AUD-13 | C/P | **Supply chain da árvore de UI não validada.** `docs/SUPPLY_CHAIN.md` admite que `cargo deny/audit` não rodaram sobre winit/accesskit/glow/ashpd/zbus; `cargo vet` nunca foi configurado (AGENTS §8 exige). `[P]` `deny.toml` não lista licenças prováveis de `epaint_default_fonts` (OFL-1.1, UFL), o que quebraria `cargo deny check licenses`. | `deny.toml`, `docs/SUPPLY_CHAIN.md` |
| AUD-14 | C | **Mensagens de erro genéricas** ("analysis engine failed", "storage operation failed") contradizem o AGENTS ("nunca reduzir falha específica a mensagem genérica") e a spec §30.3 (o quê, impacto, ação, ID técnico). Há um teste que fixa o comportamento genérico. | `error.rs::user_message` |
| AUD-15 | C | **Isolamento do engine abaixo do que a documentação afirma (RF-033).** Sem `env_clear` (o engine herda todo o ambiente do app); `DFIR_ENGINE_INPUT_FILE` é definida pelo app mas **nunca lida pelo engine** (a "contenção por arquivo" só existe no app, e o engine enxerga toda a raiz `evidence/`); sem `kill_on_drop`; no Windows não há limite de memória nem Job Object. | `application/process.rs`, `engine/input.rs` |
| AUD-31 | C | **Sem fixture EVTX e sem teste positivo de parsing.** `tests/fixtures` está vazio. Nenhum teste parseia um `.evtx` válido (só arquivo corrompido e extensão errada). A camada `service` (import/analyze/destroy/timeline) não tem testes, nem a GUI. | `tests/`, `service/mod.rs` |

### P2 — Médio

| ID | C | Achado |
|---|---|---|
| AUD-16 | C | Custody chain **sem âncora** (head/contagem): remover a cauda de `chain_of_custody` é indetectável (a audit chain tem `audit_state`). Hashes sem chave/assinatura (`signature_reference` nunca usado). `actor` fixo `"local-analyst"`, sem usuário/host do SO. |
| AUD-17 | C | GUI não expõe custódia, auditoria, verificação (só CLI `audit-verify`), histórico de jobs, jobs com falha. RF-004/029 "verificável" fica fora do alcance do analista. |
| AUD-18 | C | UX abaixo da spec §30.3 / RF-046: filtro só texto livre (faltam período, tipo, host, usuário, origem); progresso só spinner (engine emite 0% e 10%); sem cancelamento; sem breadcrumbs; caminho crítico em lista (spec recomenda SVG de grafo); `timeline_filter` único compartilhado entre evidências. |
| AUD-19 | C | RF-034 / spec §4.3: a UI esconde o botão EVTX para outros formatos, mas não **declara** quais formatos são só preservados. |
| AUD-20 | C | IOC só por entrada manual: a spec §17 ("IOC extraction") e RF-019 ("extraídos com origem") pedem extração a partir de eventos, com campo de evento. Hoje `event_context` é texto livre. |
| AUD-21 | C | Evidência importada sem proteção contra escrita (só 0700 nos diretórios); sem `fsync` do diretório; arquivo parcial em `quarantine/` não é limpo em falha de I/O nem na partida. |
| AUD-22 | C | `import_evidence` não canonicaliza/rejeita symlink na origem escolhida (AGENTS §6 manda validar mesmo no caminho do operador). Modos `SANITIZED`/`DEMO` existem no schema, mas não na UI. |
| AUD-24 | C/P | `BEGIN IMMEDIATE` manual: cancelamento/panic entre `BEGIN` e `COMMIT` devolve ao pool uma conexão em transação `[P]`. Panic numa task do backend nunca emite `Event` e a UI fica em spinner eterno `[C]`. |
| AUD-25 | C | `let _ = audit::append(...)` em falha de integridade da evidência ignora erro de auditoria (AGENTS §6: não ocultar falhas relevantes). |
| AUD-27 | C | NIST: catálogo CSF 2.0 não embutido nem importável offline; o analista digita `subcategory_id`/rótulo. Tiers "contextualizados" só como rótulo. |
| AUD-28 | C | Exportação cobre só IOC + recomendações, sem manifest, hash, versão ou timestamp. Fora: NIST gap, ATT&CK, caminho crítico, impacto, YARA, timeline. |
| AUD-29 | P | `DFIR_MAX_UPLOAD_BYTES` vai até 4 GiB, mas `MAX_SCAN_BYTES` do YARA é 256 MiB: comportamento acima do limite não definido (truncar em silêncio violaria RF-006). `RLIMIT_AS` de 1 GiB × mmap + wasmtime do yara-x: validar em runtime. |
| AUD-32 | C | **Bug de UI:** executar YARA depois de EVTX sobrescreve o painel "Timeline" da evidência com um resultado vazio (`on_analyze_completed` sempre dispara `JobResults` do job recém-criado, seja qual for o `kind`). |

### P3 — Baixo / higiene

AUD-23: o item `YARA_MATCH.evidence_id` carrega o `input_reference` (`evidence/…`), não o id da evidência (nome enganoso). AUD-26: re-hash completo (BLAKE3 + SHA-256) a cada análise, antes do semáforo e sem limite de concorrência. AUD-30 (menores): regra `-e ` ruidosa; `ENGINE_COMPATIBILITY` sincronizado à mão; `Level` numérico sem mapa; formato só por extensão, sem magic bytes; `JSON_DEPTH` 256 acima do limite padrão de 128 do `serde_json` (`[P]` o app cai em `BAD_JSON`, não em `JSON_DEPTH_LIMIT`); `PARSER_RECORD_ERROR_LIMIT` emitido duas vezes; `rfd::pick_file` bloqueia a thread da UI; `request_repaint_after(150ms)` constante; `main` devolve `Debug` do erro (invisível em GUI sem console); `list_evidence LIMIT 100` sem paginação.

### Documentação e governança

| ID | Achado |
|---|---|
| DOC-01 | **RTM com evidência STALE não invalidada** (AGENTS §9.8): linhas `ACCEPTED` (evidência+hash+custódia, EVTX+timeline, descarte) ainda apontam para `src/web/mod.rs`, `frontend/`, `DELETE /api/evidence`. O fluxo de importação mudou de comportamento. A RTM diz "RF-011 cobre descarte local", mas na spec §39 RF-011 é **ATT&CK**. RF-015 e RF-034 não têm ticket; o backlog cita RF-031, que não existe. |
| DOC-02 | Versão desatualizada: `0.33.3` em TOOLCHAINS, SUPPLY_CHAIN, ADR-0017, LICENSES_THIRD_PARTY; NATIVE_UI_ARCHITECTURE descreve `App::update()` (o código usa `App::ui`); README/PROMPT/deny citam `evtx 0.12.2` (lock: 0.12.3). |
| DOC-03 | ADR-0017/AGENTS chamam o pedido do operador de "requisito explícito da demanda" (nível 1). Reclassificar como decisão do operador e emitir spec v6.4.6/6.5.0 consolidada: o corpo da v6.4.5 ainda tem Axum/Askama/Podman/Bun, corrigidos só por notas de topo. |
| DOC-04 | FASE0_STATUS mantém narrativa longa de frontend/rotas removidas; DEFINITION_OF_READY ainda marca "frontend com Bun"; status dos tickets mistura `ready-for-agent`/`PLANNED`; README repete "processo parser separado → processo separado". |
| DOC-05 | ~1,1 MB de duplicatas byte-idênticas: cada spec arquivada existe em duas cópias (`PROJECT_SPEC_*` e `nist-senai-*`), mais a cópia raiz da spec corrente. |
| DOC-06 | Sem CI; `tests/*/fase0.md` são descrições, não testes; `tests/performance` vazio; sem `supply-chain/` do `cargo vet`. |

---

## 3. Backlog

**Convenções:** novos tickets continuam a numeração `F1-xx` (IDs nunca reutilizados, AGENTS §10). Esforço: S ≤ 1 dia, M ≤ 3 dias, L > 3 dias. Todo ticket só vira `VERIFIED` com evidência executada (AGENTS §12) e atualiza README/STATUS/RTM/CHANGELOG/ADR no mesmo passo (AGENTS §9).

### Onda 0 — Destravar (fazer primeiro)

**F1-13 — Regenerar `Cargo.lock` e criar gate de coerência manifesto × lock** · S · AUD-01, DOC-02
- `cargo update -p eframe -p egui -p egui_extras` (ou build sem `--locked` uma vez) com o toolchain 1.97.1; commitar o lock.
- `validate_fase0.py` passa a comparar requisito do manifesto com versão do lock (falha se divergir).
- Docs corrigidas para 0.36.2 / evtx 0.12.3.
- Aceite: `just check` e `just test` verdes com `--locked` em Windows e Linux.

**F1-14 — Baseline de verificação e CI mínimo** · M · DOC-06
- Rodar `fmt/check/clippy/nextest` no toolchain pinado (Windows + Linux) e registrar `EVD-*`.
- Abrir defeito para tudo que falhar; promover F1-01..05 a `VERIFIED` só com evidência.
- CI (qualquer provedor) executando `just verify` + `just supply-chain`.
- Aceite: relatório de gates com versões efetivas de rustc/cargo; nenhum "PASS" sem execução.

**F1-15 — Supply chain da árvore de UI** · M · AUD-13
- `cargo deny --workspace check`, `cargo audit`, `cargo vet init` sobre eframe 0.36 + rfd.
- Ajustar `deny.toml` com exceções individuais e justificadas (ex.: licenças de fontes embutidas), sem ignore genérico.
- Avaliar desligar features padrão do eframe não usadas (`datepicker` não tem uso no código).
- Atualizar `SUPPLY_CHAIN.md` e `LICENSES_THIRD_PARTY.md` com a árvore real.

### Onda 1 — Integridade e corretude (P1)

**F1-16 — Fixtures EVTX e testes de integração do pipeline** · M · AUD-31, AUD-05
- Fixtures EVTX sanitizados (pequeno, médio, corrompido, com SystemTime ausente) com proveniência documentada.
- Teste positivo: parse → eventos normalizados → timeline ordenada.
- Testes de `service` com SQLite temporário: import, analyze (engine real), integridade adulterada, destroy.
- Aceite: caso de EVTX grande documenta o limite real de saída e define o teto adequado (ver F1-18).

**F1-17 — Erros de parsing persistidos e visíveis (RF-006)** · S · AUD-04
- Gravar `PARSER_RECORD_ERROR` em `results.jsonl`/tabela, com contagem e amostra, e exibir na tela de resultados.
- Job com erros parciais termina `COMPLETED_WITH_ERRORS` (ou flag equivalente), nunca `COMPLETED` limpo.
- Teste com EVTX parcialmente corrompido.

**F1-18 — Timeline persistida, paginada e filtrável** · L · AUD-05, AUD-06, AUD-18
- Persistir eventos normalizados (tabela indexada por tempo/evidência/Event ID/host/usuário) ou índice sobre o JSONL; leitura por cursor.
- Ordenar **no banco**, nunca depois de truncar. Eliminar `MAX_TIMELINE_EVENTS=1000` como corte de dados.
- Streaming no engine (limite de saída por lote, não por job) ou teto configurável com falha explícita.
- Filtros: intervalo, tipo (Event ID/canal), host, usuário, origem/evidência (spec §30.3).
- Aceite: EVTX com >100 mil eventos exibe o primeiro evento cronológico correto.

**F1-19 — Resultados analíticos: vigentes sem janela, YARA idempotente, vínculo evidência↔incidente** · M · AUD-07, 08, 10, 23
- Query SQL de "vigentes" (`NOT EXISTS newer.supersedes = ar.id`, filtro por incidente e tipo, sem `LIMIT 500`), usada por UI e export.
- YARA: `supersedes` pelo resultado vigente da mesma `correlation_key`; reexecução idempotente (spec §26: chave de idempotência por sha256 + ruleset + engine + schema).
- Importação aceita `incident_id`; GUI com seletor de incidente e seletor de evidência nos formulários (fim do UUID digitado).
- Aceite: 600+ resultados no incidente aparecem completos; dois scans seguidos não duplicam.

**F1-20 — YARA: cobertura de EVTX e limites** · S · AUD-09, AUD-29
- `wide ascii` nas strings de PowerShell/Mimikatz; teste com fixture UTF-16 e com EVTX real contendo o termo.
- Política explícita para arquivo > `MAX_SCAN_BYTES` (falhar ou registrar truncamento; nunca silencioso). Validar `RLIMIT_AS` × mmap/wasmtime no Linux.
- Revisar `-e ` (ruído).

**F1-21 — Destruição de evidência em duas fases** · S · AUD-02, AUD-21
- Fase 1: transação com `DESTROY_REQUESTED` (audit + custody) e arquivo movido para quarentena. Fase 2: remover físico e registrar `DESTROYED`.
- Recuperação na partida para destruição pela metade.
- Testes com falha injetada entre as fases.

**F1-22 — Isolamento do engine (RF-033/037)** · M · AUD-15
- `env_clear()` e allow-list mínima de variáveis; engine passa a **usar** `DFIR_ENGINE_INPUT_FILE` (ou staging/hardlink por job) e rejeitar qualquer outro arquivo.
- `kill_on_drop(true)`; no Windows, Job Object com limite de memória e kill-on-close.
- Testes: engine não lê outro arquivo da raiz; ambiente não vaza.

**F1-23 — Erros acionáveis** · S · AUD-14, AUD-25
- Mensagem com: o que aconteceu, impacto, ação sugerida e código técnico (`error_code` do job). Sem vazar caminhos.
- Remover `let _ =` de auditoria em caminho de falha de integridade.

**F1-24 — Custódia verificável e visível** · M · AUD-16, AUD-17
- Âncora de head/contagem por evidência; `audit-verify` roda na partida e a GUI mostra o resultado.
- Telas: custódia por evidência, auditoria filtrável, histórico de jobs (incluindo falhos), botão "Verificar integridade".
- Identidade do operador: usuário/host do SO registrados no `actor`.
- Avaliar assinatura opcional (spec §19.4/§31) para o export final.

### F1-10 refinado — Robustez operacional (já existe no backlog, agora com escopo real)

AUD-03, AUD-11, AUD-24, AUD-26 · L
- Migration nova (tabela reconstruída, pois o CHECK do SQLite não se altera) com estados `PENDING, PROCESSING, COMPLETED, FAILED, TIMEOUT, CANCELLED, CANCELLING, RECOVERY_PENDING, PERMANENTLY_FAILED`, `attempt_count`, `next_attempt_at`, `idempotency_key`.
- Worker único com fila real (spec §26): `analyze` apenas enfileira; GUI acompanha por polling/eventos.
- Recovery na partida: `PROCESSING` → `RECOVERY_PENDING` + auditoria + reenfileirar ou `PERMANENTLY_FAILED`.
- Graceful shutdown: fechar janela/SIGTERM/SIGINT → drenar ou cancelar com prazo; nunca marcar parcial como `COMPLETED`.
- Retry com backoff (1s→60s) só para falha transitória; DLQ visível e reprocessável na UI.
- Checagem de espaço livre (<10% bloqueia import, alerta antes, auditado, nunca apaga evidência).
- `sqlx::Transaction` com rollback no drop no lugar de `BEGIN` manual; guard de panic no backend (todo `Command` gera `Event`).
- Re-hash com semáforo próprio.

### Onda 2 — Funcionalidades ausentes da demanda

**F1-25 (novo) — Modelo de Finding/Vulnerabilidade e conteúdo dos Entregáveis 1, 2 e 5** · M · AUD-12 · *pré-requisito de F1-07*
- Novos `result_type`: `FINDING` (condição explorada, evidência, impacto, severidade, confiança), `ROOT_CAUSE` (nível de evidência: causa-raiz/provável/fatores) e `DATA_PROTECTION_ITEM` (classe do dado, proteção em trânsito/repouso, gestão/rotação/recuperação de chaves).
- Matriz de risco derivada de severidade × confiança, sem inventar valores.
- Lacunas explícitas de evidência (RB-001) como entidade, não texto solto.

**F1-26 (novo) — Extração de IOC a partir de eventos** · M · AUD-20
- Extrair IP, domínio, URL, e-mail, hashes, caminho e hostname dos eventos normalizados, com referência ao evento (`raw_reference`) e primeira/última ocorrência.
- Regra: o analista confirma (nada vira IOC "confirmado" sem revisão); duplicatas acumulam sightings (reuso do modelo atual).

**F1-27 (novo) — UX do analista** · M · AUD-18, AUD-19, AUD-32
- Progresso: etapa + percentual quando confiável, estado indeterminado caso contrário; cancelamento de job.
- Painel "formatos suportados × apenas preservados" por evidência.
- Corrigir AUD-32 (YARA não sobrescreve timeline); filtro por evidência; breadcrumbs; caminho crítico como grafo (SVG/`egui::Painter`).

**F1-06 — Hardening/Ansible** · M · RF-012 · depende de F1-25
- Recomendação de hardening com rationale, referência CIS, pré/pós-condições e dry-run.
- Gerar Ansible só para recomendações em allow-list; templates fixos, dados de evidência nunca controlam estrutura. Caso não suportado vira texto explícito.
- Artefato com versão, origem e hash.

**F1-07 — Entregáveis e relatórios** · L · RF-013/040/046 · depende de F1-06, F1-25
- Decompor em: 7a Relatório DFIR · 7b Vulnerabilidades+ATT&CK · 7c NIST+Recomendações · 7d Hardening · 7e Criptografia/Proteção de dados.
- Perfis executivo e técnico sobre o mesmo dataset; preview sem assinar; render determinístico (Typst sem shell, com timeout e diretório controlado; ou Markdown/JSON no MVP se Typst não for adotado por ADR).
- Testes de injeção/traversal em template; falha de render = falha de job.

**F1-08 — `.dfirpkg`** · L · RF-014/025 · depende de F1-07
- Manifest com versão, conteúdo e hashes; validação completa **antes** de persistir; rejeitar truncado/alterado/zip-slip/bomba de compressão (spec §20.2); round-trip preservando proveniência.

**F1-09 — Atualização offline** · L · RF-026/030/032/047 · depende de F1-08
- Bundle assinado/verificado, preflight, backup antes de migration, health/smoke antes de ativar, rollback documentado.
- Documento de instalação de referência com proteção do volume (BitLocker/LUKS) — cobre RF-030.

**F1-11 — Demo, UX final e benchmark** · M · RF-016/017/045/046
- Dataset BEC determinístico e sanitizado (spec §42); `just perf` com metadados reprodutíveis; testes de XSS/injection/traversal executáveis (hoje só descritos em `.md`).

**F1-12 — Release candidate** · M · RF-* restantes
- Reconciliar RTM com **todos** os 43 RFs MVP (implementação, teste, evidência); zerar `PENDING/BLOCKED/STALE`; demo ponta a ponta offline.

### Onda 3 — Documentação e governança

**F1-28 — Higiene documental e RTM** · S · DOC-01..05
- RTM: invalidar para `STALE` as linhas que citam código removido e revalidar após F1-14; corrigir RF-011; mapear RF-015/RF-034; remover RF-031 do backlog.
- Reclassificar ADR-0017 (e a precedência no AGENTS) como decisão do operador; publicar spec v6.4.6/6.5.0 consolidada, sem Axum/Podman/Bun no corpo normativo.
- Enxugar FASE0_STATUS; corrigir DEFINITION_OF_READY; padronizar estados de ticket; remover as ~1,1 MB de duplicatas e trocar a cópia raiz por verificação automática.

### Ordem sugerida e dependências

```
F1-13 → F1-14 → F1-15
          │
          ├─ F1-16 → F1-17 → F1-18
          ├─ F1-19, F1-20, F1-21, F1-22, F1-23, F1-24   (paralelizáveis)
          └─ F1-10 refinado (depende de F1-16 e F1-19)
                    │
                    └─ F1-25 → F1-26, F1-27 → F1-06 → F1-07 → F1-08 → F1-09 → F1-11 → F1-12
F1-28 em paralelo, contínuo
```

Caminho mínimo para um MVP defensável: **Onda 0 → F1-16/17/18/19/20/21 → F1-10 → F1-25 → F1-06 → F1-07**. Sem F1-07 (e F1-25 antes dele), o projeto não entrega nenhum dos seis requisitos da demanda como documento.

---

## 4. Verificações pendentes para quem tem o toolchain

1. Regenerar o lock (F1-13) e rodar `just verify` — a primeira compilação com egui 0.36.2 nunca foi executada.
2. Abrir um EVTX real de >10 MB para confirmar AUD-05 (OUTPUT_LIMIT) e AUD-09 (YARA sem `wide`).
3. `cargo deny --workspace check` para confirmar o achado de licenças de AUD-13.
4. Fechar a janela durante uma análise e reabrir, para reproduzir AUD-03 (job preso e evidência não destruível).
5. Rodar `just runtime-test` no Windows (o único ambiente onde `reject_symlink` e o engine ainda não foram exercitados, segundo FASE1_STATUS).


---

# Parte 2 — Aderência à demanda original e poda do backlog

# Adendo — aderência à demanda original (SENAI 12412) e poda do backlog

Data: 2026-10-02. Complementa `AUDITORIA_E_BACKLOG_2026-10-01.md`. Onde este adendo contradiz aquele relatório, **vale este**.
Base: texto original da demanda (transcrito em `DEMANDA_ORIGINAL_SENAI_12412.md`) × spec v6.4.5 × código.
Certeza: `[C]` confirmado no código/documento; `[P]` provável, exige teste.

---

## 1. A arquitetura atende a demanda?

**Parcialmente. Ela atende bem a primeira metade da demanda (preservar evidência com integridade) e quase nada da segunda (produzir o plano de fortalecimento).**

A demanda tem dois blocos: (A) investigar o incidente e (B) transformar a investigação em plano de melhoria. Os 6 entregáveis são documentos de B, exceto o primeiro. O projeto investiu pesado em A (custódia, cadeia de auditoria, engine isolado, limites de recurso) e deixou B sem modelo de dados.

### 1.1 Cinco desvios de arquitetura (não são bugs, são decisões de desenho)

1. **O "evento normalizado" não é normalizado `[C]`.** `NormalizedEvent` tem `timestamp` e um `data: Value` com o JSON cru do EVTX. Não há `event_id`, canal, host, usuário, IP, processo, tipo de logon. Consequência em cadeia: o filtro de timeline só consegue texto livre; não dá para detectar nada por regra; a extração de IOC fica manual; e **nenhuma outra fonte de log cabe nesse formato**. É a decisão de desenho mais cara do projeto.
2. **Só uma das seis fontes da demanda é analisada `[C]`.** EVTX é analisado. Tráfego de rede, SIEM, dispositivos de segurança e autenticação não-Windows são só preservados (a spec §4.3 aceita isso, mas manda a UI **declarar** isso, e ela não declara).
3. **O cenário BEC não é demonstrável com o que existe `[C]`.** A spec elege BEC como cenário principal do demo (§2.3): login anômalo, regra de encaminhamento, origem geográfica, acesso a dados financeiros. Essas evidências vivem em logs de e-mail/identidade (JSON/CSV/EML), não em EVTX. O dataset de demo só poderia mostrar EVTX + YARA, ou ser preenchido à mão.
4. **"Identificar" é 100% manual `[C]`.** A demanda pede identificar vetor inicial, movimentação lateral, escalonamento de privilégios e exfiltração. Não existe nenhuma regra de detecção sobre eventos (zero ocorrência de 4624/4625/4672/4688/7045/1102 no código). ATT&CK, caminho crítico e impacto dependem de o analista digitar tudo.
5. **O lado B não tem entidade `[C]`.** Não há Finding/Vulnerabilidade, causa-raiz nem item de plano. Faltam, sem nenhuma estrutura: revisão de rede/dispositivos, autenticação e acesso, monitoramento, criptografia, gestão de chaves, políticas e procedimentos de resposta a incidentes. Hardening (F1-06) está planejado só como Ansible, que a demanda não pede (pede **plano**).

### 1.2 Matriz demanda → projeto

| Item da demanda | Cobertura | Obs. |
|---|---|---|
| Coleta/preservação de evidência | ✔ forte | Coleta em si é externa (aceitável); proveniência (origem, área, método, autorização) existe no schema e **nunca é preenchida** `[C]`; `source` tem default stale `LOCAL_BROWSER` |
| Análise: logs de servidores / endpoints / autenticação | ◐ só EVTX | Sem detecção; sem outros formatos |
| Análise: tráfego de rede / SIEM / dispositivos de segurança | ✖ só preservado | Sem declaração na UI |
| Vetor inicial, vulnerabilidades, lateral, privilégios, exfiltração, impacto | ◐ manual | Sem detecção automática; sem entidade Finding |
| Linha do tempo | ◐ | Só EVTX; bugs AUD-05/06 |
| NIST CSF 2.0 — as 6 funções (GV, ID, PR, DE, RS, RC) | ◐ | Validação das 6 funções OK `[C]`. **Sem visão agregada por Function**: o entregável de maturidade só teria uma lista de subcategorias `[C]` |
| Plano: hardening de SO/servidores | ✖ | F1-06 não iniciado |
| Plano: redes/dispositivos; autenticação/acesso; monitoramento; IR | ✖ | Sem ticket |
| Plano: criptografia; gestão de chaves | ✖ | Sem ticket |
| 6 entregáveis | ✖ | Nenhum gerado |

### 1.3 As 6 restrições da demanda

| # | Restrição | Tratamento hoje | Veredito |
|---|---|---|---|
| 1 | Retenção insuficiente de logs | Spec: "declarar lacunas" | Sem entidade de lacuna/cobertura temporal por fonte `[C]` |
| 2 | Dependência de outras áreas | Spec: "fluxos assíncronos" | Sem registro de quem forneceu o quê; colunas de proveniência vazias |
| 3 | Indisponibilidade durante testes | Análise offline | OK; hardening Ansible *aumentaria* esse risco |
| 4 | **Orçamento limita implementar tudo de imediato** | Spec (RB-004) reinterpretou como "usar open source" | **Interpretação trocada.** A restrição é do *cliente*: as recomendações precisam de prioridade, esforço, custo e fases. Hoje só há IMMEDIATE/STRATEGIC, sem esses campos `[C]` |
| 5 | Políticas internas e requisitos regulatórios | Spec: "trilhas de auditoria" | Sem campo de referência regulatória/política nas recomendações |
| 6 | Confidencialidade e privacidade | Spec: "dados sanitizados" | Nenhuma redação/mascaramento implementado; `SANITIZED` é só um valor de enum `[C]` |

---

## 2. Triagem do backlog existente (F1-01 a F1-12)

Critério: o item ajuda a produzir um dos 6 entregáveis, ou a proteger a evidência que os sustenta? Se não, é custo sem retorno.

| Ticket | Veredito | Por quê |
|---|---|---|
| F1-01 modelo analítico | **Manter, simplificar** | Núcleo certo (proveniência + confiança), mas a cadeia `supersedes` em todo tipo é o que gerou AUD-07/08. Usar só onde há revisão real (NIST, Finding) |
| F1-02 YARA-X | **Manter congelado** | A demanda não cita YARA. Com 4 regras (uma é EICAR) é quase decorativo. Corrigir `wide`, **não expandir** |
| F1-03 NIST Current/Target/Gap | **Manter + completar** | É o entregável 3. Falta a visão por Function |
| F1-04 ATT&CK/caminho/impacto | **Manter** | Mapeia direto a lateral/privilégios/exfiltração/impacto. Grafo SVG é luxo, adiar |
| F1-05 IOC + recomendações | **Manter, trocar IOC manual por extração** | Recomendação é núcleo; ver F1-30/F1-32 |
| F1-06 Hardening + Ansible | **Reduzir** | A demanda pede *plano*. Gerar playbook executável a partir de dados de evidência é risco sem exigência. Virar item de plano (F1-32); Ansible só como anexo textual |
| F1-07 Relatórios executivo + técnico | **Reduzir** | Ver §2.1 |
| F1-08 `.dfirpkg` | **Adiar/cortar** | A demanda não pede pacote portátil. Cria uma superfície nova de entrada não confiável (zip-slip, bomba de compressão, manifest forjado), contradizendo a postura do projeto. Basta exportar entregáveis + manifest de hashes |
| F1-09 Atualização offline | **Cortar** | É engenharia de distribuição de software; zero relação com a demanda. Manter só: backup do banco antes de migration + procedimento manual documentado |
| F1-10 Robustez | **Reduzir** | Ver §2.2 |
| F1-11 UX/demo/benchmark | **Manter demo e filtros; cortar `just perf`** | Benchmark não é requisito de negócio |
| F1-12 Release candidate | **Manter enxuto** | Reconciliar RTM só dos itens que sobraram |

**Atenção de governança:** RF-025 (`.dfirpkg`) e RF-026 (atualização offline) estão como **MVP** na spec §39. Para cortar, é preciso uma ADR + bump de spec reclassificando-os para fase posterior; senão o F1-12 continua exigindo-os.

### 2.1 "Perfis" (executivo/técnico) em F1-07

Interpreto "adição de perfis" como o **perfil executivo × técnico dos relatórios**. A demanda pede um "relatório completo", nunca duas visões. Duplicar a renderização (e o teste, e o preview) para dois públicos é custo sem origem na demanda. **Troque por um relatório único com resumo executivo no topo.**

Se você quis dizer os **Perfis NIST (Current/Target)**: esses não são cortáveis. É a avaliação de maturidade que a demanda exige. O que falta neles é agregação, não mais perfis.

### 2.2 F1-10 (robustez)

- **Manter:** recovery de job preso na partida, shutdown limpo, checagem de espaço em disco.
- **Cortar:** retry com backoff, `PERMANENTLY_FAILED`, DLQ. Reprocessar um parse determinístico de arquivo corrompido não muda o resultado, e num app local monousuário falha transitória é rara. É padrão de sistema distribuído sem público aqui.

### 2.3 Autocorreção: onde eu mesmo inflei o backlog no relatório anterior

| Ticket meu | Ajuste |
|---|---|
| F1-10 "refinado" (fila com worker, 9 estados) | Superdimensionado. Vale o §2.2 |
| F1-24 (âncora de custódia, assinatura opcional) | Manter só: mostrar custódia/auditoria na GUI + rodar `audit-verify` na partida. Cortar assinatura e âncora extra |
| F1-27 (grafo SVG, breadcrumbs, cancelamento) | Manter só filtros reais e o fix do AUD-32. Resto é cosmético |
| F1-21 (destruição em duas fases + recovery) | Basta inverter a ordem: transação no banco primeiro, depois apagar o arquivo; se o apagamento falhar, registrar e permitir nova tentativa |
| F1-22 (Job Object, limite de memória no Windows) | Manter só `env_clear` + engine usar `DFIR_ENGINE_INPUT_FILE`. Job Object, adiar |
| F1-15 (`cargo vet`) | Rodar `deny` + `audit`; `vet` é overhead para projeto solo — registrar a dispensa em ADR |

---

## 3. Novos tickets orientados pela demanda

### F1-29 — Evento realmente normalizado · M · *(chave: desbloqueia F1-30, F1-31, filtros e extração de IOC)*
Estender `NormalizedEvent` com campos estruturados opcionais: `event_id`, `provider/canal`, `host`, `usuário`, `ip_origem`, `ip_destino`, `processo`, `ação`, `resultado`, `tipo_fonte`. O EVTX preenche o que existir; `data` continua guardando o original. Colunas indexadas no banco para filtro e ordenação (resolve também AUD-06).

### F1-30 — Importador genérico CSV/JSONL → eventos · M · RB-001, fontes da demanda
Mapeamento de colunas para o schema do F1-29. Cobre exportações de SIEM, logs de autenticação, firewall/IDS, Zeek/Wireshark em CSV e logs de e-mail/identidade (M365/Entra) do cenário BEC, **sem escrever um parser por produto**. É o maior ganho de cobertura de fontes por custo. Rejeitar linhas malformadas com contagem visível (RF-006).

### F1-31 — Regras de detecção sobre eventos (candidatos a achado) · M · demanda: vetor, lateral, privilégios, exfiltração
Conjunto pequeno e versionado de regras declarativas (no estilo Sigma, só o necessário): logon falho em rajada, logon tipo 3/10 de origem nova, uso de privilégio especial, criação de serviço/tarefa, criação/alteração de conta e grupo, limpeza de log, PowerShell com conteúdo codificado, regra de encaminhamento de e-mail, login de origem incomum. Cada regra gera **candidato** com ATT&CK sugerida e evento de origem; **nada vira achado sem confirmação do analista**.

### F1-32 — Item de plano (PlanItem) · M · entregáveis 3, 4, 5 + IR · *(substitui F1-06 e o plano de criptografia)*
Uma entidade ligada a Finding e a subcategoria NIST, com:
- `domínio`: `OS_HARDENING | REDE_DISPOSITIVOS | AUTENTICACAO_ACESSO | MONITORAMENTO | CRIPTO_REPOUSO | CRIPTO_TRANSITO | GESTAO_CHAVES | POLITICA_IR`
- `prioridade`, `esforço`, `classe de custo`, `fase` (resolve a restrição 4), `dependências`
- `referência` (CIS/NIST/regulatória/política interna — restrição 5)
- `pré-condição`, `pós-condição`, `como verificar`
- para `CRIPTO_*`: classe do dado, local, mecanismo, rotação, recuperação de chave.

Esta única entidade alimenta Recomendações, Hardening, Criptografia/Proteção de dados e as políticas de IR. Ansible fica fora do MVP.

### F1-33 — Visão NIST por Function · S · entregável 3
Contagem por status e Tier descritivo para GV/ID/PR/DE/RS/RC, derivados do Gap. Sem "score oficial" (a spec §3 já veta; manter o veto).

### F1-34 — Proveniência, cobertura e lacunas · S/M · restrições 1 e 2
- Preencher na importação: origem (sistema/área), método e ferramenta de coleta, data de coleta, referência de autorização. Corrigir o default `LOCAL_BROWSER`.
- Tela "formatos analisados × apenas preservados" por evidência.
- Registro de **lacuna**: fonte esperada, período sem dados, motivo (retenção, não fornecido). Entra nos relatórios.

### F1-35 — Redação/PII na exportação · S · restrição 6
Flag "contém dado pessoal" por evidência; export com opção de mascarar IP/usuário/e-mail; aviso antes de exportar sem máscara.

---

## 4. Backlog podado, em ordem

```
Onda 0  F1-13 lock · F1-14 baseline · F1-15 deny/audit (sem vet)
Onda 1  F1-16 fixtures · F1-17 erros de parse · F1-29 evento normalizado
        F1-18 timeline persistida/filtrável · F1-19 vigentes/YARA idempotente · F1-20 YARA wide
        F1-21' destruição (ordem) · F1-22' env_clear + input_file · F1-23 erros · F1-10' recovery/shutdown/disco
Onda 2  F1-30 import CSV/JSONL · F1-31 detecção · F1-25 Finding · F1-33 NIST por Function · F1-34 proveniência/lacunas
        F1-32 PlanItem
Onda 3  F1-07' relatório único (6 conteúdos) · F1-35 redação · F1-11' demo BEC + filtros
        F1-24' custódia visível
Onda 4  F1-12' release candidate · F1-28 higiene documental
Cortados/adiados (precisam de ADR + spec):  F1-08 .dfirpkg · F1-09 atualização · retry/DLQ · benchmark · perfis exec/téc · Ansible · grafo SVG
```

**Caminho mínimo defensável:** Onda 0 → F1-29 → F1-18 → F1-30 → F1-31 → F1-25 → F1-32 → F1-07'. Esse caminho produz, ponta a ponta, os 6 conteúdos da demanda a partir de evidência. Todo o resto é suporte.

---

## 5. Higiene da fonte de verdade

1. **A demanda original não está no repositório.** A spec a chama de "fornecida pelo usuário nesta conversa" e o `DOCUMENT_AUTHORITY.md` a coloca como fonte nº 1 de requisitos. Uma fonte nº 1 que não está no repo não é auditável nem rastreável pela RTM.
2. Ações: salvar `docs/DEMANDA_ORIGINAL_SENAI_12412.md` (anexo) e o PDF em `docs/source/` com SHA-256; criar `docs/rtm_demanda.md` mapeando cada item da seção 1.2 acima a RF/ticket/evidência; apontar o `DOCUMENT_AUTHORITY` para esses arquivos.
3. Corrigir na spec: RB-004 (interpretação trocada, §1.3 acima) e a qualificação do BEC (é uma *ideia* registrada sob a demanda, códigos 310326/310327, e não a demanda em si; a demanda descreve "incidente cibernético ocorrido", genérico).
