# 39 — Grafo SVG do caminho crítico

**What to build:** Renderer SVG determinístico a partir de `CriticalPathView`, com escaping XML, nós/arestas estáveis e alternativa textual.

**Blocked by:** F1-04 (já implementado; validar integração).

**Status:** PLANNED (ativado pelo operador em 2026-11-20, ADR-0030 D-1/D-4).

**Origem:** RF-018, RF-046 e seção 30.3; melhoria de engenharia, não requisito novo da demanda.

- [ ] Mesma entrada produz bytes idênticos; ordem de entrada não muda o grafo.
- [ ] Texto hostil é escapado e não cria markup, links, scripts ou recursos externos.
- [ ] Estágios e arestas válidos aparecem; arestas para destino ausente geram erro diagnóstico, não omissão silenciosa.
- [ ] SVG entra no manifest de entregáveis com SHA-256 e tem cobertura de testes sem exigir leitura de imagem.
- [ ] Lista textual continua disponível na GUI e nos documentos.
