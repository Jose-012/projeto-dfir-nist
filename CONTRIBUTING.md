<!-- doc-type: procedure | status: current -->
# Contribuindo

## Princípios

1. Alterações pequenas e rastreáveis.
2. Nenhuma arquitetura nova sem necessidade demonstrada.
3. Código sensível deve permanecer fácil de localizar e revisar.
4. Documentação e RTM mudam junto com o código quando afetados.

## Camadas

- Rust: caminho de produção confiável, incluindo a interface nativa (eframe/egui/egui_extras, `src/gui/`) e a camada de serviço que ela chama (`src/service/`).
- Python/Shell: QA/tooling.

## Antes de mudar

Leia `AGENTS.md`, `docs/DOCUMENT_AUTHORITY.md` e `docs/PROJECT_SPEC_v6_5_1.md`.

## Verificação local

```bash
just validate-fase0
just fmt
just check
just test
just clippy
```

Quando o ambiente oferecer as ferramentas de supply chain, execute também `just supply-chain`.

## Pull requests / revisão

Quando CI com proteção de merge existir, os gates configurados ali devem refletir `just verify` e os gates de supply chain definidos pelo projeto. Até lá, `justfile` e `docs/RELEASE_GATE.md` são a referência operacional.


Toolchains de referência: `toolchains.toml`.
