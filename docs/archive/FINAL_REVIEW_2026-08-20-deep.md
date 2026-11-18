<!-- doc-type: review | status: historical -->
> **Status: HISTÓRICO / SUPERSEDED.** Este documento registra uma revisão anterior e não é fonte normativa do runtime atual. Consulte `docs/PROJECT_SPEC_v6_4_5.md` e `docs/FASE0_STATUS.md`.

# Deep Review — 2026-08-20

## Escopo

Revisão estática de todo o runtime Rust, engine/protocolo, frontend TypeScript/Bun, Python, shell, PowerShell, migrations, configuração de container, documentação ativa e supply-chain metadata.

## Correções aplicadas nesta revisão

- Implementado o `run_process boundary_smoke_probe()` que era referenciado pelo engine, mas estava ausente.
- O smoke agora verifica escrita em `DFIR_ENGINE_OUTPUT_ROOT` e nega leitura/escrita no caminho proibido quando fornecido.
- A canonicalização JCS usada pela audit chain reaplica a validação de inteiros fora de `±2^53` durante a verificação/hash, não apenas no append.
- README/CHANGELOG foram sincronizados para `BLAKE3-256-v2-JCS` como formato atual.
- QA passou a exigir o smoke probe e o teste de runtime nativo Windows.
- `justfile` usa Python `python3` em Unix e `python` em Windows via `os_family()`.
- `release_gate.py` aceita `python3` ou `python`.
- `engine_protocol_smoke.py` usa o diretório do repositório como `cwd`.
- Adicionado `tests/integration/fase0_runtime.ps1` para runtime nativo Windows, evitando que o gate fique permanentemente em SKIP.
- Removidos artefatos `__pycache__/*.pyc` do pacote.
- Dependências analisadas: todas as dependências diretas ativas têm uso no código.

## Estado do lockfile e compilação

- `Cargo.lock` desta árvore já contém ``; a pendência documental anterior sobre lockfile desatualizado está superada.
- `cargo check --workspace --locked --all-targets` — PASS, conforme evidência fornecida pelo desenvolvedor no Windows em 2026-08-20.
- Os gates de testes, Clippy, Nextest e supply chain continuam sem execução registrada nesta atualização.

## Validações executadas nesta sessão

- `python3 -m py_compile scripts/*.py tests/security/*.py` — PASS
- `scripts/validate_fase0.py` — PASS estático, com pendência explícita de Cargo/Bun/lockfile
- `scripts/doc_gate.py` — PASS
- `tests/security/fase0_config.py` — PASS
- `tests/security/rejected_external_tools.py` — PASS
- TOML/JSON parsing — PASS
- dependências diretas verificadas por presença de uso no código — PASS
- busca de segredos reais — nenhum segredo encontrado
- busca de TODO/FIXME/HACK ativos — nenhum encontrado no runtime

## Validações ainda necessárias

- `cargo test --workspace --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo nextest run --workspace --locked`
- `bun run --cwd frontend build`
- `bun test frontend/src/app.test.ts`

## Automação customizada

A remoção do Minijinja não elimina scripts personalizados. Python, PowerShell e shell continuam adequados para checklists, fixtures, QA, preparação de evidências e automações auxiliares. O produto, porém, ainda não possui um DSL/runner de checklist de análise como funcionalidade de primeiro nível; a futura automação de análise deve passar pelo supervisor/process boundary, e não executar o parser de evidência diretamente.
