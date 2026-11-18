# 19 — Resultados vigentes sem janela, YARA idempotente, vínculo evidência↔incidente

**What to build:** (1) Consulta SQL de "vigentes" (`NOT EXISTS` sobre `supersedes`, filtro por incidente e tipo, sem `LIMIT 500`), usada por UI e exportação; (2) YARA: `supersedes` pelo vigente da mesma `correlation_key`, reexecução idempotente; (3) importação aceita `incident_id` e a GUI oferece seletor de incidente e de evidência (fim do UUID digitado).

**Blocked by:** F1-16.

**Status:** VERIFIED (2026-11-15, `EVD-010`) — vínculo evidência↔incidente implementado: `incident_id` na evidência, `link_evidence_to_incident` idempotente com auditoria `EVIDENCE_LINKED_TO_INCIDENT`, vigentes sem janela (`evidence_for_incident`), seletor de incidente no painel da evidência (GUI) e CLI `link-evidence`/`incident-evidence`; `confirm_candidate` vincula ao confirmar; YARA idempotente e vigentes sem `LIMIT 500` (AUD-07/08, 2026-10-04). Testes de serviço e `egui_kittest` próprios.

**Origem:** AUD-07, AUD-08, AUD-10, AUD-23, RF-039

- [ ] 600+ resultados no incidente aparecem completos na UI e na exportação.
- [ ] Dois scans YARA seguidos não duplicam resultados vigentes.
- [ ] Formulários de IOC/ATT&CK escolhem evidência por lista.
