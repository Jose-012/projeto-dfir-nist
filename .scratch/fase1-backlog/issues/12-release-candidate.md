# 12 — Release candidate e reconciliação

**What to build:** Reconciliar a RTM com os RFs que permanecem MVP (inclui RF-048..RF-055), executar a demo offline ponta a ponta, publicar o documento de instalação de referência com proteção do volume (BitLocker/LUKS — RF-030) e o procedimento manual de atualização/rollback.

**Blocked by:** F1-07, F1-10, F1-11 e demais tickets do MVP.

**Status:** PARCIAL (ADR-0023, Etapa 7, 2026-10-23)

**Status vigente (2026-11-04, espelho de `docs/rtm.md`; Gap origem: INC-03):** PARCIAL — detalhe na linha F1-12 da RTM, que é a autoridade. O texto de status acima é histórico: F1-26 e F1-28 foram implementados e F1-27 foi adiado (atualização de 2026-10-27), e a cópia automática antes de migration existe (ADR-0026).

**Nota (2026-11-04) — Gap origem: EVI-05.** Demonstração executada com engine e EVTX reais em rustc/cargo 1.91.1 (registro acima); **não** verificada no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

**Origem:** RF-014, RF-030, RF-032, RF-047

- [ ] Nenhuma linha da RTM em `PENDING`, `BLOCKED` ou `STALE` para RFs MVP; cada RF com implementação, teste e evidência. Não atendido: F1-26, F1-27 e F1-28 seguem `PLANNED`; RF-032 automático não existe; nada está `VERIFIED` no toolchain pinado.
- [ ] `docs/DEMAND_TRACE.md` sem item `PLANEJADO` ou `PARCIAL` sem justificativa. Reconciliado em 2026-10-23; os `PARCIAL`/`PLANEJADO` restantes têm a lacuna nomeada e dependem de F1-26..F1-28.
- [ ] Demo ponta a ponta executada offline, com versões efetivas de rustc/cargo registradas. Executada com engine e EVTX reais em rustc/cargo 1.91.1 (registrado na ADR-0023); **não** medida com a rede cortada.
- [x] Documento de instalação de referência cobre proteção do volume e procedimento manual de atualização. (`docs/OPERATION.md`; comandos de LUKS e BitLocker não executados)
