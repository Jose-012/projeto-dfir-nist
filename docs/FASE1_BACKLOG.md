<!-- doc-type: backlog | status: current -->
# Backlog da Fase 1

Este arquivo lista somente trabalho remanescente. Estado de requisitos/tickets: `docs/rtm.md`. Execuções e waivers: `docs/EVIDENCE_REGISTER.md`. O plano histórico completo está em `docs/archive/FASE1_BACKLOG_plano_2026-10_2026-11.md`.

## Pendências

| Pista | Escopo restante | Estado |
|---|---|---|
| A. Verificação externa | Windows x86_64 (F1-22), Linux aarch64 e incidente real com sanitização | `PENDING-HUMAN`; Windows/aarch64 têm WVR-001/WVR-002 até 2026-12-31 |
| B. CI | Definir hospedagem remota e, se houver, pipeline no toolchain pinado (F1-14) | não implementado; WVR-003 até 2026-12-31; sem repositório remoto no momento |
| E. Supply chain | Repetir `just supply-chain` a cada mudança de dependência e revisar exceções com prazo | manual verde em `EVD-012`; revisões registradas em `docs/SUPPLY_CHAIN.md` |
| F. Fechamento do RC | Resolver ou aceitar formalmente as pendências externas e a decisão de CI antes de declarar aceite final | bloqueado pelas pistas A e B; estado normativo em `docs/RELEASE_GATE.md` |

Nenhuma pista autoriza afirmar verificação em ambiente não executado. Fixture DEMO e testes sintéticos não substituem incidente real.

## Fora do escopo corrente

Coleta ativa de rede, importação CSV/JSONL e formatos além de EVTX permanecem adiados por ADR-0025. Não reabrir esses itens sem decisão explícita do operador.

## Definition of Done de uma mudança

1. Requisito/decisão identificado e menor mudança suficiente implementada.
2. Testes e documentação afetados atualizados; RTM e especificação permanecem coerentes.
3. Gates aplicáveis executados com resultado registrado.
4. Ausência de verificação externa ou supply chain declarada explicitamente.
5. `WAIVED` não apresentado como `VERIFIED` e nenhuma falha relevante ocultada.
