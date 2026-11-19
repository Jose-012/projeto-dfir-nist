<!-- doc-type: reference | status: current -->
# Supply-chain gates

## Estado vigente (2026-11-19)

> Resumo do estado atual; o histórico cronológico segue abaixo, sem alteração.

- `cargo audit` e `cargo deny --workspace check` foram executados em Linux x86_64 com Rust/Cargo 1.100.0 e estão verdes em `EVD-051`: 1550 advisories carregados, 613 dependências do lock varridas, zero vulnerabilidades e um aviso informativo permitido de bincode; advisories, bans, licenses e sources ok. A parte supply-chain de `EVD-041` ficou `STALE` quando a mesma rotina encontrou oito vulnerabilidades novas de Wasmtime antes desta atualização.
- Não há exceção de vulnerabilidade vigente. As duas exceções informativas restantes no `deny.toml` são `RUSTSEC-2025-0141` (`bincode`, via `yara-x`) e `RUSTSEC-2026-0543` (`chrono 0.4.45`, sem CVE). `WVR-004` foi encerrado após `sqlx 0.9.1` + `libsqlite3-sys 0.40.0` remover `RUSTSEC-2026-0459` do lock; chrono tem revisão até 2026-12-31 e bincode é reavaliado a cada upgrade do `yara-x`.
- `encoding`/`RUSTSEC-2021-0153` e a cadeia zbus/AT-SPI estão fora do grafo. As exceções correspondentes foram removidas.
- `cargo vet`: dispensado no MVP (ADR-0018 D-2; confirmado pelo operador em 2026-11-04).
- As execuções anteriores permanecem registradas como `EVD-001` a `EVD-009`; pistas C/D usam `EVD-010`/`EVD-011`. Não se reescrevem evidências históricas.

| Crate | Versão no lock | Estado e evidência registrados | EVD |
|---|---|---|---|
| `wasmtime` | 48.0.9 | os oito advisories antigos foram resolvidos por 48.0.8; a revisão de 2026-11-19 encontrou `RUSTSEC-2026-0566`..`RUSTSEC-2026-0573` em 48.0.8 e a atualização mínima para 48.0.9 os removeu, sem trocar `yara-x` | `EVD-051` |
| `yara-x` | 1.22.1 | atualizado de `1.20.0`; destrava `wasmtime >= 48`; repetir `cargo test --workspace` e e2e a cada atualização (seccomp/Landlock validados contra o JIT de cada versão) | `[PENDING-HUMAN]` |
| `ctutils` | 0.4.7 | `RUSTSEC-2026-0385` resolvido (`0.4.2` → `0.4.7`) | `[PENDING-HUMAN]` |
| `tokio` | 1.54.1 | `RUSTSEC-2026-0386` resolvido (`1.53.1` → `1.54.1`) | `[PENDING-HUMAN]` |
| `clipboard-win` | 5.4.3 | *yanked* `5.4.1` resolvido (`5.4.3`) | `[PENDING-HUMAN]` |
| `rkyv` | 0.8.18 | **resolvido**: o baseline `0.8.16` exigia `>= 0.8.17`; o lock está em `0.8.18` | `[PENDING-HUMAN]` |
| `evtx` | 0.15.1 | **`RUSTSEC-2026-0431` resolvido** (`0.12.3` → `0.15.1`; alocação sem limite em `.wevtcache`). O crate passou a usar `encoding_rs` desde 0.13, então `RUSTSEC-2021-0153` (`encoding`) **saiu do grafo** e a exceção foi removida do `deny.toml`. e2e 110/110 com `security.evtx`: 2 261 eventos, 6 candidatos, sem avisos do parser — mesmo resultado da versão anterior | `EVD-002` |
| `libsqlite3-sys` | 0.40.0 | **`RUSTSEC-2026-0459` resolvido**. `sqlx 0.9.1` admite a versão corrigida; a atualização conjunta substituiu `sqlx 0.9.0` + `libsqlite3-sys 0.37.0`. A exceção foi removida de `deny.toml` e `.cargo/audit.toml`, e `WVR-004` ficou encerrado, sem apagar seu histórico. | `EVD-051` |
| `bincode` | 2.0.1 | exceção informativa `RUSTSEC-2025-0141` (dependência obrigatória do `yara-x 1.22.1` e também presente no dry-run de 1.23.0) mantida | `EVD-051` |
| `chrono` | 0.4.45 | exceção informativa `RUSTSEC-2026-0543` ("unmaintained", sem CVE); uso restrito a timestamps locais e parsing RFC 3339 de dados internos; revisão até 2026-12-31 | `EVD-012` |
| `ttf-parser` | — | **fora do grafo de build** desde o `eframe 0.37.0` sem `accesskit` (a exceção `RUSTSEC-2026-0192` foi removida do `deny.toml`) | `EVD-007` |
| `zvariant, zbus_names, zvariant_derive` | — | **fora do grafo de build** desde o `eframe 0.37.0` sem `accesskit` (exceções `RUSTSEC-2026-0370`/`0371`/`0372` removidas do `deny.toml`). A pendência de política do operador ("aceitar as três ou esperar o eframe que larga o zbus") foi resolvida pelo upgrade: o `eframe 0.37.0` permite desligar a feature `accesskit` do `egui-winit`, e o projeto a desligou | `EVD-007` |


O projeto trata mudanças de dependências e de toolchain como mudanças que afetam o release. O ambiente de desenvolvimento deve fornecer:

- `cargo audit` para os avisos do RustSec;
- `cargo deny --workspace check` para advisories, licenças, bans e fontes (**sempre com `--workspace`**, ver abaixo);
- ~~`cargo vet` for dependency audit policy;~~ dispensado no MVP (ADR-0018 D-2; confirmado pelo operador em 2026-11-04; Gap origem: INC-04);
- `cargo nextest run --workspace` para a suíte de testes, quando disponível (Gap origem: INC-02);
- `proptest` é dev-dependency de `dfir-protocol` e é usado em `protocol/tests/properties.rs` para profundidade JSON, round-trip de eventos e referências lógicas; não entra no runtime.



### Baselines desta revisão

- cargo-audit 0.22.2
- cargo-deny 0.20.2
- cargo-nextest 0.9.151
- ~~cargo-vet: versão efetiva deve ser registrada no ambiente/release após `cargo vet init` e revisão dos audits importados.~~ Sem efeito: `cargo vet` está dispensado no MVP (ADR-0018 D-2; Gap origem: INC-04).

`cargo-nextest` não possui retries no perfil padrão do repositório; flakiness deve ser investigada, não mascarada.

## RNG e compatibilidade do grafo

- Não há dependência direta de RNG dedicada a autenticação/sessões.
- `uuid` e as APIs criptográficas adotadas devem fornecer a aleatoriedade necessária sem criar uma camada RNG própria desnecessária.

## Toolchains não pertencentes ao runtime

O release deve distinguir claramente dependências de runtime, build e tooling:

- **Rust 1.100.0** (até 2026-11 era a 1.99.1; decisão do operador de usar a stable mais recente): toolchain do produto e dos testes Rust.
- **CPython:** 3.14.6 é a referência declarada em `toolchains.toml` e foi efetivamente instalado em scratch, com hash conferido contra o release, para `EVD-049`: `validate_fase0.py`, `doc_gate.py` e e2e 110/110 passaram com esse interpretador. O `/usr/bin/python3` do host continua 3.13.5; os comandos devem escolher explicitamente o interpretador 3.14.6 registrado. CPython não entra na imagem de produção por padrão.
- **Shell**: automação de desenvolvimento/QA; não é runtime da plataforma.

A versão efetivamente usada no gate deve ser capturada por `rustc --version --verbose`, `cargo --version --verbose` e `python3 --version`, conforme disponibilidade. Uma linha de toolchain no manifesto não constitui prova de execução.


## Componentes desta revisão

- `evtx 0.15.1`: parser EVTX externo usado como biblioteca; sem CLI embutida e sem features opcionais de multithreading/extra tooling (`default-features = false`). É a primeira versão com a correção de `RUSTSEC-2026-0431`.
- `dfir-protocol`: crate interno, sem publicação externa, centraliza o contrato JSONL.



## Exceções explícitas de advisories

- `RUSTSEC-2026-0459` (`libsqlite3-sys 0.37.0` / CVE-2026-119375) **não é mais uma exceção vigente**. Em 2026-11-19, dry-runs mostraram que `sqlx 0.9.1` admite `libsqlite3-sys 0.40.0`. A atualização foi aplicada no lock, os ignores foram removidos de `deny.toml` e `.cargo/audit.toml`, e o gate completo passou sem essa exceção (`EVD-051`). O registro histórico de `WVR-004` permanece, mas não cobre o lock atual nem prova inexploração de versões antigas.
- `RUSTSEC-2021-0153` (`encoding`) **deixou de existir no grafo** com o `evtx 0.15.1`; a exceção correspondente foi removida do `deny.toml` em 2026-11-14.
- Vulnerabilidades de segurança do `rkyv` não são aceitáveis como exceção. O baseline `rkyv 0.8.16` deve ser atualizado para `0.8.17` ou superior antes do fechamento do gate. **[Resolvido — nota de 2026-11-04, Gap origem: VAZ-04: o lock está em `rkyv 0.8.18`; ver "Estado vigente" no topo.]**
- A mudança de `cargo-deny` 0.20 removeu a chave legada `licenses.unlicensed`; a configuração usa somente a semântica atual de allow-list explícita.

- `serde_json_canonicalizer 0.3.2`: RFC 8785/JCS canonicalization for audit payloads; reject raw integers outside ±2^53 before canonicalization.

## `cargo deny` precisa de `--workspace`

A raiz do repositório é um pacote (`dfir-app`) e não um manifest virtual. Nesse caso `cargo deny check` usa **somente o pacote raiz** como origem do grafo e ignora os demais membros do workspace. Até esta revisão o gate rodava sem `--workspace`, então o grafo do `dfir-engine` (`evtx`, `yara-x`, `wasmtime`, `cranelift`…) nunca foi examinado. O sintoma era o aviso `advisory-not-detected` para `RUSTSEC-2021-0153` (o `encoding`, dependência do `evtx`, não aparecia no grafo verificado). O `justfile` (`just licenses`) agora usa `cargo deny --workspace check`; use sempre essa forma, inclusive no CI.

## Revisão do grafo do `yara-x 1.20.0` (adendo à ADR-0010)

A ADR-0010 manteve as features padrão do `yara-x` como decisão deliberada, mas não verificada. A verificação foi feita com `cargo deny --workspace check` (cargo-deny 0.20.2, banco RustSec de 2026-09-21) sobre o lockfile resolvido:

| | Features padrão | `default-features = false` + `constant-folding`, `exact-atoms`, `fast-regexp` |
|---|---|---|
| Pacotes no `Cargo.lock` | 492 (o engine adicionou 186 aos 306 anteriores) | 407 |
| Licenças rejeitadas | 24 (todas `Apache-2.0 WITH LLVM-exception`) | 24 (idem) |
| Advisories | `rsa` RUSTSEC-2023-0071 (Marvin, sem correção), `wasmtime` RUSTSEC-2026-0222 e -0269, `bincode` RUSTSEC-2025-0141 | `wasmtime` -0222 e -0269, `bincode` -0141 |

**Decisão**: `default-features = false` com só as três features de otimização que não trazem dependências. O ruleset embutido (`engine/rules/dfir_mvp_ruleset.yar`) não usa nenhum `import`, então nenhum módulo do `default-modules` (`console`, `crx`, `cuckoo`, `dex`, `dotnet`, `elf`, `lnk`, `macho`, `math`, `hash`, `pe`, `string`, `time`, `vt`, `zip`) faz falta, e a feature `crypto` que vários deles ativam (`rsa`, `dsa`, `ecdsa`, `p256`, `p384`, `x509-parser`, `der-parser`, `sha1`, `sha2`, `md-5`, `md2`, `digest`, `const-oid`) deixa de ser puxada pelo `yara-x` (`sha2`/`digest` seguem no grafo por outros motivos, p. ex. o app). `generate-proto-code` também sai: o crate traz o código protobuf pré-gerado. **Se uma regra futura precisar de `import "pe"` (ou outro módulo), habilite a feature específica (`pe-module`, `elf-module`…) e refaça esta revisão** — o `rsa` volta ao grafo junto com `pe-module`.

**Licença**: `Apache-2.0 WITH LLVM-exception` é a licença Apache-2.0 acrescida de uma exceção que *relaxa* a obrigação de atribuição para código compilado. É estritamente mais permissiva que `Apache-2.0`, já permitida; foi adicionada à allow-list do `deny.toml`. Não há licença copyleft nem conflitante no grafo.

**Advisories que permanecem** (todos com exceção explícita e justificada no `deny.toml`):

- `RUSTSEC-2026-0222` e `RUSTSEC-2026-0269` (`wasmtime 45.0.3`): o `yara-x 1.20.0`, a versão mais recente publicada, exige `wasmtime ^45.0.3`, e as correções só existem em `>=46.0.2`/`>=46.0.3`. Não há atualização possível pelo nosso lado. Análise de alcance: o -0269 é uma fuga de sandbox do sistema de arquivos **WASI**, e `wasmtime-wasi`/`cap-primitives` não estão no grafo (o `yara-x` habilita `wasmtime` só com `cranelift`+`runtime`); o -0222 trata de índices de tipo misturados entre `Engine`s distintos, e o `yara-x` mantém um único `Engine` global (`OnceLock`). Além disso, o WASM executado é gerado pelo próprio `yara-x` a partir do ruleset embutido — nunca vem da evidência analisada. **Reavaliar quando o `yara-x` lançar versão com `wasmtime >= 46.0.3`.**
- `RUSTSEC-2025-0141` (`bincode 2.0.1`): informativo (crate sem manutenção), não vulnerabilidade. É dependência obrigatória do `yara-x`, usada em `Rules::serialize`/`deserialize` (que o engine não chama) e para decodificar dados internos do próprio compilador. Reavaliar a cada upgrade do `yara-x`.

Ressalva de método: o texto completo dos avisos do GitHub Security Advisories não foi lido integralmente; a análise de alcance se apoia no título/versões corrigidas dos avisos, nas notas de release do `wasmtime` e na leitura do código do `yara-x 1.20.0`. As duas exceções do `wasmtime` são, portanto, uma decisão de risco documentada — não uma prova de inexploração — e devem ser revistas na próxima atualização do `yara-x`.

## Nova árvore de dependências: interface nativa (ADR-0017)

`axum`/`tower`/`tower-http`/`askama` saíram; entraram `eframe 0.36.2`, `egui 0.36.2`, `egui_extras 0.36.2` (com a feature `datepicker`) e `rfd 0.17.2` (com a feature padrão `xdg-portal` no Linux — sem depender de GTK do sistema).

**O que foi verificado nesta sessão** (leitura direta do `Cargo.toml` de cada crate baixado, não `cargo deny`): licença declarada de `eframe`/`egui`/`egui_extras` é `MIT OR Apache-2.0`; `rfd` é `MIT`. Ambas já constam na allow-list de `deny.toml`.

**O que não foi verificado** — `cargo-deny`/`cargo-audit` não estão instalados no ambiente onde esta migração foi implementada, então nada abaixo tem evidência real ainda:

- advisories RustSec na árvore transitiva completa (`winit`, `accesskit` e sua cadeia de acessibilidade via AT-SPI/`zbus` no Linux, `glow`/`glutin`, `ashpd`/`zbus` puxados por `rfd`);
- licenças da árvore transitiva completa (esperado MIT/Apache-2.0/BSD-3-Clause/Zlib de acordo com o ecossistema `egui`, mas não confirmado crate a crate);
- contagem de pacotes/tamanho do `Cargo.lock` resultante (o lockfile foi regenerado sem `--locked` nesta sessão, com um toolchain diferente do pinado em `rust-toolchain.toml`).

**Antes de fechar este gate**: rodar `cargo deny --workspace check` e `cargo audit` com o toolchain `1.97.1` real e registrar o resultado aqui, no mesmo formato usado acima para o `yara-x`.

## Execução de 2026-10-11 (ADR-0021)

`cargo deny --workspace check` (cargo-deny 0.20.2, binário oficial, banco RustSec baixado em 2026-10-11) foi executado pela primeira vez sobre a árvore real com `eframe`/`egui` 0.36.2, em rustc 1.91.1. Resultado após os ajustes abaixo: `advisories ok, bans ok, licenses ok, sources ok`.

- **Licenças.** Faltavam na lista: `BSD-2-Clause` (arrayref), `ISC` (libloading) e `BSL-1.0` (clipboard-win, error-code), todas permissivas; adicionadas. As fontes embutidas do egui (`epaint_default_fonts`: `OFL-1.1` e `Ubuntu-font-1.0`) são permitidas **apenas** para essa crate via `exceptions`.
- **Avisos novos.** RUSTSEC-2026-0316 e RUSTSEC-2026-0327 (wasmtime 45, fixado pelo yara-x 1.20.0; não há versão 45.x corrigida, os reparos estão em 36.0.16+, 48.0.3/48.0.4+ e 49.0.1/49.0.2+): ambos tratam do modelo de componentes (lifting dinâmico de registros e callbacks async-lifted); `cargo tree -e features` mostra que o wasmtime é compilado só com `cranelift`, `runtime` e `std`, sem `component-model` nem `async`. RUSTSEC-2026-0192 (`ttf-parser` não mantido, informativo): transitivo de winit → sctk-adwaita → ab_glyph, usado só para a decoração de janela no Wayland. Os três têm exceção justificada em `deny.toml`.
- **Crate retirada.** `yoke-derive` 0.8.3 estava marcada como retirada (yanked); `Cargo.lock` atualizado para 0.8.4.
- **Não executado:** `cargo audit` e `cargo vet` (o banco RustSec usado pelo deny é o mesmo do audit); `cargo deny` com o toolchain pinado 1.97.1. A dispensa de `cargo vet` (ADR-0018) continua dependendo de F1-15 atualizar `AGENTS.md` §8.
  - **Nota (2026-11-04) — Gap origem: INC-04.** A dispensa de `cargo vet` foi confirmada pelo operador e `AGENTS.md` §8 foi atualizado; `cargo audit` e `cargo deny --workspace check` foram executados em 2026-10-29 (seção abaixo).

## `egui_kittest` (2026-10-11, ADR-0022)

Dev-dependency (`egui_kittest` 0.36.2, `kittest` 0.4.0, mais `toml`/`serde_spanned` e atualização de `toml_datetime`/`toml_parser`): só em `cargo test`, não entra no binário distribuído. `cargo deny --workspace check` continuou com `advisories ok, bans ok, licenses ok, sources ok`. Exige rustc 1.95 (como o egui 0.36).

## Atualização 2026-10-27 — `libc` e `windows-sys` no engine (ADR-0024)

O engine ganhou duas **dependências diretas**, ambas já presentes no grafo (o `Cargo.lock` só ganhou as arestas `libc` e `windows-sys` 0.61.2 em `dfir-engine`, **sem crate novo**):

- `libc` (só Linux): chamadas de sistema do sandbox (`landlock_*`, `prctl`, `setrlimit`). Landlock e seccomp são feitos **à mão sobre `libc`**, sem as crates `landlock`/`seccompiler`: cada uma traria árvore própria (`bitflags`, `thiserror`, `enumflags2`...) para um ganho pequeno, e o filtro é uma lista de negação de 44 chamadas com testes de enforcement.
- `windows-sys` (só Windows; features `Win32_Foundation`, `Win32_System_JobObjects`, `Win32_System_Threading`): Job Object.

`cargo deny` e `cargo audit` **não foram reexecutados** nesta rodada (sem a ferramenta no ambiente); como não há crate novo, o resultado de 2026-09-21 não muda por esta mudança, mas isso é inferência, não medição. Fica para o operador (F1-15).

## Atualização 2026-10-29 — `cargo audit`/`cargo deny` reexecutados; `cargo update` aplicado (F1-15)

**Situação antes (Cargo.lock de 2026-10-27, base RustSec de 2026-10-29, 1 386 avisos):** o `cargo audit` achou **9 vulnerabilidades** e o `cargo deny --workspace check advisories` **falhava**. Os avisos eram posteriores às exceções revisadas de 2026-09/10, então a falha **não vem das mudanças de 2026-10-27**:

| Aviso | Crate | Corrigido em | Alcançável aqui? |
|---|---|---|---|
| RUSTSEC-2026-0222, -0269, -0316, -0327, -0362, -0363, -0359, -0379 | `wasmtime 45.0.3` (preso por `yara-x 1.20.0`) | `wasmtime >= 48.0.7` | em geral não (só módulos *core* gerados pelo yara-x; sem componente, async nem WASI), **mas não é verificado**; `-0379` é só aarch64 |
| RUSTSEC-2026-0385 | `ctutils 0.4.2` (comparação não constante) | `>= 0.4.4` | não avaliado |
| RUSTSEC-2026-0386 | `tokio 1.53.1` (`AsyncFd::new` viola I/O safety) | `>= 1.53.3` | não avaliado |
| `clipboard-win 5.4.1` | *yanked* | `5.4.3` | — |
| RUSTSEC-2026-0370/0371/0372 | `zvariant`, `zbus_names`, `zvariant_derive` (sem manutenção) | — | só a árvore de UI (acessibilidade D-Bus, Linux); nunca o engine nem dados de evidência |

**O que foi feito:** `cargo update` completo (111 pacotes; 0 crates diretos mudaram de *requisito*): `yara-x 1.20.0 → 1.22.1`, `wasmtime 45.0.3 → 48.0.8`, `tokio 1.53.1 → 1.54.1`, `ctutils → 0.4.7`, `clipboard-win → 5.4.3` e o restante dentro das faixas já declaradas. Um `cargo update -p yara-x` isolado **não resolve** (o `yara-x 1.22.1` exige `wasm-bindgen ^0.2.129`, e o `web-sys` do eframe fixa `=0.2.128`): é preciso o `cargo update` completo.

**`deny.toml`:** removidas as 4 exceções do wasmtime 45 (a condição que elas mesmas fixavam, "reavaliar quando o yara-x sair com wasmtime >= 48.0.4", foi atendida); acrescentadas 3 exceções **revisáveis** para a família `zvariant`/`zbus_names` (informativas, só UI), no mesmo formato da de `ttf-parser`. **Decisão de política que é sua:** se preferir não aceitar essas três, a alternativa é esperar um `eframe` que largue o `zbus` (o `eframe 0.37.0` existe; não foi verificado se resolve).

**Resultado depois:** `cargo audit`: **0 vulnerabilidades** (6 avisos informativos: `bincode`, `encoding`, `ttf-parser`, `zbus_names`, `zvariant`, `zvariant_derive`). `cargo deny --workspace check`: `advisories ok, bans ok, licenses ok, sources ok`. Com o lock novo, **neste pacote exato**, em Linux x86_64 e rustc 1.97.1: `fmt --check`, `clippy --workspace --all-targets -D warnings` (limpo), `test --workspace` (app 288, engine 83 + 7 do sandbox, protocolo 26) e e2e **110/110**; e o engine real sobre o `security.evtx` rodou timeline (2 261 eventos), YARA e detecção (6 candidatos) **sob `required`** (Landlock ABI 7 + seccomp), com o JIT do wasmtime 48, resultados idênticos aos de antes. **Não repetido com o lock novo:** Windows (Job Object) e aarch64.

**Não atualizado, de propósito:** `eframe`/`egui`/`egui_extras`/`egui_kittest` 0.36.2 → 0.37.0, `evtx` 0.12.3 → 0.14.0 e `serde_json_canonicalizer` 0.3.2 → 0.4.1 são mudanças de versão 0.x (potencialmente quebram API; a canonicalização alimenta a **cadeia de auditoria**, então uma troca exige teste de compatibilidade dos hashes existentes). Cada uma é uma tarefa própria. `proptest`, `uuid`, `libc`, `serde_json`, `sha2` e outras entraram só por dentro das faixas do `Cargo.toml`.

## Atualização 2026-11-14 — `evtx 0.15.1` e exceção do `libsqlite3-sys`

Rodada executada com rustc/cargo 1.99.1 em Linux x86_64, `--locked`. Corrige SEC-01, SEC-02 e TOOL-01 do registro de problemas de 2026-11-14.

- **`evtx 0.12.3 → 0.15.1`** (`engine/Cargo.toml`, `default-features = false`). `RUSTSEC-2026-0431` (alocação sem limite ao abrir `.wevtcache`) deixa de existir. O parser do engine só chama `EvtxParser::from_path`/`serialized_records`/`into_json_value`, que não mudaram; a API compilou sem ajustes. Verificação funcional: e2e **110/110** com o `security.evtx` de referência (SHA-256 `5f29a03c…9a4d8d`), **2 261 eventos**, 6 candidatos de detecção, nenhum aviso do parser, mesmos resultados da versão anterior (`EVD-002`). `cargo tree --workspace -i encoding` deixou de encontrar o crate; a exceção `RUSTSEC-2021-0153` foi removida do `deny.toml`.
- **`libsqlite3-sys 0.37.0`**: sem atualização possível. `sqlx-sqlite 0.9.0` declara `libsqlite3-sys = ">=0.30.1, <0.38.0"`, e `sqlx 0.9.0` é a última versão publicada; `cargo update -p libsqlite3-sys --precise 0.39.0` termina em erro de seleção de versão. `RUSTSEC-2026-0459` fica como exceção temporária em `deny.toml` e `.cargo/audit.toml`, com análise de alcance e revisão até 2026-12-31. **Isto não é prova de inexploração**; é uma decisão de risco documentada.
- **`async-executor 1.14.0` (yanked) → 1.14.1**, sem mudança de requisito.
- **Resultado:** `cargo audit --no-yanked`: **0 vulnerabilidades** e 5 avisos informativos; `cargo deny --workspace check`: `advisories ok, bans ok, licenses ok, sources ok`; `just supply-chain` roda as duas etapas e termina com sucesso (`EVD-003`).
- **Não repetido nesta rodada:** Windows, Linux aarch64 e incidente real. O e2e foi executado no Linux x86_64 com o sandbox `required` (Landlock + seccomp), como antes.

## Atualização 2026-11-14 (tarde) — `eframe`/`egui`/`egui_extras`/`egui_kittest` 0.36.2 → 0.37.0; `cargo update` geral

Rodada executada com rustc/cargo 1.99.1 em Linux x86_64, `--locked`. Resolve a pendência de política dos avisos informativos `zbus`/`zvariant` (RUSTSEC-2026-0370/0371/0372) e `ttf-parser` (RUSTSEC-2026-0192) registrada em 2026-10-29.

- **`eframe 0.37.0` com `default-features = false`** e features `default_fonts`, `wgpu`, `x11`, `wayland`, `clipboard`, `links` (`Cargo.toml`). Em 0.37 a feature `accesskit` do `egui-winit` é **opcional**; desligada, a cadeia de acessibilidade D-Bus (`zbus`, `zvariant`, `zbus_names`, `zvariant_derive`, `atspi`, `accesskit_unix`, `ashpd`, `sctk-adwaita`, `ttf-parser`) **sai do grafo de build** (`cargo tree --locked -e normal,build` não encontra nenhum desses nomes; o lock caiu para 612 pacotes). `accesskit 0.25.2` permanece como dependência não-optional do próprio `egui 0.37` (API de árvore de acessibilidade do egui), mas sem a feature `accesskit` do `egui-winit` nenhuma integração AT-SPI/D-Bus é compilada. `rfd` subiu para 0.18.0, `egui_kittest` para 0.37.0. O app compilou **sem nenhuma mudança de código** no binário; só os testes de GUI exigiram ajustes (harness com `allow_missing_glyphs` e nome acessível em cada campo via `labelled_by`/`ComboBox::from_label`, por causa das checagens novas do `egui_kittest 0.37`).
- **`cargo update --workspace` geral**: 0 mudanças pendentes (o lock já estava nas versões máximas compatíveis com rustc 1.99); foram aplicados minor updates de `jiff 0.2.38→0.2.44`, `toml 1.1.8→1.2.0`, `uuid 1.29.0→1.29.2` (o `defmt` saiu do grafo). Os 52 pacotes restantes disponíveis ficam bloqueados por `rust-version > 1.99`.
- **`deny.toml`**: removidas as 4 exceções mortas (`RUSTSEC-2026-0370/0371/0372`, `RUSTSEC-2026-0192`). Restam apenas `RUSTSEC-2026-0459` (`libsqlite3-sys`, SEC-02) e `RUSTSEC-2025-0141` (`bincode`, via `yara-x`).
- **Resultado:** `cargo audit`: **0 vulnerabilidades**, 1 aviso permitido (`bincode`); `cargo deny --workspace check`: `advisories ok, bans ok, licenses ok, sources ok`; `just verify` PASS (nextest 410/410, clippy `-D warnings` limpo, fmt ok); `just e2e <security.evtx>` **110/110** (`EVD-005`, `EVD-006`, `EVD-007`).
- **Não repetido nesta rodada:** Windows (Job Object), Linux aarch64, janela real da GUI. A `accesskit` desligada remove a dependência de D-Bus do processo da UI no Linux; se no futuro for preciso leitor de tela, a decisão volta por ADR.

## Atualização 2026-11-19 (segurança): Wasmtime 48.0.9 e SQLite 0.40.0

Antes da mudança, `just supply-chain` falhou com oito vulnerabilidades de `wasmtime 48.0.8` (`RUSTSEC-2026-0566`..`RUSTSEC-2026-0573`), tornando `STALE` a parte supply-chain de `EVD-041`. Dry-runs também mostraram que a antiga premissa de `WVR-004` estava superada: `sqlx 0.9.1` permite `libsqlite3-sys 0.40.0`.

Foi aplicado apenas `cargo update -p sqlx -p libsqlite3-sys -p wasmtime`, sem trocar `yara-x`, `chrono` ou framework. O lock passou a 613 pacotes. Depois, `cargo audit --no-yanked` e `cargo deny --workspace check advisories` passaram; os ignores de SQLite foram removidos dos dois arquivos de configuração. `just supply-chain` voltou a passar sem `RUSTSEC-2026-0459`, e `cargo check --workspace --locked --all-targets` terminou em 2m13s (`EVD-051`). O dry-run de `yara-x 1.23.0` ainda mostra `bincode 2.0.1`; o dry-run de chrono não propõe atualização. As duas exceções informativas restantes continuam sob revisão.
