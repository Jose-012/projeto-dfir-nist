# 22 — Isolamento do engine: ambiente mínimo, arquivo único, `kill_on_drop`

**What to build:** `env_clear()` com allow-list mínima (inclui as variáveis exigidas pelo Windows: `SystemRoot`, `TEMP`, `PATH` do engine); o engine passa a **usar** `DFIR_ENGINE_INPUT_FILE` e rejeitar qualquer outro arquivo da raiz; `kill_on_drop(true)`. Job Object e limite de memória no Windows ficam fora do MVP.

**Blocked by:** F1-16.

**Status:** IMPLEMENTED / PENDING-HUMAN (ADR-0024, 2026-10-27). Ambiente reduzido, arquivo único, `kill_on_drop`, Landlock + seccomp + `no_new_privs` (Linux) e Job Object (Windows). Linux provado com o engine real (`engine/tests/sandbox.rs`, `scripts/e2e_headless.py`); **Windows só compilado**; confinamento de arquivo/rede no Windows não feito.

**Origem:** AUD-15, RF-033, RF-037

- [x] Engine não lê outro arquivo da raiz de evidência (teste `the_engine_refuses_any_file_other_than_the_pinned_one`; Landlock nega o resto do disco).
- [x] Ambiente do app não vaza para o engine (teste com processo real e teste do `reduced_environment` para Windows).
- [ ] Engine sobe e analisa nos dois sistemas operacionais com ambiente reduzido.
