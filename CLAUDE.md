<!-- rtk-instructions v2 -->
# Command output

Command output here is condensed to save tokens, keeping every signal and
dropping costly noise. Treat it as the complete result: run commands
normally, and batch related commands into one call to avoid extra turns.
Truncated results state their recovery path in their own output. Re-run a
command as `rtk proxy <cmd>` only when its result is unusable: empty when
output was clearly expected, contradicting its exit code, or garbled.
<!-- /rtk-instructions -->

# Loop obrigatório de sessão

Leia `AGENTS.md` §16, `docs/SESSION_IMPROVEMENT.md` e as lições aplicáveis em `docs/SESSION_LEARNINGS.md` no início. Converta as lições em ações do plano e, no fechamento, registre somente lição nova, recorrência ou confirmação observada. Não antecipe evidências nem use exit 0 como substituto de pós-condição.

# Ferramentas preferidas do agente (ver AGENTS.md §15)

- Busque código com a ferramenta `agentgrep` (grep/find/outline/trace), nunca
  `grep -r` via bash; AST com `ast-grep` (não `sg`, depreciado).
- Leia arquivos com `read`, edite com `edit`/`apply_patch`/`write`; nunca
  `cat`/`sed`/scripts via bash para isso.
- Em bash inevitável: agrupe comandos (`batch` ou `&&`), use `rtk <cmd>` para
  saída condensada (`rtk proxy <cmd>` para recuperar o bruto), `fd`/`eza -T`
  para listar, `z` (zoxide) para navegar, `bat -p` para ler com paginação,
  `git diff` com delta. Temporários em `$JCODE_SCRATCH_DIR`.
- Detalhes e exceções (segurança do §6 prevalece): `AGENTS.md` §15.
