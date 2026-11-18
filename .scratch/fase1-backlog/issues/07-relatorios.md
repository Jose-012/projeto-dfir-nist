# 07 — Entregáveis: cinco documentos, um por entregável

**What to build:** Gerar os cinco documentos previstos pela spec §4.2 (seis requisitos da demanda empacotados em cinco), **um relatório por documento**, a partir do mesmo dataset persistido. Sem perfis executivo/técnico (ADR-0018): o resumo executivo é seção do Entregável 1. Exportação com manifest de hashes e opção de redação (F1-32). Formato do MVP: Markdown/JSON determinístico; PDF Typst somente se adotado por ADR própria, sem shell, com timeout e diretório controlado.

**Blocked by:** F1-25, F1-29, F1-30, F1-06, F1-32.

**Status:** IMPLEMENTED / PENDING-HUMAN (ADR-0023, 2026-10-16). A opção de redação (F1-32) e as lacunas de proveniência e retenção (F1-31) entraram na Etapa 6. Verificado em rustc 1.91.1, não no 1.97.1 pinado.

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado em rustc 1.91.1 (execução registrada no texto acima); **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

**Origem:** DEM-ENT-01..06, RF-013, RF-040, RF-046

- [x] Entregável 1 (relatório forense com custódia, linha do tempo, hipóteses, lacunas e resumo executivo), 2 (vulnerabilidades + ATT&CK), 3 (maturidade NIST por Function + recomendações), 4 (hardening), 5 (criptografia e proteção de dados) são gerados a partir de dados persistidos.
- [x] Cada afirmação cita a evidência/resultado de origem; lacunas de evidência (F1-31) aparecem explicitamente (itens sem base documentada, avisos de análise, falha de custódia, lacunas das conclusões, proveniência não informada e lacunas de fonte declaradas).
- [x] Preview antes de exportar; exportação inclui manifest com hash (SHA-256 e BLAKE3), versão da ferramenta e timestamp.
- [x] Render determinístico: mesma entrada gera a mesma saída (independente da ordem de entrada); falha de render falha o job.
- [x] Testes de injection/traversal em template (Markdown e HTML), nome de arquivo (constantes; id do incidente só com caracteres seguros) e caminho de saída (armazenamento interno, link simbólico, caminho ocupado por arquivo).
