<!-- doc-type: llm-rule | status: current -->
# Prompt de continuidade — DFIR/NIST / baseline v6.5.1

Você é a LLM responsável por continuar este projeto.

## Autoridade obrigatória

Antes de alterar qualquer coisa, leia:

- `AGENTS.md`
- `docs/DOCUMENT_AUTHORITY.md`
- `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md` (fonte primária de requisitos de negócio) e `docs/DEMAND_TRACE.md`
- `docs/PROJECT_SPEC_v6_5_1.md`
- `docs/adrs/ADR-0018-realinhamento-demanda-original.md` (escopo vigente: o que foi cortado, adiado e acrescentado)
- `docs/FASE1_BACKLOG.md`, `docs/FASE1_STATUS.md`
- `docs/FASE0_STATUS.md`
- `docs/rtm.md`
- `docs/RELEASE_GATE.md`
- `docs/SUPPLY_CHAIN.md`
- `docs/adrs/ADR-0017-interface-nativa-eframe-egui.md`
- `Cargo.lock`
- `Cargo.toml`

As especificações anteriores (v6.3.13, v6.4.1, v6.4.5) estão arquivadas em `docs/archive/` e são históricas, não normativas.

Pedidos do operador que mudam tecnologia ou escopo são decisões de arquitetura (ADR), **não** requisitos da demanda. Só o texto da demanda original ocupa o nível 1 de precedência.

## Arquitetura atual

```text
Production Runtime
  Rust dfir-app (com interface nativa eframe/egui/egui_extras)
  Rust dfir-engine

Build Toolchain
  Rust/Cargo

Engineering/QA Tooling
  CPython 3.14.x
  Shell
```

`dfir-app` e `dfir-engine` permanecem em Rust.

A interface é nativa (`src/gui/`, eframe/egui/egui_extras), parte do binário `dfir-app`; chama `src/service/` diretamente, no mesmo processo. Não há navegador, HTML, TypeScript, servidor HTTP nem porta de rede (ADR-0017).

Python/Shell não são backend do produto.

Node, Bun e Deno não pertencem a este projeto: o frontend que os usava foi removido por inteiro (ADR-0017; ADR-0009, sobre Bun→Node, fica só de valor histórico).

## Fronteiras

- Interface ↔ lógica de negócio: chamada de função direta, in-process (`src/gui/` → `src/service/`); nenhuma serialização nem transporte de rede no meio.
- App ↔ engine: stdin/stdout + JSONL.
- Engine: sem SQLite, secrets, autenticação ou RBAC.

## Regras de stack

- SQLite + SQLx + SQL explícito continuam.
- A crate `evtx` (requisito e lock em 0.15.1) é a única implementação de parsing EVTX do engine no MVP; não reimplementar BinXML.
- `dfir-protocol` é a única definição dos tipos JSONL compartilhados; não usar `Box<dyn Serialize>`/trait objects para mensagens.
- Repositories finos não são ORM.
- Não introduzir uma crate RNG direta sem necessidade demonstrada; use as APIs criptográficas já fornecidas pelas dependências efetivamente necessárias.
- Não adicionar outra crate de parsing EVTX sem ADR/benchmark; o engine deve permanecer fino.
- Novas dependências de UI (`eframe`/`egui`/`egui_extras`/outras) entram só mediante necessidade demonstrada e nova validação de supply chain/offline — mesma régua que valia para bibliotecas de frontend antes do ADR-0017.

## Gates pré-mudança

```text
cargo metadata --locked
cargo tree --locked -e features
cargo tree --locked -d
cargo deny --workspace check
cargo audit
```

> `cargo vet` está dispensado no MVP (ADR-0018 D-2; confirmado pelo operador em 2026-11-04); permanecem `cargo deny --workspace check` e `cargo audit`. Gap origem: INC-04.

## Gates pós-mudança

```text
cargo fmt --all -- --check
cargo check --workspace --locked --all-targets
cargo clippy --workspace --locked --all-targets --all-features -- -D warnings
cargo nextest run --workspace --locked
```

> Gap origem: INC-02 (2026-11-04). `--workspace` também em `cargo deny --workspace check` nos gates pré-mudança: a raiz é um package e o workspace não declara `default-members`; sem ele, os comandos só cobrem `dfir-app`. Ver `docs/RELEASE_GATE.md`.

## Escopo do MVP (ADR-0018)

Fora do MVP, na Fase 4: `.dfirpkg`, atualização offline assinada, geração de Ansible, benchmark `just perf`, retry/DLQ de jobs, perfis executivo/técnico de relatório. Dentro do MVP e **implementados**: evento normalizado, detecção determinística com confirmação do analista (F1-26/F1-28), Finding/causa-raiz, item de plano, visão NIST por Function, proveniência/lacunas, redação de dados pessoais e os cinco documentos de entregável. A importação CSV/JSONL (F1-27) está **adiada** por decisão do operador (ADR-0025). Antes de implementar qualquer ticket, leia `docs/DEMAND_TRACE.md`.

## Decisões posteriores (ADR-0019 a ADR-0023)

- **CLI headless** (`dfir-app import|evidence-list|analyze|results|findings|destroy|deliverables|demo|diagnostics`) e `scripts/e2e_headless.py` (engine real + EVTX real); sem TUI.
- **Sem limite de saída nem truncamento** (ADR-0020): o freio é o tempo do engine, a memória virtual, o tamanho de importação e o espaço em disco (`DFIR_MIN_FREE_DISK_BYTES`).
- **Timeline no SQLite, paginada** (ADR-0021), avisos do parser gravados no job, arquivo grande no Linux, `cargo deny` executado.
- **Testes de GUI com `egui_kittest` e logs em arquivo** (ADR-0022): nunca logar nome de arquivo de evidência nem conteúdo de eventos.
- **Tipos `FINDING`, `ROOT_CAUSE`, `PLAN_ITEM` e migration 0010; plano de conclusão do protótipo** (ADR-0023): o operador definiu o projeto como protótipo e deixou fora desta fase os formatos além de EVTX; a ordem de execução (Etapas 0 a 7) está na ADR. Todo tipo analítico novo exige migration com a receita da 0010. Etapas 1 (migration e tipos), 2 (Finding, causa-raiz e aba "Achados") 3 (item de plano e aba "Plano") 4 (visão NIST por Function, F1-30) 5 (gerador dos entregáveis, F1-07: `service/deliverables/`, aba "Entregáveis", `dfir-app deliverables`) e 6 (proveniência, aba "Lacunas" e máscara de dados pessoais, F1-31 e F1-32; migration 0011) concluídas entre 2026-10-13 e 2026-10-18; **Etapa 7 concluída em 2026-10-23** (`dfir-app demo`, `service/demo.rs`, e2e sobre os cinco documentos, `docs/DEMO.md`, `docs/OPERATION.md`, reconciliação). O plano da ADR-0023 acabou. F1-16 e os tickets F1-19/F1-20/F1-21/F1-23/F1-24/F1-33 estão `VERIFIED` por `EVD-010`/`EVD-011`; o estado corrente completo está em `docs/rtm.md`. Permanecem abertos a verificação no operador (Windows, Linux aarch64 e incidente real), CI/F1-14 e as decisões já registradas no backlog.
- **Rodada 2026-10-27 — contenção, detecção, cópia antes de migration** (ADR-0024, ADR-0025, ADR-0026): **F1-22** (ambiente reduzido, arquivo único, `kill_on_drop`, Landlock + seccomp + `no_new_privs` no Linux provados com o engine real; Job Object no Windows só compilado), **F1-26** (campos estruturais no evento, esquema 4, migration 0012, filtro `campo:valor`), **F1-28** (19 regras em `engine/src/detect.rs`, candidatos pendentes que o analista confirma ou rejeita; `docs/DETECTION_RULES.md`), **RF-032 automático** (`persistence/backup.rs`) e neutralização de caracteres de direção de texto (`text_safety`). **F1-27 (CSV/JSONL) está adiado por decisão do operador.** Regras que valem daqui em diante: a regra de detecção só produz **candidato**; mudar uma regra ou limiar exige mudar `RULE_SET_VERSION` e regenerar `docs/DETECTION_RULES.md` (um teste confere); o engine nunca deve ser testado só com `-p dfir-engine` (a unificação de features do workspace muda o runtime: rode `cargo test --workspace`); qualquer migration que recrie tabela passa pela cópia automática e ganha teste de preservação de dados. Continua pendente a verificação do operador em Windows, em Linux aarch64 e sobre um incidente real.
- **Rodada 2026-10-29 — dependências** (`docs/SUPPLY_CHAIN.md`): `cargo update` completo (`yara-x 1.22.1`, `wasmtime 48.0.8`, `tokio 1.54.1`); `cargo audit` 0 vulnerabilidades e `cargo deny --workspace check` ok; 3 exceções informativas de UI no `deny.toml` aguardam revisão do operador. Atualizar `yara-x`/`wasmtime` exige repetir `cargo test --workspace` e o e2e (o sandbox é validado contra o JIT de cada versão). `eframe`/`egui` 0.37 e `serde_json_canonicalizer` 0.4 ficaram de fora.
- **Rodada 2026-11-14 — registro de problemas de execução local** (SEC-01, SEC-02, TOOL-01, GUI-01, GFX-01, BUILD-01, DOC-01): `evtx 0.12.3 → 0.15.1` corrige `RUSTSEC-2026-0431` e remove `encoding`/`RUSTSEC-2021-0153`; `libsqlite3-sys 0.37.0` fica como exceção temporária justificada para `RUSTSEC-2026-0459` (sem `sqlx` publicado que admita `>=0.39.0`; revisar até 2026-12-31); `just supply-chain` roda `cargo audit` e `cargo deny --workspace check` e ambos passam; o seletor de arquivos roda fora da thread da UI e há caminho digitado/arrastar-e-soltar; o adaptador de GPU é registrado no log; o e2e ficou robusto a qualquer EVTX (não estoura mais sem candidatos). `just verify` e e2e **110/110** com `security.evtx` em rustc 1.99.1; evidências `EVD-001` a `EVD-003`. Detalhe: `docs/SUPPLY_CHAIN.md`, `docs/OPERATION.md` §3D, `docs/NATIVE_UI_ARCHITECTURE.md`.
- Estado real de verificação mais recente: `EVD-012` (rustc/cargo 1.100.0, Linux x86_64, 2026-11-18; nextest 426/426 e e2e 110/110). Windows, aarch64 e incidente real continuam sem execução. Consultar `docs/rtm.md` e `docs/EVIDENCE_REGISTER.md`, não uma narrativa duplicada neste prompt.

## Regra de honestidade

Se uma ferramenta não estiver disponível, faça revisão estática e marque o gate como `PENDING`. Nunca declare compilação, runtime, supply chain ou frontend verificados sem execução real. Nunca invente source/checksum de registry em `Cargo.lock`; gere-os com Cargo no ambiente de release.

## Formato de resposta da LLM

```xml
<implementation-report>
  <status>IMPLEMENTED | INTEGRATED | VERIFIED | BLOCKED | PENDING</status>
  <changes>...</changes>
  <traceability>RF/D/CA/TEST/GATE/ADR</traceability>
  <tests>executados/pendentes</tests>
  <evidence>EVD-*</evidence>
  <waivers>WVR-*</waivers>
  <decisions>novas ou superseded</decisions>
  <documentation-updated>arquivos</documentation-updated>
</implementation-report>
```


Toolchains de referência: `toolchains.toml`.


## Hardening preventivo pré-congelamento
- O processamento de evidência deve falhar fechado quando o isolamento exigido pela política `DFIR_ENGINE_SANDBOX` não puder ser aplicado (ADR-0024); "sandbox" é a contenção do SO aplicada pelo engine, "isolamento" é processo separado + ambiente reduzido + arquivo único.

## Supply-chain corrections recentes

- `cargo-deny 0.20.x`: não usar `licenses.unlicensed`; a allow-list explícita é a política ativa.
- `rkyv 0.8.16` foi identificado em `cargo audit` com advisories RUSTSEC-2026-0233/0234/0235; atualizar o grafo para `rkyv >=0.8.17`.
- `encoding 0.2.33` saiu do grafo com `evtx 0.15.1`; a exceção histórica foi removida. Não reintroduzir essa dependência sem necessidade demonstrada.
- ~~Antes de `cargo vet`, inicializar o store com `cargo vet init` uma única vez e versionar/revisar o resultado.~~ Sem efeito: `cargo vet` está dispensado no MVP (ADR-0018 D-2; Gap origem: INC-04).
