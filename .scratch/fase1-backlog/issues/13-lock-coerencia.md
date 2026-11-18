# 13 — Regenerar `Cargo.lock` e corrigir compilação

**What to build:** O lock estava em egui 0.33.3 com manifesto em 0.36.2 (todo alvo `--locked` falhava). Regenerar com o toolchain pinado, **commitar o lock**, e manter o gate estático de coerência manifesto × lock (já em `scripts/validate_fase0.py`). Também corrigir o erro de compilação E0505 em `src/persistence/nist_profiles.rs` (`previously_open_gaps`), introduzido pela correção de 2026-09-26 que nunca foi compilada.

**Blocked by:** None — pode começar imediatamente.

**Status:** IMPLEMENTED / PENDING-HUMAN — lock regenerado em cópia de trabalho (rustc 1.91 com `rust-version = 1.97` do workspace guiando a resolução) e E0505 corrigido; falta confirmar com rustc 1.97.1.

**Nota (2026-11-04) — Gap origem: EVI-05.** Lock regenerado e E0505 corrigido em rustc 1.91 (registro acima); **não** confirmado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

**Origem:** AUD-01, AUD-34, DOC-02, RF-047

- [ ] `just check` e `just test` verdes com `--locked` em Windows e Linux com rustc 1.97.1.
- [ ] `validate_fase0.py` falha se o lock divergir do manifesto.
- [ ] Docs citam as versões efetivas (egui 0.36.2, evtx 0.12.3 no lock).
