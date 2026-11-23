# 35 — Observabilidade (logs em arquivo, diagnóstico) e testes de GUI (`egui_kittest`)

**What to build:** Logs JSON em arquivo com rotação, eventos com ids e durações (sem nomes de arquivo nem conteúdo), panics no log, `dfir-app diagnostics` e testes de GUI sem janela com `egui_kittest` (ADR-0022).

**Blocked by:** None.

**Status:** IMPLEMENTED / PENDING-HUMAN — compilado e testado em rustc 1.91.1 (não o pinado): 4 testes de rotação/permissão do log, 1 teste do diagnóstico (sem nomes de arquivo nem mensagens), 6 testes de GUI; verificado com o binário real (arquivo 0600 criado, eventos com duração, nome do arquivo ausente do log e do diagnóstico).

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado em rustc 1.91.1 (execução registrada no texto acima); **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

**Nota (2026-11-14) — DOC-01.** O pin atual é **1.99.1**; `just verify` de 2026-11-14 executou `cargo test --workspace` no 1.99.1, incluindo os testes de GUI com `egui_kittest` (`EVD-001`). A rodada também acrescentou testes dos caminhos de importação por caminho digitado e arrastar-e-soltar (GUI-01).

**Nota (2026-11-23) — EVD-062.** A apresentação UTC e as descrições contextuais de Evidências/Incidentes/Linhagem/Timeline passaram em 245 testes GUI, `just verify` com 713 passed/1 skipped, E2E 123/123 e fluxo X11 com Zenity/resize. O estado permanece `IMPLEMENTED / PENDING-HUMAN`; estética, AT-SPI, outras plataformas e produção não foram promovidos.

**Origem:** ADR-0022; limite encontrado ao testar a GUI sob Xvfb (teclado sintético não chegou ao campo de filtro).

- [x] Log em arquivo com rotação por tamanho, permissão 0600 em Unix, stderr mantido.
- [x] Eventos de importação, análise, indexação, descarte e recuperação com ids e `elapsed_ms`.
- [x] Panics registrados no log.
- [x] `dfir-app diagnostics` sem nomes de arquivo, caminhos de evidência nem mensagens de erro.
- [x] Testes de GUI da timeline com `egui_kittest` (digitar o filtro, clicar, páginas pedidas uma vez, avisos).
- [x] Rodar `cargo test` com o toolchain pinado (agora 1.99.1) e conferir `egui_kittest` (`EVD-001`).
- [ ] Mesmo padrão de teste de GUI para as demais telas (evidência, NIST, IOCs, ATT&CK).
- [ ] Fuzzing da validação de linha JSONL e do importador CSV/JSONL quando F1-27 existir.
