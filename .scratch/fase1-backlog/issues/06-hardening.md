# 06 — Plano de hardening (item de plano, sem Ansible no MVP)

**What to build:** Recomendações de hardening de SO/servidores como itens de plano (`PlanItem`, domínio `OS_HARDENING`) com referência CIS quando aplicável, pré-condição, pós-condição e como verificar. A demanda pede **plano** de hardening; a geração de playbook Ansible (RF-012) foi para a Fase 4 (ADR-0018).

**Blocked by:** F1-29 (modelo `PlanItem`), F1-25 (Finding).

**Status:** IMPLEMENTED / PENDING-HUMAN quanto ao modelo (ADR-0023, 2026-10-14): é o domínio `OS_HARDENING` de F1-29. A saída no Entregável 4 depende de F1-07.

**Origem:** DEM-PLAN-01, DEM-ENT-05, RF-052; RF-012 movido para Fase 4

- [x] Cada item de hardening liga a um Finding e/ou subcategoria NIST e traz rationale, referência (CIS/NIST), pré/pós-condição e como verificar.
- [x] Itens sem evidência de suporte são marcados como "recomendação geral", nunca como achado.
- [x] Nenhum artefato executável é gerado a partir de dados de evidência neste ticket.
- [ ] Saída entra no Entregável 4 (F1-07).
