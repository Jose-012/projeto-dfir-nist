<!-- doc-type: log | status: current -->
# Relatório de alterações — 2026-10-29

> **Nota (2026-11-04) — Gap origem: EVI-02** (cabeçalho `doc-type`: AUS-08). Este relatório não repete a afirmação de reconciliação documental do relatório de 2026-10-27; a correção dessa afirmação está na nota de `RELATORIO_ALTERACOES_2026-10-27.md`. As execuções aqui narradas (`cargo audit`, `cargo deny --workspace check`, testes e e2e em rustc 1.97.1) não têm `EVD-*` em `docs/EVIDENCE_REGISTER.md`: `[PENDING-HUMAN]` registrar.

Pedido: verificar se há atualizações de dependências a fazer e entregar o pacote alterado. Base: pacote `projeto-dfir-nist-v6_5_0-2026-10-27`. Código-fonte **não** mudou; mudaram `Cargo.lock`, `deny.toml` e a documentação.

## Por que

O `cargo audit` (base RustSec de 2026-10-29) achou **9 vulnerabilidades** no lock anterior, e o `cargo deny --workspace check` falhava. Os avisos são posteriores às exceções antigas e **não** vêm da rodada de 2026-10-27.

| Aviso | Crate | Corrigido em |
|---|---|---|
| RUSTSEC-2026-0222, -0269, -0316, -0327, -0362, -0363, -0359, -0379 | `wasmtime 45.0.3` (preso por `yara-x 1.20.0`) | `>= 48.0.7` |
| RUSTSEC-2026-0385 | `ctutils 0.4.2` | `>= 0.4.4` |
| RUSTSEC-2026-0386 | `tokio 1.53.1` (*unsound*) | `>= 1.53.3` |
| *yanked* | `clipboard-win 5.4.1` | `5.4.3` |

Alcançabilidade dos avisos do wasmtime: o `deny.toml` anterior já argumentava que não (só módulos *core* gerados pelo yara-x; sem componente, async nem WASI). **Isso não foi verificado.**

## O que mudou

- `Cargo.lock`: `cargo update` completo (111 pacotes): `yara-x 1.22.1`, `wasmtime 48.0.8`, `tokio 1.54.1`, `ctutils 0.4.7`, `clipboard-win 5.4.3`. Nenhum `Cargo.toml` mudou. `cargo update -p yara-x` isolado não resolve (o `yara-x 1.22.1` exige `wasm-bindgen ^0.2.129`; o `web-sys` do eframe fixa `=0.2.128`).
- `deny.toml`: 4 exceções do wasmtime 45 removidas (a condição de reavaliação foi atendida); **3 exceções informativas novas** (`zvariant`, `zbus_names`, `zvariant_derive`), só da árvore de UI (eframe → accesskit → D-Bus, Linux).
- Documentação: `docs/SUPPLY_CHAIN.md`, `docs/FASE1_STATUS.md`, `docs/rtm.md` (F1-15), `PROMPT_NOVA_CONVERSA.md`, `CHANGELOG.md`.

## Verificação (rustc 1.97.1, Linux x86_64, `--locked`)

| Gate | Resultado |
|---|---|
| `cargo audit` | 0 vulnerabilidades (6 avisos informativos: `bincode`, `encoding`, `ttf-parser`, `zbus_names`, `zvariant`, `zvariant_derive`) |
| `cargo deny --workspace check` | `advisories ok, bans ok, licenses ok, sources ok` (neste pacote) |
| `cargo metadata --locked` | lock consistente com os `Cargo.toml` (neste pacote) |
| `scripts/doc_gate.py`, `scripts/validate_fase0.py` | PASS; 0 erros, 0 avisos (neste pacote) |
| `cargo fmt --all -- --check` | limpo (neste pacote) |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | limpo (neste pacote) |
| `cargo test --locked --workspace` | app 288 (1 ignorado como antes), engine 83 + 7 do sandbox, protocolo 26 (neste pacote) |
| `scripts/e2e_headless.py`, engine e `security.evtx` reais | 110/110 (neste pacote) |
| engine real sob `required` (Landlock ABI 7 + seccomp), JIT do wasmtime 48 | timeline 2 261 eventos, YARA 0 casamentos, detecção 6 candidatos; idêntico ao de antes |

O engine real foi exercitado também por uma execução manual do protocolo (timeline, YARA e detecção) numa cópia do lock novo, antes da verificação deste pacote.

## Não feito

- Windows (Job Object) e aarch64 **não** foram repetidos com o lock novo (nenhum dos dois ambientes está disponível aqui).
- Não atualizados, de propósito (0.x, podem quebrar API): `eframe`/`egui`/`egui_extras`/`egui_kittest` 0.36.2 → 0.37.0, `evtx` 0.12.3 → 0.14.0, `serde_json_canonicalizer` 0.3.2 → 0.4.1 (esta alimenta a **cadeia de auditoria**: exige teste de compatibilidade dos hashes já gravados).

## Decisão sua

As 3 exceções informativas novas no `deny.toml`. Se preferir não aceitá-las, a alternativa é esperar um `eframe` que largue o `zbus` (o 0.37.0 existe; não foi verificado se resolve).
