# 37 — Breadcrumbs contextuais da interface nativa

**What to build:** Trilha textual que mostra seção, incidente/evidência e subvisão atuais, atualizada pelo estado real da navegação.

**Blocked by:** None.

**Status:** IMPLEMENTED (2026-11-20, ADR-0030 D-1/D-4; `just verify` e fluxo X11 PENDING).

**Origem:** RF-046 e seção 30.3; melhoria de engenharia, não requisito novo da demanda.

- [x] Evidência selecionada mostra contexto e subpainel correto; incidente selecionado mostra incidente e aba.
- [x] Trocar seleção, aba ou seção não deixa breadcrumb obsoleto.
- [x] Rótulos têm wrapping e nomes acessíveis testados com `egui_kittest`, sem depender de imagens.
- [x] Não se declara AT-SPI nem fluxo X11 verificado sem evidência própria.
