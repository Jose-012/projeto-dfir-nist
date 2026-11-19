<!-- doc-type: gate | status: current -->
# Release Gate - ferramenta forense local

O release é fechado somente com evidência executada.

> **Evidências (Gap origem: AUS-03).** Os `EVD-*` e os `WVR-*` deste release vivem em [`docs/EVIDENCE_REGISTER.md`](EVIDENCE_REGISTER.md). Toda declaração de `PASS` abaixo cita um `EVD-*` daquele registro; sem `EVD-*`, o gate permanece `PENDING` ou `PENDING-HUMAN`. Os gates que dependem de execução do operador estão marcados `[PENDING-HUMAN → EVIDENCE_REGISTER]`.
>
> **Atualização 2026-11-14.** Execuções reais nesta árvore, em Linux x86_64 e rustc 1.99.1: Gate Rust e supply chain `PASS` (`EVD-001`/`EVD-003`); e2e 110/110 com engine real (`EVD-002`). Windows, Linux aarch64 e incidente real continuam sem execução.
>
> **Atualização 2026-11-14 (tarde/noite).** Após upgrade `eframe`/`egui`/`egui_extras`/`egui_kittest` 0.37.0 (sem `accesskit`; cadeia zbus/AT-SPI fora do grafo) + `cargo update` geral: Gate Rust `PASS` (`EVD-005`, nextest 410/410), e2e `PASS` 110/110 (`EVD-006`), supply chain `PASS` (`EVD-007`), doc-gate `PASS` (`EVD-008`), janela real da GUI aberta no display do host com adaptador wgpu registrado no log (`EVD-009`, cobre GUI-01/GFX-01). Itens p2–p6 do plano do operador resolvidos: waivers formais `WVR-001` (Windows), `WVR-002` (aarch64), `WVR-003` (CI automático) e `WVR-004` (SEC-02/RUSTSEC-2026-0459, revisão até 2026-12-31), todos `approved` pelo operador em 2026-11-14T22:00Z e registrados em `docs/EVIDENCE_REGISTER.md`; `docs/archive/DEFINITION_OF_READY_fase0_fase1.md` marca todos os itens com `EVD-*`/`WVR-*`; políticas operacionais de `backups/`, distribuição de `.evtx` e F1-16 documentadas em `docs/OPERATION.md` §3.1.1 e §5.

> **Atualização 2026-11-15.** Pistas C e D do plano remanescente histórico (`docs/archive/FASE1_BACKLOG_plano_2026-10_2026-11.md` §7) concluídas: Gate Rust `PASS` (`EVD-010`: fmt ok, clippy `-D warnings` limpo, nextest 424/424) e e2e `PASS` 110/110 sem arquivo externo sobre a fixture versionada `tests/fixtures/senai-demo.evtx` (`EVD-011`). Tickets F1-16, F1-19, F1-20, F1-21, F1-23, F1-24 e F1-33 promovidos a `VERIFIED` em `docs/rtm.md`/`docs/FASE1_STATUS.md`, cada um com teste próprio.
>
> **Plano remanescente (2026-11-18).** As pendências atuais estão em [`docs/FASE1_BACKLOG.md`](FASE1_BACKLOG.md): verificação externa, decisão de CI, revisões de supply chain e fechamento do RC. O plano detalhado anterior foi preservado em `docs/archive/FASE1_BACKLOG_plano_2026-10_2026-11.md`.
>
> **Atualização 2026-11-18.** Pin subiu para **rustc/cargo 1.100.0** (stable de 2026-11-12) e todos os gates locais rodaram nesse toolchain (`EVD-012`): fmt ok, check `--workspace --locked --all-targets` ok, clippy `-D warnings` limpo, nextest **426/426** (1 ignorado) incluindo os testes novos de RF-003 e RF-027, e2e **110/110** com engine real e sandbox `required`, supply chain verde (deny+audit com 2 exceções justificadas: WVR-004/RUSTSEC-2026-0459 e RUSTSEC-2026-0543 chrono informativo), doc-gate PASS. Pistas A (Windows/aarch64/incidente real), B (CI) e E (revisão recorrente) continuam abertas com waivers até 2026-12-31; pista F segue sem RF MVP pendente sem waiver.

> **Atualização 2026-11-18 (fechamento da rodada de engenharia).** No commit `f8dd8c6`, `EVD-013`: doctor 0 erros/1 aviso de Python; `just verify` com nextest 432 passed/1 skipped, doc-tests, fmt/check/clippy; supply chain verde com as exceções documentadas; doc-gate PASS; e2e 110/110. Windows, aarch64, incidente real e CI continuam não executados/não implementados, sob os waivers e pendências já registrados.
>
> **Atualização 2026-11-18 (ajuste visual da GUI).** No commit `26cc8b3`, `EVD-014`: fmt/check/clippy/doc-tests verdes, nextest 435 passed/1 skipped, supply chain verde, doc-gate PASS e e2e 110/110. A verificação cobre os três novos testes de glifos e layout; nenhuma janela real ou verificação externa nova foi declarada.
>
> **Atualização 2026-11-18 (cobertura de glifos da navegação).** No commit `3ff341e`, `EVD-015`: o caminho real da navegação passa no teste estrito; mutação temporária com 📁 falhou por `No glyph` e a restauração passou. `just verify` terminou com 436 passed/1 skipped e doc-gate PASS. Supply chain e e2e não foram repetidos; `EVD-014` segue como evidência desses gates.
>
> **Atualização 2026-11-18 (largura mínima das abas).** No commit `536f009`, `EVD-016`: o teste das abas agora usa os 780 px disponíveis na janela mínima. A mutação sem wrapping falhou com `Plano` até x=819,3 e a restauração passou. `just verify` terminou com 436 passed/1 skipped e doc-gate PASS. Supply chain/e2e seguem em `EVD-014`.
>
> **Atualização 2026-11-18 (ações da tabela de evidências).** No commit `791ea58`, `EVD-017`: o novo teste verifica 48 px e não sobreposição a 780 px; a 600 px, dois eventos `MouseWheel` revelam `Destruir` e o clique atualiza `pending_destroy`. A mutação sem scroll horizontal falhou e a restauração passou. `just verify` terminou com 437 passed/1 skipped e doc-gate PASS. Supply chain/e2e seguem em `EVD-014`.
>
> **Atualização 2026-11-18 (scroll da grade NIST).** No commit `8d80b87`, `EVD-018`: `Gap baixo` começa fora da viewport de 600 px e fica visível após `MouseWheel`; a mutação vertical não move o conteúdo e falha. `just verify` terminou com 438 passed/1 skipped e doc-gate PASS. Supply chain/e2e seguem em `EVD-014`; nenhuma janela real foi aberta nesta etapa.
>
> **Atualização 2026-11-18 (janela real isolada e gates finais da GUI).** No commit `dcae49a`, `EVD-019`: a janela real permaneceu aberta até o timeout de 25 s com `DFIR_DATA_ROOT` isolado. O log registrou o adaptador Intel GL e não contém avisos de fonte/glyph/ScrollArea ID nem nomes de fixture. Supply chain verde, e2e 110/110 e doc-gate PASS. Não se declara navegação manual ou verificação externa nova.
>
> **Atualização 2026-11-18 (integridade dos candidatos de detecção).** No commit `1d39401`, `EVD-020`: antes da correção, duas reproduções confirmavam candidatos com `event_refs` ou IOCs JSON inválidos; IOCs inválidos retornavam `ioc_ids: []`. Agora listagem e confirmação propagam erro específico e a confirmação valida ambos antes de qualquer escrita analítica. `just verify` terminou com 440 passed/1 skipped, e2e 110/110, supply chain e doc-gate verdes. Novo smoke real de 25 s terminou em 124 esperado com log isolado limpo. Não se declara reparo de dados legados, navegação manual ou verificação externa nova.
>
> **Atualização 2026-11-18 (integridade dos warnings de análise).** No commit `f915d11`, `EVD-021`: as reproduções retornavam `warning_count: 1` e `warnings: []` para JSON inválido e para contagem divergente. Agora a leitura rejeita JSON inválido, contagem negativa/divergente, e a escrita não tem fallback vazio. `just verify` terminou com 442 passed/1 skipped, e2e 110/110 com os dois avisos do EVTX truncado persistidos, e doc-gate PASS. Supply chain não foi repetido e permanece em `EVD-020`; nenhuma janela real nova ou reparo legado foi declarado.
>
> **Atualização 2026-11-18 (integridade das views de custódia/auditoria).** No commit `179183a`, `EVD-022`: as reproduções retornavam `metadata: Null` em `list_custody` e `list_events` para JSON inválido. Agora ambas propagam `Protocol` específico, enquanto `verify`/`verify_custody` continuam `false`. `just verify` terminou com 444 passed/1 skipped, e2e 110/110 com a cadeia válida íntegra, e doc-gate PASS. Supply chain não foi repetido e permanece em `EVD-020`; nenhuma janela real nova, reparo legado ou incidente real foi declarado.
>
> **Atualização 2026-11-18 (integridade de payload/provenance analíticos).** No commit `5fade47`, `EVD-023`: as reproduções retornavam `payload: Null` por evidência e `provenance: Null` por incidente. `AnalyticalResultResponse` agora usa `TryFrom` e ambas as coleções propagam erro específico. `just verify` terminou com 446 passed/1 skipped, e2e 110/110 com todos os resultados/documentos válidos, e doc-gate PASS. Supply chain não foi repetido e permanece em `EVD-020`; nenhuma janela real nova ou reparo legado foi declarado.
>
> **Atualização 2026-11-18 (Gap Analysis falha antes de escrever sobre dado inválido).** No commit `367c9d5`, `EVD-024`: um `NIST_GAP` inválido retornava um novo ID de fechamento. Agora o payload falha antes de `BEGIN IMMEDIATE`, sem nova linha ou superseding. `just verify` terminou com 447 passed/1 skipped, e2e 110/110 com as dez lacunas válidas, e doc-gate PASS. Supply chain não foi repetido e permanece em `EVD-020`; nenhuma janela real nova ou reparo legado foi declarado.
>
> **Atualização 2026-11-18 (JSONL inválido preserva o índice anterior).** No commit `a2a619b`, `EVD-025`: uma segunda linha inválida era aceita durante a reindexação com `Ok((2, 0))`. Agora JSON inválido e profundidade excessiva falham com a linha e acionam rollback, preservando eventos e índice. O novo teste cobre os dois casos; 29 testes existentes de timeline, `just verify` com 448 passed/1 skipped, e2e 110/110 e doc-gate passaram. Supply chain não foi repetido e permanece em `EVD-020`; nenhuma janela real nova ou reparo legado foi declarado.
>
> **Atualização 2026-11-18 (RESULT persistido exige `items`).** No commit `9cecefb`, `EVD-026`: resultado sem array `items` retornava `Ok([])` na leitura analítica e `Ok((2, 0))` na reindexação. Agora campo ausente/tipo errado falha por linha nos dois leitores, rollback preserva o índice e `items: []` continua válido. Dois testes novos, `just verify` com 450 passed/1 skipped, e2e 110/110 e doc-gate passaram. Supply chain não foi repetido e permanece em `EVD-020`; nenhuma janela real nova ou reparo legado foi declarado.
>
> **Atualização 2026-11-18 (agregado IOC inválido não gera nova versão).** No commit `128af44`, `EVD-027`: um IOC com count 7 e `sightings={}` foi substituído por uma versão com count 8 e apenas um sighting. Agora array/count inválido, inconsistência e overflow falham sem escrever, e o legado sem contador continua válido. Um teste com quatro casos, seis testes de agregação, `just verify` com 451 passed/1 skipped, e2e 110/110 e doc-gate passaram. Supply chain não foi repetido e permanece em `EVD-020`; nenhuma janela real nova ou reparo legado foi declarado.
>
> **Atualização 2026-11-18 (dependência de plano inválida não permite revisão).** No commit `c9d4cab`, `EVD-028`: uma dependência string contendo A permitiu uma revisão A→B→A. Agora campo presente deve ser array de strings; tipo errado/elemento inválido falha sem revisão, campo ausente continua legado vazio e o controle de ciclo válido permanece verde. `just verify` terminou com 453 passed/1 skipped, e2e 110/110 e doc-gate PASS. Supply chain não foi repetido e permanece em `EVD-020`; nenhuma janela real nova ou reparo legado foi declarado.
>
> **Atualização 2026-11-18 (arrays de plano inválidos não chegam aos entregáveis).** No commit `c0e9413`, `EVD-029`: `finding_ids` string gerava o Entregável 4 com falsa recomendação geral. Agora `plan_view` valida os três arrays antes da coleta/renderização, preservando campos ausentes legados. `just verify` terminou com 455 passed/1 skipped, e2e 110/110 com os cinco documentos, e doc-gate PASS. Supply chain não foi repetido e permanece em `EVD-020`; nenhuma janela real nova ou reparo legado foi declarado.
>
> **Atualização 2026-11-18 (caminho crítico limitado ao incidente).** No commit `d7470e9`, `EVD-030`: A0→B0 e B0→B1 eram aceitos no contexto de A, e o grafo de A expunha uma aresta para estágio estrangeiro. A persistência agora exige que ambos os lados pertençam ao incidente solicitado; ligação válida continua verde. `just verify` terminou com 456 passed/1 skipped, e2e 110/110 e doc-gate PASS. Supply chain permanece em `EVD-020`; não houve nova janela real nem reparo de relações antigas.
>
> **Atualização 2026-11-18 (Finding inválido não desaparece da matriz).** No commit `62797f4`, `EVD-031`: SEVERE gerava Entregável 2 com um achado, zero células e CRITICAL zerado. Agora severity inválida/`NONE` e confidence ausente falham com ID/campo antes da renderização; dois controles válidos continuam verdes. `just verify` terminou com 457 passed/1 skipped, e2e 110/110 e doc-gate PASS. CERTAIN foi rejeitado pelo CHECK no setup. Supply chain permanece em `EVD-020`; sem nova janela real ou reparo legado.
>
> **Atualização 2026-11-18 (verificação ambiental final).** Sobre `7080dd5`, sem mudança de código, `EVD-032`: supply chain verde e janela real isolada aberta por 25 s até timeout 124 esperado, backend Gl/Mesa Intel, sem diagnósticos de fonte/glyph/scroll ou nomes de fixtures. Não houve navegação manual ou incidente real. `just verify`/e2e não foram repetidos e continuam em `EVD-031`; doc-gate PASS.
>
> **Atualização 2026-11-18 (resultado analítico com uma só sucessora por pai).** No commit `d5b7840`, `EVD-033`: uma segunda sucessora do mesmo pai gerava duas linhas vigentes na listagem enquanto a leitura singular escolhia a terceira. A transação agora rejeita pai já substituído antes do INSERT/auditoria; histórico sequencial e listagem continuam verdes. `just verify` terminou com 458 passed/1 skipped, e2e 110/110 e doc-gate PASS. Sem reparo de histórico ramificado; supply chain/smoke permanecem em `EVD-032`.
>
> **Atualização 2026-11-18 (uma só raiz por correlation_key).** No commit `ecc324d`, `EVD-034`: duas raízes com a mesma chave geravam duas linhas vigentes e a leitura singular escolhia a segunda. Nova raiz agora exige `supersedes` quando a chave já existe, antes do INSERT/auditoria; 18 controles centrais passaram. `just verify` terminou com 459 passed/1 skipped, e2e 110/110 e doc-gate PASS. Sem reparo de histórico antigo; supply chain/smoke permanecem em `EVD-032`.
>
> **Atualização 2026-11-18 (tipo e origem estáveis na supersessão).** No commit `e4bd37a`, `EVD-035`: revisões mudavam evidência, incidente e tipo do resultado vigente. A transação agora preserva tipo/incidente e evidência para não-IOC; IOC mantém agregação entre evidências no mesmo incidente. O gate inicial detectou essa exceção e foi ajustado antes do PASS final. `just verify` final terminou com 462 passed/1 skipped, e2e 110/110 e doc-gate PASS. Sem reparo antigo; supply chain/smoke permanecem em `EVD-032`.
>
> **Atualização 2026-11-18 (verificação ambiental da árvore final).** Sobre `3e3132c`, sem mudança de código, `EVD-036`: supply chain verde e janela real isolada aberta por 25 s até timeout 124 esperado, backend Gl/Mesa Intel, sem diagnósticos de fonte/glyph/scroll ou nomes de fixtures. Não houve navegação manual ou incidente real. `just verify`/e2e não foram repetidos e continuam em `EVD-035`; doc-gate PASS.

> **Atualização 2026-11-19 (reavaliação e fluxos da GUI).** ADR-0029 mantém `eframe/egui` após auditoria prática. `EVD-037`: suíte GUI 115/115, `just verify` com 509 passed/1 skipped, doctests/Clippy verdes, e2e 110/110 e doc-gate PASS. A janela X11 abriu com Gl/Mesa e log sem glyph/ID/panic; captura pós-resize é inconclusiva e não sustenta navegação manual, diálogo nativo ou AT-SPI. Supply chain e smoke anterior permanecem em `EVD-036`.

## Gate documental

Estado: `PASS` em 2026-11-19 (`EVD-047`; anteriores `EVD-004`/`EVD-008`/`EVD-012`/`EVD-013`/`EVD-014`/`EVD-019`/`EVD-020`/`EVD-021`/`EVD-022`/`EVD-023`/`EVD-024`/`EVD-025`/`EVD-026`/`EVD-027`/`EVD-028`/`EVD-029`/`EVD-030`/`EVD-031`/`EVD-032`/`EVD-033`/`EVD-034`/`EVD-035`/`EVD-036`/`EVD-037`/`EVD-038`/`EVD-039`/`EVD-040`/`EVD-041`/`EVD-042`/`EVD-043`/`EVD-044`/`EVD-045`/`EVD-046`): Fase 0 com 0 erros/0 avisos e `documentation gate: PASS`.

```bash
just validate-fase0
just doc-gate
```

`just doc-gate` executa `scripts/validate_fase0.py` e `scripts/doc_gate.py` (Gap origem: INC-02).

## Gate Rust

Estado: `PASS` em Linux x86_64 em 2026-11-19 (`EVD-047`, nextest 526 passed/1 skipped; anteriores `EVD-001`/`EVD-005`/`EVD-010`/`EVD-012`/`EVD-013`/`EVD-014`/`EVD-015`/`EVD-016`/`EVD-017`/`EVD-018`/`EVD-020`/`EVD-021`/`EVD-022`/`EVD-023`/`EVD-024`/`EVD-025`/`EVD-026`/`EVD-027`/`EVD-028`/`EVD-029`/`EVD-030`/`EVD-031`/`EVD-033`/`EVD-034`/`EVD-035`/`EVD-037`/`EVD-038`/`EVD-039`/`EVD-040`/`EVD-041`/`EVD-042`/`EVD-043`/`EVD-044`/`EVD-045`/`EVD-046`); Windows e aarch64 viraram waivers formais (`WVR-001`, `WVR-002`, válidos até 2026-12-31; ver Runtime).

```bash
rustc --version --verbose
cargo --version --verbose
cargo metadata --locked
cargo tree --locked -e features
cargo fmt --all -- --check
cargo check --workspace --locked --all-targets
cargo clippy --workspace --locked --all-targets --all-features -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
```

> **Receita atual (2026-11-19):** `just verify` usa nextest para unitários/integração e `cargo test --doc` para doc-tests. Um setup script escopado a `dfir-engine::sandbox` constrói o binário real antes desses testes, e o helper resolve `NEXTEST_BIN_EXE_dfir_engine` em runtime, sem depender do caminho absoluto de quem compilou (`EVD-038`). `just test` continua disponível separadamente; evidências anteriores descrevem a receita executada naquela data.

> **Nota (2026-11-04) — Gap origem: INC-02.** `cargo check`, `cargo clippy`, `cargo nextest` e `cargo deny` (seção Supply chain) agora levam `--workspace`. A raiz do repositório é um package (`dfir-app`) e o workspace declara `members = [".", "engine", "protocol"]` sem `default-members`; sem `--workspace`, esses comandos só cobrem `dfir-app` e omitem `dfir-engine`, `dfir-protocol` e `engine/tests/sandbox.rs`. O `justfile` já usa `--workspace` nos alvos `check`, `nextest`, `clippy` e `licenses`.

## Supply chain

Estado: `PASS` em 2026-11-19 (`EVD-041`; anteriores `EVD-003`/`EVD-007`/`EVD-012`/`EVD-013`/`EVD-014`/`EVD-019`/`EVD-020`/`EVD-032`/`EVD-036`), com **uma** exceção de vulnerabilidade justificada (`RUSTSEC-2026-0459`, `WVR-004`) e duas exceções informativas documentadas (`RUSTSEC-2025-0141`, bincode; `RUSTSEC-2026-0543`, chrono). SQLite e chrono têm revisão até 2026-12-31. Exceção é risco aceito ou aviso analisado, não prova de inexploração.

```bash
cargo deny --workspace check
cargo audit
```

## Runtime

- aplicação não expõe porta de rede alguma, nem em loopback (interface nativa, ADR-0017);
- evidência tratada como entrada não confiável;
- engine sempre separado do processo principal;
- entradas e saídas devem permanecer dentro das raízes confiáveis;
- `PASS` testes adversariais de filesystem/rede/processo/protocolo (testes do workspace + e2e sobre EVTX real): `EVD-005`, `EVD-006`;
- `PASS` **contenção do engine provada com o engine de verdade** (ADR-0024): `cargo test --locked --workspace` (inclui `engine/tests/sandbox.rs`, que exige `EACCES`/`EPERM` onde devido e tem controle com a política `off`) e `scripts/e2e_headless.py` (repete a prova sobre o EVTX real) — `EVD-005`, `EVD-006`. **Rodar sempre pelo workspace**: `cargo test -p dfir-engine` não unifica as features do tokio e esconde falhas do runtime sob seccomp. Em Linux sem Landlock os testes de enforcement avisam e saem: o release no Linux **não** pode ser fechado nesse ambiente;
- `WAIVED` **Windows** (`WVR-001`, aprovado pelo operador em 2026-11-14, válido até 2026-12-31): o Job Object só foi compilado; o release **não declara suporte a Windows** nesta rodada. Condição de saída: `dfir-app engine-smoke` + `just e2e <evtx>` em Windows, conferindo o aviso esperado `SANDBOX_NOT_FULLY_APPLIED`;
- `WAIVED` **aarch64** (`WVR-002`, mesma validade): provado só em x86_64;
- `WAIVED` **CI automático** (`WVR-003`, F1-14, mesma validade): baseline roda manual (`just verify`, `EVD-047`);
- `PASS` **identidade analítica versionada:** raiz única, sucessora única e tipo/incidente estáveis; evidência estável exceto agregação IOC no mesmo incidente; violações falham antes de nova linha/auditoria, sem reparar histórico antigo (`EVD-033`/`EVD-034`/`EVD-035`);
- `PASS` **janela real na árvore final:** build e abertura isolada por 25 s até timeout 124 esperado, backend Gl/Mesa Intel, sem diagnósticos de fonte/glyph/scroll ou nomes de fixtures; sem navegação manual (`EVD-036`);
- `PASS` **matriz de risco sem perda silenciosa:** severity inválida/`NONE` ou confidence inválida/ausente falha com ID/campo antes dos entregáveis (`EVD-031`);
- `PASS` **impacto sem campos obrigatórios normalizados:** IMPACT_ASSESSMENT vigente com disponibilidade/confidencialidade/integridade desconhecida, ausente ou não textual, ou confiança inválida/ausente, falha com ID/campo antes da UI/entregáveis; `NONE` e impacto financeiro ausente continuam válidos (`EVD-046`);
- `PASS` **cobertura NIST sem lacuna perdida ou identidade divergente:** gap aberto vigente com subcategoria/prioridade desconhecida, ausente ou não textual, ou subcategoria divergente da correlation key, falha com ID/campo antes de lista, Function, entregáveis ou recálculo; rejeição no recálculo não escreve sucessora, `CLOSED` sem prioridade e legado sem `status` continuam válidos (`EVD-042`/`EVD-043`/`EVD-044`);
- `PASS` **lacunas de fonte sem payload obrigatório normalizado:** SOURCE_GAP vigente com fonte vazia/ausente/não textual, período inválido/invertido, motivo desconhecido ou `OTHER` sem nota falha com ID/campo antes da listagem/entregáveis; períodos ausentes/nulos continuam válidos (`EVD-045`);
- `PASS` **plano sem defaults de vocabulário:** os seis campos obrigatórios vigentes falham com ID/campo antes de ordenação, cobertura ou entregáveis quando desconhecidos, ausentes ou não textuais; arrays legados ausentes continuam válidos (`EVD-040`/`EVD-041`);
- `PASS` **recomendações sem reclassificação silenciosa:** categoria vigente desconhecida, ausente ou não textual falha com ID/campo antes de listar ou exportar; filtros e partição válidos preservados (`EVD-039`);
- `PASS` **caminho crítico limitado ao incidente:** os dois lados de `SEQUENCE_NEXT` devem pertencer ao incidente solicitado; relações antigas não são reparadas automaticamente (`EVD-030`);
- `PASS` **entregáveis sem arrays de plano perdidos:** `finding_ids`, `nist_subcategories` e `depends_on` inválidos falham em collect antes de renderizar; campos ausentes continuam legados (`EVD-029`);
- `PASS` **ciclos de plano sem bypass por dado inválido:** `depends_on` presente deve ser array de strings; erro impede a revisão e campo ausente continua legado vazio (`EVD-028`);
- `PASS` **agregação de IOC sem reescrita sobre dado inválido:** `sightings`, `sighting_count`, consistência e overflow são validados antes da nova versão; legado sem contador continua aceito (`EVD-027`);
- `PASS` **identidade de IOC antes de listar/exportar:** tipo desconhecido/ausente/não textual, valor inválido/ausente/não textual/não normalizado ou divergência da correlation key falha com ID/campo nos dois caminhos; legado sem `sighting_count` continua válido (`EVD-047`);
- `PASS` **RESULT persistido íntegro:** os leitores de timeline e analíticos exigem `items` array, com erro por linha e rollback na timeline; array vazio continua válido (`EVD-026`);
- `PASS` **timeline sem commit parcial:** JSONL inválido ou profundidade excessiva falha com a linha e rollback preserva eventos/índice anteriores; o e2e confirma a timeline real válida (`EVD-025`);
- `PASS` **Gap Analysis sem fechamento indevido:** payload inválido de um gap vigente falha antes da transação, sem nova linha ou superseding; o e2e confirma as dez lacunas válidas (`EVD-024`);
- `PASS` **resultados analíticos sem perda silenciosa:** `payload` e `provenance` inválidos produzem erro específico nas leituras por evidência e incidente; o e2e confirma todos os resultados e documentos válidos (`EVD-023`);
- `PASS` **custódia/auditoria legíveis sem ocultar corrupção:** `list_custody` e `list_events` rejeitam metadata inválido em vez de produzir `null`, e a verificação permanece `false`; o e2e confirma a cadeia válida íntegra (`EVD-022`);
- `PASS` **warnings de análise:** o e2e persiste os dois avisos do EVTX truncado; `EVD-021` também prova que JSON inválido e `warning_count` negativo/divergente falham em vez de apagar os detalhes;
- `PASS` **candidatos de detecção:** o e2e confirma que os candidatos nascem pendentes, que só a confirmação cria achado, que rejeitar exige motivo e que nova execução não duplica; `EVD-020` também prova que JSON persistido inválido falha antes das escritas analíticas;
- `PASS` **cópia antes de migration (RF-032):** `cargo test` cobre a cópia, a verificação, a recusa por falta de espaço e a adulteração (`EVD-005`); a execução real sobre dados reais aconteceu no e2e (`EVD-006`); política de retenção de `backups/` documentada em `docs/OPERATION.md` §3.1.1;
- SQLite local e staging com retenção definida.

## Release da interface

A interface é nativa (eframe/egui/egui_extras), parte do próprio binário `dfir-app` — não há artefato de build de frontend separado para verificar ou empacotar (ADR-0017). Os binários distribuídos (`dfir-app`, `dfir-engine`) não empacotam Node, Bun, Deno, Python, cargo ou rustc.

## Gates de verificação acrescentados (ADR-0019, ADR-0021, ADR-0022)

- **Ponta a ponta com engine real:** `just e2e caminho/arquivo.evtx` (`scripts/e2e_headless.py`) **110/110** em 2026-11-14 com o `security.evtx` de referência (2 261 eventos, 6 candidatos) — `PASS` (`EVD-002`). Qualquer EVTX é aceito; etapas que dependem de conteúdo (candidatos de detecção) são `FAIL` honesto quando o arquivo não contém os sinais.
- **Testes de GUI sem janela:** `cargo test --workspace` inclui os testes com `egui_kittest` — `PASS` (`EVD-001`).
- **Supply chain executado, não só configurado:** `cargo deny --workspace check` com advisories, bans, licenses e sources verdes (exceção individual e justificada em `deny.toml`) — `PASS` (`EVD-003`).
- `PASS` **Logs sem dado de evidência:** `dfir-app diagnostics` não contém nome de arquivo nem conteúdo de evento (teste dedicado) — `EVD-005`;
- **Documentação em dia:** `scripts/validate_fase0.py` falha se uma ADR ≥ 0018 não for citada na especificação ou se a documentação viva citar identificadores removidos do código.
- **Documentos sobre o incidente de demonstração:** o mesmo `scripts/e2e_headless.py` roda `dfir-app demo` e confere os cinco documentos e o `manifest.json` (contagens do cenário, rótulo DEMO, hashes, tabelas, máscara, determinismo, descarte da evidência) — `PASS` (`EVD-002`). Qualquer alteração em `service/demo.rs` sobe `DEMO_DATASET_VERSION` e ajusta as contagens esperadas (`docs/DEMO.md`).
- **Operação documentada:** `docs/OPERATION.md` cobre proteção do volume (RF-030) e o procedimento manual de backup, atualização e rollback (RF-032, parte documental). ~~A cópia automática do banco antes de migration continua pendente e **deve constar como lacuna** em qualquer release candidate enquanto não existir.~~ Substituído em 2026-11 por esta nota.
  - **Nota (2026-11-04) — Gap origem: INC-01.** A cópia automática do banco antes de migration foi implementada em 2026-10-27 (ADR-0026, RF-032) e é verificada por `migrate_with_backup` (cópia consistente, SHA-256 e verificação antes de publicar; `src/persistence/backup.rs`) e por `verify_published` (reconferência da pasta publicada, `dfir-app backup-verify <pasta>`). Deixa de constar como lacuna de release candidate (`docs/OPERATION.md` §3.1; item de RF-032 acima). Testes e e2e cobrem o mecanismo (`EVD-005`/`EVD-006`/`EVD-012`); uma migração sobre banco de caso real do operador continua `PENDING-HUMAN`.
