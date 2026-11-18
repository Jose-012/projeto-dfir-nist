# 24 — Custódia e auditoria visíveis na interface

**What to build:** Telas de custódia por evidência, auditoria filtrável, histórico de jobs (inclui falhos) e botão "Verificar integridade"; `audit-verify` roda na partida e mostra o resultado; `actor` registra usuário/host do sistema operacional. Assinatura e âncora extra de custódia ficam fora do MVP.

**Blocked by:** F1-16.

**Status:** VERIFIED (2026-11-15, `EVD-010`) — custódia e auditoria visíveis: painel "Vínculo com incidente e custódia" no detalhe da evidência (tabela de custódia), abas **Evidências** e **Auditoria** na aba Incidentes (GUI), CLI `custody <EVIDENCE_ID>` e `audit [--limit N]`; 4 testes `egui_kittest` novos (suíte `gui::` 62/62).

**Origem:** AUD-16, AUD-17, RF-004, RF-029

- [ ] Analista consulta custódia, auditoria e jobs na GUI.
- [ ] Verificação de cadeia é acionável sem CLI.
- [ ] `actor` inclui usuário e host do SO.
