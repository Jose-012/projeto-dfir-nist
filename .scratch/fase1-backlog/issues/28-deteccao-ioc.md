# 28 — Regras determinísticas de detecção e extração de IOC

**What to build:** Conjunto pequeno e versionado de regras sobre eventos normalizados: rajada de logon falho, logon tipo 3/10 de origem nova, uso de privilégio especial, criação de serviço/tarefa, criação/alteração de conta e grupo, limpeza de log, PowerShell codificado, regra de encaminhamento de e-mail, login de origem incomum. Cada regra gera **candidato** com ATT&CK sugerida e evento de origem; nada vira achado sem confirmação do analista. IOCs (IP, domínio, URL, e-mail, hash, caminho, host) extraídos dos eventos com referência de origem.

**Blocked by:** F1-26, F1-25.

**Status:** IMPLEMENTED / PENDING-HUMAN (ADR-0025, 2026-10-27). 19 regras (`docs/DETECTION_RULES.md`), candidatos com confirmação ou rejeição motivada; as regras de e-mail/identidade na nuvem dependem de F1-27 (adiado). Verificado no `security.evtx` real e em eventos sintéticos, não em incidente real.

**Origem:** RF-050, DEM-ID-01, DEM-ID-03, DEM-ID-04, DEM-ID-05, DEM-FONTE-03

- [x] Cada regra tem fixture positiva e negativa (`engine/src/detect.rs`, módulo de testes; mais o `security.evtx` real).
- [x] Candidato mostra regra, versão, evento de origem e ATT&CK sugerida (tela, CLI e teste de UI).
- [x] Confirmar ou rejeitar candidato é auditado (`DETECTION_CANDIDATE_CONFIRMED`/`_REJECTED`); rejeitado não vira achado (teste e e2e).
- [x] IOC extraído aponta o evento de origem (`event_ref` em cada IOC; vira o contexto do IOC ao confirmar).
