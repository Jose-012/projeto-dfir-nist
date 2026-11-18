# 29 — Item de plano (`PlanItem`)

**What to build:** Entidade ligada a Finding e a subcategoria NIST com `domínio` (`OS_HARDENING`, `REDE_DISPOSITIVOS`, `AUTENTICACAO_ACESSO`, `MONITORAMENTO`, `CRIPTO_REPOUSO`, `CRIPTO_TRANSITO`, `GESTAO_CHAVES`, `POLITICA_IR`), `prioridade`, `esforço`, `classe de custo`, `fase`, `dependências`, `referência` (CIS/NIST/regulatória/política interna), pré/pós-condição e como verificar; para `CRIPTO_*`: classe do dado, local, mecanismo, rotação e recuperação de chave. Uma entidade serve recomendações, hardening, criptografia e políticas de IR; substitui o modelo de recomendação só com IMMEDIATE/STRATEGIC.

**Blocked by:** F1-25.

**Status:** IMPLEMENTED / PENDING-HUMAN (ADR-0023, 2026-10-14). Migration `0010` e tipo em 2026-10-13; modelo, serviço e aba "Plano" em 2026-10-14. Absorve F1-06 (visão por domínio, sem Ansible). Verificado em rustc 1.91.1, não no 1.97.1 pinado.

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado em rustc 1.91.1 (execução registrada no texto acima); **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

**Origem:** RF-052, DEM-PLAN-01..07, RB-004, RB-005

- [x] As sete ações do plano da demanda têm um domínio cada (oito domínios; a criptografia se divide em repouso e trânsito) e a aba mostra a cobertura, inclusive domínios sem itens.
- [x] Prioridade, esforço, custo e fase obrigatórios (restrição RB-004).
- [x] Referência regulatória/política interna é campo próprio (RB-005).
- [x] Item sem evidência de suporte fica marcado como recomendação geral (`support_kind`).
