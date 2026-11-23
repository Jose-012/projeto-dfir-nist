<!-- doc-type: backlog | status: current -->
# Backlog da Fase 1

Este arquivo lista somente trabalho remanescente. Estado de requisitos/tickets: `docs/rtm.md`. Execuções e waivers: `docs/EVIDENCE_REGISTER.md`. O plano histórico completo está em `docs/archive/FASE1_BACKLOG_plano_2026-10_2026-11.md`.

## Pendências

| Pista | Escopo restante | Estado |
|---|---|---|
| A. Verificação externa | Windows x86_64 (F1-22), Linux aarch64 e incidente real com sanitização | `PENDING-HUMAN`; Windows/aarch64 têm WVR-001/WVR-002 até 2026-12-31 |
| E. Supply chain | Repetir `just supply-chain` a cada mudança de dependência e revisar as exceções informativas bincode/chrono | local verde em `EVD-058` após `url 2.5.9`/`form_urlencoded 1.2.3` (621 dependências, 1560 advisories, sem exceção nova); anteriores `EVD-056`/`EVD-051` e CI `EVD-054`/`EVD-055`; revisões recorrentes continuam em `docs/SUPPLY_CHAIN.md` |
| F. Fechamento do RC | Resolver ou aceitar formalmente as pendências externas antes de declarar aceite final | bloqueado pela pista A; a pista G tem PASS delimitado em `EVD-050`/`EVD-060`, sem promover produção, outros backends ou demais plataformas |
| G. Fluxo nativo X11 | Manter o smoke pós-condicionado quando a UI afetada mudar; outros backends, AT-SPI, resize e latência são limites separados | `PASS` delimitado em Linux x86_64 por `EVD-050`/`EVD-060`: navegação, importação manual e por seleção Zenity, jobs, timeline e abrir/cancelar seletor |
| H. Melhorias ativadas | Fatias 1–6 de `docs/MVP_IMPROVEMENT_PLAN.md`; F1-36..F1-40 por ADR-0030 | `EVD-059` integra a árvore `63ac821` com verify 701/701, E2E 123/123 e X11 delimitado; F1-38/F1-40 também têm `EVD-056`. Revisão humana, eficácia real e bloqueadores externos não são promovidos |

Nenhuma pista autoriza afirmar verificação em ambiente não executado. Fixture DEMO, testes sintéticos e os dois datasets públicos externos de `EVD-052` não substituem incidente real autorizado e sanitizado.

## Critério de saída da pista A

Quando o operador fornecer cada ambiente, registrar versões, comandos, resultados e novos `EVD-*` sem incluir nome ou conteúdo de evidências:

- **Windows x86_64:** `just doctor`, `just verify`, `just e2e tests/fixtures/senai-demo.evtx` e `cargo run --locked --bin dfir-app -- engine-smoke`. Confirmar o aviso esperado `SANDBOX_NOT_FULLY_APPLIED`; não afirmar confinamento de arquivo/rede que ainda não existe nessa plataforma.
- **Linux aarch64 nativo:** repetir os mesmos comandos e os testes de sandbox do workspace. Registrar arquitetura, kernel e estado efetivo do Landlock/seccomp; compilação cruzada não substitui execução.
- **Incidente real:** usar cópia autorizada e sanitizada, preservar o original, registrar proveniência e executar o fluxo headless no volume protegido. Dados do caso, exports e logs brutos não entram no Git; a evidência registra somente métricas e resultados sanitizados.

Essas execuções continuam `PENDING-HUMAN` até que os ambientes e o caso existam. Waivers vigentes não contam como verificação.

## Critério de saída da pista G

Cumprido em Linux x86_64 nas atualizações de `EVD-050` e `EVD-060` de 2026-11-23. `scripts/native_x11_smoke.py` usou raízes vazias e fixture sanitizada, confirmando navegação Evidence ↔ Incidentes, importação pelo campo manual e por seleção real no Zenity, EVTX e detecção concluídos, página de timeline não vazia e abertura/cancelamento do seletor sem alterar o catálogo. O aceite veio de logs, SQLite e ciclo de vida da janela/processo, não do exit de injeção X11. O histórico BLOCKED/`BadMatch` de 2026-11-19 foi preservado.

Outros backends de diálogo, resize, AT-SPI, latência e desempenho não fazem parte desse PASS. Ao alterar a UI ou o harness, repetir o fluxo em raiz vazia e registrar nova execução sem apagar tentativas falhas.

## Fora do escopo corrente

Coleta ativa de rede, importação CSV/JSONL e formatos além de EVTX permanecem adiados por ADR-0025. Não reabrir esses itens sem decisão explícita do operador.

## Definition of Done de uma mudança

1. Requisito/decisão identificado e menor mudança suficiente implementada.
2. Testes e documentação afetados atualizados; RTM e especificação permanecem coerentes.
3. Gates aplicáveis executados com resultado registrado.
4. Ausência de verificação externa ou supply chain declarada explicitamente.
5. `WAIVED` não apresentado como `VERIFIED` e nenhuma falha relevante ocultada.
