# 18 — Timeline persistida, ordenada no banco, paginada e filtrável

**What to build:** Persistir eventos normalizados (F1-26) com índices por tempo/evidência/Event ID/host/usuário; ordenar **no banco**, nunca depois de truncar; leitura por cursor. Eliminar `MAX_TIMELINE_EVENTS=1000` como corte de dados. Saída do engine por lote ou teto configurável com falha explícita. Filtros: intervalo, tipo, host, usuário, texto, evidência (cada evidência com seu próprio filtro).

**Blocked by:** F1-26.

**Status:** IMPLEMENTED / PENDING-HUMAN (ADR-0020/0021; filtros estruturados em 2026-10-27, ADR-0025).

**Origem:** AUD-05, AUD-06, AUD-18, DEM-TL-01, RF-007, RF-046

- [ ] EVTX com mais de 100 mil eventos exibe o primeiro evento cronológico correto.
- [ ] Filtros por intervalo, Event ID, host e usuário funcionam; texto livre continua.
- [ ] Falha de limite é explícita, nunca truncamento silencioso.
