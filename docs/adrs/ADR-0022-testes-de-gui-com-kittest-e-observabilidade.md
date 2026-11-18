<!-- doc-type: decision | status: accepted -->
# ADR-0022 — Testes de GUI com `egui_kittest` e observabilidade (logs em arquivo, spans, diagnóstico)

## Status

ACEITA — 2026-10-11, em resposta a duas perguntas do operador: "deveria incluir eguitest e outras formas de teste?" e "logs/telemetria mais específicos para melhor debug futuro?". Decisão de arquitetura (nível 3); não altera RF.

## Contexto

1. **Testes de GUI.** Sob Xvfb a GUI renderizou e cliques/rolagem funcionaram, mas o teclado sintético não chegou à caixa de filtro: digitar o filtro **não pôde ser verificado**. Xvfb + `xdotool` também é frágil (depende de coordenadas, de captura de tela e de um servidor X) e não roda em CI sem display.
2. **Observabilidade.** Havia 11 chamadas de `tracing` no app inteiro, só em JSON no stderr (invisível numa GUI sem console), sem arquivo, sem duração das etapas, sem registro de panics e sem forma de pedir um diagnóstico a quem usa a ferramenta.

## Decisão

### Testes de GUI: sim, `egui_kittest`, e só ele

- `egui_kittest` 0.36.2 como **dev-dependency** (não entra no binário). Renderiza a tela sem janela e opera pela árvore de acessibilidade (clicar, digitar, consultar rótulos), como um leitor de tela. Roda em `cargo test`, sem display, sem GPU.
- 6 testes em `gui/timeline.rs` (`ui_tests`): digitar o filtro e clicar em "Filtrar" envia **um** pedido da página 0 com o filtro sem espaços e `want_total`; clicar com o texto inalterado não consulta o banco; os avisos do parser aparecem; uma página ausente é pedida **uma vez** ao longo de vários quadros; o estado vazio orienta o analista; respostas de filtro ou job antigos são descartadas. Suporte de teste: `Backend::for_test()` e `TimelinePage::for_test()` (`#[cfg(test)]`).
- **Fora de escopo:** testes de snapshot de imagem (exigem GPU/`wgpu` e geram arquivos binários a manter) e o resto das telas (evidência, NIST, IOCs…), que ficam como trabalho futuro com o mesmo padrão.

### Outras formas de teste: o que já existe e o que NÃO entra

- Já existem: unitários, `scripts/e2e_headless.py` (engine real + EVTX real, 38 verificações), testes de propriedade (`proptest`) no protocolo, e o gate estático em Python.
- **Não entram agora:** fuzzing (`cargo-fuzz`/libFuzzer: o parser EVTX é de terceiros, e o que o projeto controla — validação de linha JSONL e do importador CSV/JSONL futuro, F1-27 — é a superfície certa para fuzzar **quando o importador existir**); testes de mutação (`cargo-mutants`: custo alto, retorno baixo agora); cobertura (`cargo-llvm-cov`: útil como número, não como critério); testes de snapshot de GUI (ver acima). Registrados como candidatos, sem ticket.

### Observabilidade (sem telemetria remota)

A ferramenta é offline e lida com evidência: **não há telemetria enviada a lugar nenhum** (RB-006). Tudo é local.

1. **Logs em arquivo** (`infrastructure/logging.rs`): `<DFIR_DATA_ROOT>/logs/dfir.log`, JSON, rotação por tamanho (5 arquivos de 5 MiB), permissão 0600 em Unix, e continuam no stderr (CLI). Filtro por `RUST_LOG` (padrão `info`). A configuração é lida antes do `tracing` para saber onde gravar.
2. **Eventos com duração e ids**: importação (id, tamanho, formato, `elapsed_ms`), análise (evidência, job, tipo, itens, avisos, duração total; ou o erro), indexação da timeline (itens, ignorados, duração), descarte, recuperação de jobs órfãos e, em nível `debug`, cada página de timeline servida. **Nunca** o nome do arquivo de evidência nem conteúdo de eventos (testado com o binário real).
3. **Panics** vão para o log (antes só para o stderr).
4. **`dfir-app diagnostics`**: JSON com versão/OS/arquitetura, configuração relevante, migrations aplicadas, contagem de evidências e de jobs por estado, jobs com avisos, as 10 últimas falhas **por código**, espaço livre e arquivos de log. Sem nomes de arquivo, caminhos de evidência nem mensagens de erro (testado).

## Riscos e limites

- O texto dos erros vai para o log local e pode conter caminhos do próprio sistema do operador; o `diagnostics` não os inclui, mas o arquivo `dfir.log` sim. Quem enviar o log a terceiros deve revisá-lo.
- `egui_kittest` exige rustc 1.95 (como o egui 0.36); aqui compilou em 1.91.1 com as cópias locais descartáveis já descritas. Adiciona 6 pacotes ao `Cargo.lock` (`egui_kittest`, `kittest`, `toml`, `serde_spanned`, `toml_datetime`/`toml_parser` atualizados); `cargo deny check` continua verde.
- A API do `kittest` é jovem (0.4): atualizações do egui podem exigir ajustes nos testes.
