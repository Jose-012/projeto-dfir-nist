<!-- doc-type: review | status: historical -->
> **Status: HISTÓRICO / SUPERSEDED.** Este documento registra uma revisão anterior e não é fonte normativa do runtime atual. Consulte `docs/PROJECT_SPEC_v6_4_5.md` e `docs/FASE0_STATUS.md`.

# Revisão final — 2026-08-19

## Resultado

O projeto foi revisado nas camadas de protocolo, engine, autenticação, auditoria, persistência, process boundary, supply chain e tooling. Foram corrigidos problemas visíveis de lógica e consistência.

## Correções principais

- Autenticação não transforma senha curta em erro interno; entradas acima do limite usam o hash dummy e retornam falha genérica.
- `max_output_bytes` é aplicado ao mesmo conjunto de mensagens em app e engine (`RESULT`) e inclui o byte de newline em ambos os lados.
- Batches de resultados são ajustados dinamicamente para nunca exceder `MAX_LINE_BYTES`; um item isolado grande é rejeitado explicitamente.
- Auditoria usa `BLAKE3-256-v2-JCS` para novos eventos, com encoding por campos com presença/comprimento explícitos. Eventos BLAKE3-256-v1 continuam verificáveis; hashes SHA-256 não fazem parte da cadeia de auditoria atual.
- O engine continua com isolamento obrigatório e sem fallback direto.

## Dependências

As dependências diretas encontradas em uso permanecem justificadas. `tokio-util` e `futures-util` são usados diretamente no framing/streaming; `askama` serve web e CLI; MiniJinja foi removido após a consolidação dos templates internos; `sha2` continua necessário para compatibilidade com hashes de auditoria legados; `blake3` é usado diretamente na cadeia de auditoria.

Não foram adicionadas dependências de baixo nível sem necessidade, como `mio`, `zerocopy` ou `rayon`.

## Toolchain frontend

Bun é a toolchain de build/test do frontend. O frontend continua sem runtime JS no produto final; Bun produz os assets estáticos e executa os testes do browser layer.

## Validação possível neste ambiente

- `scripts/validate_fase0.py`: PASS — 0 erros, 0 warnings.
- `scripts/doc_gate.py`: PASS.
- Python scripts: PASS (`py_compile`).
- Shell scripts: PASS (`bash -n`).
- `Cargo.lock`: TOML válido; checksum de `blake3 1.8.6` confere com a publicação oficial.

## Validação ainda dependente de ambiente

Este ambiente não possui `cargo`, `rustc` ou `bun`. Portanto não é possível declarar como executados aqui `cargo check`, `cargo test`, `cargo clippy`, build/transpile do frontend ou testes reais de runtime.


## Recomendação de fase

Vale a pena congelar esta versão como candidata a Fase 1 e iniciar testes reais. A Fase 1 deve ser usada como ciclo de teste/refino, não como autorização para adicionar funcionalidades analíticas indiscriminadamente. Os primeiros testes devem concentrar-se em compilação, auditoria, streaming/limites, isolamento Linux, persistência concorrente e UX local.
