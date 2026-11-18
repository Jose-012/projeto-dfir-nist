# 33 — Estado da GUI: correções de UX

**What to build:** Corrigir AUD-32 (YARA não pode sobrescrever o painel de timeline); garantir que todo `Command` do backend gere `Event` mesmo em panic (fim do spinner eterno); filtro de timeline por evidência; progresso por etapa quando confiável. Grafo SVG, breadcrumbs e cancelamento ficam fora do MVP.

**Blocked by:** F1-18.

**Status:** VERIFIED (2026-11-15, `EVD-010`) — AUD-32 corrigido: só análise EVTX recarrega a timeline (YARA não sobrescreve o painel com resultado vazio); teste próprio em `src/gui/evidence.rs`. Pendente residual (não bloqueia): filtro por evidência na timeline e progresso por etapa.

**Origem:** AUD-18, AUD-24, AUD-32, RF-046

- [x] Rodar YARA depois de EVTX não altera a timeline exibida (lógica testada; GUI não foi exercitada visualmente).
- [ ] Panic em tarefa do backend resulta em erro visível.
- [ ] Cada evidência mantém seu próprio filtro.
