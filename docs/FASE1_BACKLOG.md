<!-- doc-type: backlog | status: current -->
# Backlog da Fase 1

Este arquivo lista somente trabalho remanescente. Estado de requisitos/tickets: `docs/rtm.md`. Execuções e waivers: `docs/EVIDENCE_REGISTER.md`. O plano histórico completo está em `docs/archive/FASE1_BACKLOG_plano_2026-10_2026-11.md`.

## Pendências

| Pista | Escopo restante | Estado |
|---|---|---|
| A. Verificação externa | Windows x86_64 (F1-22), Linux aarch64 e incidente real com sanitização | `PENDING-HUMAN`; Windows/aarch64 têm WVR-001/WVR-002 até 2026-12-31 |
| B. CI | Definir hospedagem remota e, se houver, pipeline no toolchain pinado (F1-14) | não implementado; WVR-003 até 2026-12-31; sem repositório remoto no momento |
| E. Supply chain | Repetir `just supply-chain` a cada mudança de dependência e revisar exceções com prazo | manual verde em `EVD-014`; revisões registradas em `docs/SUPPLY_CHAIN.md` |
| F. Fechamento do RC | Resolver ou aceitar formalmente as pendências externas e a decisão de CI antes de declarar aceite final | bloqueado pelas pistas A e B; estado normativo em `docs/RELEASE_GATE.md` |

Nenhuma pista autoriza afirmar verificação em ambiente não executado. Fixture DEMO e testes sintéticos não substituem incidente real.

## Critério de saída da pista A

Quando o operador fornecer cada ambiente, registrar versões, comandos, resultados e novos `EVD-*` sem incluir nome ou conteúdo de evidências:

- **Windows x86_64:** `just doctor`, `just verify`, `just e2e tests/fixtures/senai-demo.evtx` e `cargo run --locked --bin dfir-app -- engine-smoke`. Confirmar o aviso esperado `SANDBOX_NOT_FULLY_APPLIED`; não afirmar confinamento de arquivo/rede que ainda não existe nessa plataforma.
- **Linux aarch64 nativo:** repetir os mesmos comandos e os testes de sandbox do workspace. Registrar arquitetura, kernel e estado efetivo do Landlock/seccomp; compilação cruzada não substitui execução.
- **Incidente real:** usar cópia autorizada e sanitizada, preservar o original, registrar proveniência e executar o fluxo headless no volume protegido. Dados do caso, exports e logs brutos não entram no Git; a evidência registra somente métricas e resultados sanitizados.

Essas execuções continuam `PENDING-HUMAN` até que os ambientes e o caso existam. Waivers vigentes não contam como verificação.

## Fora do escopo corrente

Coleta ativa de rede, importação CSV/JSONL e formatos além de EVTX permanecem adiados por ADR-0025. Não reabrir esses itens sem decisão explícita do operador.

## Definition of Done de uma mudança

1. Requisito/decisão identificado e menor mudança suficiente implementada.
2. Testes e documentação afetados atualizados; RTM e especificação permanecem coerentes.
3. Gates aplicáveis executados com resultado registrado.
4. Ausência de verificação externa ou supply chain declarada explicitamente.
5. `WAIVED` não apresentado como `VERIFIED` e nenhuma falha relevante ocultada.
