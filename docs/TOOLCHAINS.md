<!-- doc-type: reference | status: current -->
# Toolchains de desenvolvimento — v6.5.1

## Produção

Nenhum runtime Node, Bun, Deno ou Python é necessário na imagem final.

## Interface nativa

A interface (`src/gui/`) é parte do binário `dfir-app`, compilada pelo mesmo toolchain Rust — não há toolchain separado. Dependências: `eframe`/`egui`/`egui_extras` 0.37.0 (com `default-features = false`, sem `accesskit`, então sem a cadeia zbus/AT-SPI no build) + `rfd` 0.18.0 (ver `docs/SUPPLY_CHAIN.md` para a árvore de licenças/advisories). O toolchain de frontend (Node.js/npm, `esbuild`, `vitest`) foi removido por inteiro junto com `frontend/` (ADR-0017; ADR-0009, sobre Bun→Node, fica só de valor histórico).

## Python

CPython 3.14.6 é a referência do tooling Python deste ciclo. A variante free-threaded não é exigida nem adotada por padrão; só pode ser introduzida mediante benchmark/necessidade demonstrada.

## Shell

Shell é usado para automação e testes de ambiente. Scripts devem usar `set -euo pipefail` quando aplicável, evitar interpolação insegura e nunca transformar entrada de usuário em comando privilegiado.

## Harness de agente (usado)

O harness de agente de desenvolvimento usado no projeto é o **Jcode** (declaração do operador em 2026-11-04; ADR-0027). Consta aqui apenas como ferramenta que foi usada: não é ferramenta obrigatória, não faz parte do runtime nem do build toolchain, e nenhum gate, receita ou documento depende dele. O harness não impede o commit automático nem a gravação do histórico de conversa (ADR-0027, atualização de 2026-11).

Fora desta seção e da ADR-0027, os documentos não nomeiam o harness.

## Rust: stable mais recente (1.100.0)

Decisão do operador (2026-11): o projeto usa a versão **stable mais recente** do Rust, não a 1.97.1. Hoje é a **1.100.0** (anunciada em 2026-11-12), fixada em `rust-toolchain.toml` e `toolchains.toml`; `scripts/doc_gate.py` e `scripts/validate_fase0.py` exigem que as duas fixações sejam a mesma versão exata (`X.Y.Z`). `rust-version = "1.100"` nos três `Cargo.toml` (raiz, engine e protocol) é o piso de compatibilidade, igual ao pin. Para trocar de versão, atualize os dois arquivos e os três manifests juntos.

## Obter o Rust sem `rustup` (registro de 2026-10-27; repetido com `rust=1.99.1` em 2026-11-09 e 2026-11-14; `rust=1.100.0` em 2026-11-18)

> Histórico: o texto abaixo registra a 1.97.1, o pin de 2026-10-27. O procedimento é o mesmo trocando a versão por `1.100.0`; a rodada de 2026-11-18 usou `rustc 1.100.0` diretamente.

Em um ambiente sem acesso a `static.rust-lang.org`, o toolchain pinado foi obtido do conda-forge e conferiu `rustc`, `cargo`, `clippy` e `rustfmt`:

```bash
micromamba create -y -p /opt/rust -c conda-forge --override-channels "rust=1.100.0"
PATH=/opt/rust/bin:$PATH cargo test --locked --workspace
```

Isto **não** substitui o `rustup` do operador; é só o caminho que funcionou onde o `rustup` não alcança. Para checar o engine contra Windows (type-check, sem executar): `micromamba install -p /opt/rust -c conda-forge rust-std-x86_64-pc-windows-gnu=1.100.0` e `cargo check --workspace --locked --target x86_64-pc-windows-gnu` (precisa de `x86_64-w64-mingw32-gcc`).

## Evidência

A versão efetiva de Rust/Cargo/Python usada em um gate deve ser registrada. Manifests pinam a referência; eles não provam que uma ferramenta foi executada. `just doctor` consulta as versões sem instalar ou alterar ferramentas; divergência de Python/nextest é aviso, ausência de Rust/Cargo/nextest ou Rust fora do pin é erro.

### Ambiente observado em 2026-11-18

Linux x86_64: rustc/cargo **1.100.0**, Python **3.13.5**, cargo-nextest **0.9.151**, cargo-deny **0.20.2**, just **1.59.0** e Git **2.47.3**. O Python real divergia da referência 3.14.6; os gates Python executados naquela rodada valiam para 3.13.5, não para 3.14.6. Esta observação histórica não foi reescrita.

### CPython alinhado em 2026-11-19

`EVD-049` registra CPython **3.14.6** instalado somente em scratch com uv 0.12.3. O archive teve SHA-256 conferido contra o release do GitHub e o executável instalado foi comparado byte a byte com o executável extraído do archive verificado. `validate_fase0.py` (0 erros/0 avisos), `doc_gate.py` (PASS) e `e2e_headless.py` (110/110) passaram com esse interpretador. O coordinator conferiu independentemente todos os hashes do manifesto. O `/usr/bin/python3` do host continua 3.13.5; a reprodução exige selecionar o caminho 3.14.6 documentado em `EVD-049`, não uma instalação global implícita. CPython continua fora do runtime de produção.


Toolchains de referência: `toolchains.toml`.

## Rust engine dependency policy

O parser EVTX do MVP é `evtx 0.15.1` com `default-features = false` (primeira versão que corrige `RUSTSEC-2026-0431`; `scripts/validate_fase0.py` exige `>= 0.15.1`). A geração definitiva de `Cargo.lock` continua obrigatória no ambiente Cargo de release; nenhum checksum de registry é inventado manualmente.

## Ferramentas de teste e verificação (2026-10-11)

- **`egui_kittest` 0.37.0** (dev-dependency): testes de GUI sem janela, pela árvore de acessibilidade. Exige o mesmo rustc do egui 0.37. O harness de teste usa `allow_missing_glyphs()` e nome acessível em cada widget (`labelled_by`/`from_label`), por causa das checagens novas do 0.37 (`src/gui/mod.rs`, `test_harness`).
- **`cargo-deny` 0.20.2** (binário oficial, fora do `Cargo.lock`): `cargo deny --workspace check`; o banco RustSec é baixado a cada execução.
- **GitHub Actions:** o workflow usa actions fixadas por SHA completo e ferramentas com versões exatas, checksum e `fallback: none`. Antes de trocar uma action ou versão de ferramenta, confira na tag exata da `install-action` que o manifesto contém a versão e o hash da plataforma; pin de SHA, sozinho, não prova essa disponibilidade (`EVD-054`).
- **Xvfb + `xdotool` + ImageMagick** (apenas verificação manual da GUI em ambiente sem display, **não** é gate): renderiza com software; cliques e rolagem funcionam, o teclado sintético não chega ao egui. Para digitar, usar `egui_kittest`.
- **Python** (`scripts/e2e_headless.py`): tooling de QA, nunca backend do produto.
