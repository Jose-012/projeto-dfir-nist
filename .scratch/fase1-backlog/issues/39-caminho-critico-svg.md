# 39 — Grafo SVG do caminho crítico

**What to build:** Renderer SVG determinístico a partir de `CriticalPathView`, com escaping XML, nós/arestas estáveis e alternativa textual.

**Blocked by:** F1-04 (já implementado; validar integração).

**Status:** IMPLEMENTED (2026-11-20, ADR-0030 D-1/D-4; `src/service/critical_path_svg.rs` e integração em `deliverables`).

**Origem:** RF-018, RF-046 e seção 30.3; melhoria de engenharia, não requisito novo da demanda.

- [x] Mesma entrada produz bytes idênticos; ordem de entrada não muda o grafo.
- [x] Texto hostil é escapado e não cria markup, links, scripts ou recursos externos.
- [x] Estágios e arestas válidos aparecem; arestas para destino ausente geram erro diagnóstico, não omissão silenciosa.
- [x] SVG entra no manifest de entregáveis com SHA-256 e tem cobertura de testes sem exigir leitura de imagem.
- [x] Lista textual continua disponível na GUI e nos documentos.

**Evidência local:** renderer 4/4 (432 skipped) e `deliverables` 41/41 (395 skipped). A gravação contém cinco Markdown, `caminho-critico.svg` e `manifest.json`; o manifest tem seis documentos e a auditoria inclui o hash do SVG. `just verify` e E2E comuns permanecem PENDING; nenhum teste declara renderização nativa, AT-SPI ou aceite externo.
