<!-- doc-type: review | status: historical -->
> **Status: HISTÓRICO / SUPERSEDED.** Este documento registra uma revisão anterior e não é fonte normativa do runtime atual. Consulte `docs/PROJECT_SPEC_v6_4_5.md` e `docs/FASE0_STATUS.md`.

# Implementação — Bun + process boundary Windows — 2026-08-19

## Implementado

- Bun 1.3.14 formalizado como toolchain de build/test do frontend.
- `frontend/build.ts` usa `Bun.build()` para browser e gera IIFE compatível com os templates server-rendered.
- `bun:test` cobre o parser/normalizador da timeline.
- Deno removido da toolchain ativa e do artefato frontend.
- `ProcessMode` separa execução normal de smoke.
- Ambiente Windows do engine é reduzido aos vars essenciais e `DFIR_ENGINE_INPUT_ROOT`; o root físico não é exposto pelo protocolo.
- `input_reference` do engine é relativo à raiz de entrada do job; o engine não conhece o storage global.
- `engine-smoke` testa process boundary + IPC/protocolo sem depender de um arquivo EVTX artificial.
- Teste Windows inclui negative control para leitura fora da raiz e smoke end-to-end do `dfir-app`.
- Corrigida uma duplicação de erro em `engine.rs` e imports condicionais para evitar warnings Windows.

## Dependências

- `blake3 1.8.6` permanece o algoritmo principal de novos hashes.
- `tokio-util 0.7.19` fornece framing JSONL.

## Validação local nesta sessão

PASS:

- `python3 scripts/validate_fase0.py` — 0 erros / 0 warnings.
- `python3 -m py_compile scripts/*.py`.
- TOML/JSON da árvore revisada permanecem parseáveis pelos gates existentes.
- cópias canônicas da especificação permanecem idênticas.

PENDENTE e não mascarado como PASS:

- `cargo check/test/clippy/fmt` reais, pois o ambiente desta sessão não possui Cargo/Rust.
- `bun build`/`bun test` reais, pois o ambiente não possui Bun.
- `cargo audit/deny/vet/cyclonedx` reais.

## Lockfile

O `Cargo.toml` agora possui uma dependência Windows-only nova (``). O `Cargo.lock` desta árvore ainda precisa ser regenerado por Cargo no ambiente com registry disponível. Não foi inventado checksum nem entrada transitiva manualmente. O primeiro passo em Windows/CI deve ser `cargo check --workspace`; depois os gates `--locked` devem ser repetidos.
