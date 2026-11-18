# 20 — YARA: cobertura de EVTX e limites

**What to build:** Strings de BinXML em EVTX são UTF-16LE: acrescentar `wide ascii` aos padrões de PowerShell/Mimikatz e testar com fixture UTF-16 e com EVTX real. Definir política explícita para arquivo acima de `MAX_SCAN_BYTES` (falhar ou registrar truncamento; nunca silencioso) e validar `RLIMIT_AS` × mmap/wasmtime no Linux. Revisar o padrão ruidoso `-e `. Bump de `RULESET_VERSION`.

**Blocked by:** F1-16.

**Status:** VERIFIED (2026-11-15, `EVD-010`) — `wide ascii` provado em payload UTF-16LE real (teste novo `wide_modifier_matches_utf16le_payloads_only`), `scan_timeout_for` por tamanho com teto de 6 h, `MAX_MATCHES_PER_PATTERN` sem descarte silencioso (invariantes em `const {}`), `max_scan_size` removido (ADR-0020: marcador no fim de arquivo esparso de 300 MiB encontrado), `RLIMIT_AS` cobrindo o arquivo de entrada (ADR-0021). Pendente residual: revisão do padrão ruidoso `-e ` (cosmético, não bloqueia).

**Origem:** AUD-09, AUD-29

- [x] Regra casa termo em UTF-16LE de fixture (teste sintético; falta EVTX real).
- [ ] Arquivo acima do limite tem comportamento documentado e testado.
- [ ] Ruleset versionado e hash registrado no resultado.
