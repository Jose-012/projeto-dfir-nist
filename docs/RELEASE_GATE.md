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

## Gate documental

Estado: `PASS` em 2026-11-18 (`EVD-014`; anteriores `EVD-004`/`EVD-008`/`EVD-012`/`EVD-013`): Fase 0 com 0 erros/0 avisos e `documentation gate: PASS`.

```bash
just validate-fase0
just doc-gate
```

`just doc-gate` executa `scripts/validate_fase0.py` e `scripts/doc_gate.py` (Gap origem: INC-02).

## Gate Rust

Estado: `PASS` em Linux x86_64 em 2026-11-18 (`EVD-014`, nextest 435 passed/1 skipped; anteriores `EVD-001`/`EVD-005`/`EVD-010`/`EVD-012`/`EVD-013`); Windows e aarch64 viraram waivers formais (`WVR-001`, `WVR-002`, válidos até 2026-12-31; ver Runtime).

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

> **Receita atual (2026-11-18):** `just verify` usa nextest para unitários/integração e `cargo test --doc` para doc-tests. `just test` continua disponível separadamente; evidências anteriores descrevem a receita executada naquela data.

> **Nota (2026-11-04) — Gap origem: INC-02.** `cargo check`, `cargo clippy`, `cargo nextest` e `cargo deny` (seção Supply chain) agora levam `--workspace`. A raiz do repositório é um package (`dfir-app`) e o workspace declara `members = [".", "engine", "protocol"]` sem `default-members`; sem `--workspace`, esses comandos só cobrem `dfir-app` e omitem `dfir-engine`, `dfir-protocol` e `engine/tests/sandbox.rs`. O `justfile` já usa `--workspace` nos alvos `check`, `nextest`, `clippy` e `licenses`.

## Supply chain

Estado: `PASS` em 2026-11-18 (`EVD-014`; anteriores `EVD-003`/`EVD-007`/`EVD-012`/`EVD-013`), com **uma** exceção de vulnerabilidade justificada (`RUSTSEC-2026-0459`, `WVR-004`) e duas exceções informativas documentadas (`RUSTSEC-2025-0141`, bincode; `RUSTSEC-2026-0543`, chrono). SQLite e chrono têm revisão até 2026-12-31. Exceção é risco aceito ou aviso analisado, não prova de inexploração.

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
- `WAIVED` **CI automático** (`WVR-003`, F1-14, mesma validade): baseline roda manual (gates Rust, `EVD-014`);
- `PASS` **candidatos de detecção:** o e2e confirma que os candidatos nascem pendentes, que só a confirmação cria achado, que rejeitar exige motivo e que nova execução não duplica — `EVD-006`;
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
