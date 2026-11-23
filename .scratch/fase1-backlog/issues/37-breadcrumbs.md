# 37 — Breadcrumbs contextuais da interface nativa

**What to build:** Trilha textual que mostra seção, incidente/evidência e subvisão atuais, atualizada pelo estado real da navegação.

**Blocked by:** None.

**Status:** VERIFIED em Linux x86_64 (2026-11-23, `EVD-059`; ADR-0030 D-1/D-4 e commit `63ac821`). A navegação X11 delimitada de `EVD-050` não é inspeção visual dos breadcrumbs nem AT-SPI.

**Origem:** RF-046 e seção 30.3; melhoria de engenharia, não requisito novo da demanda.

- [x] Evidência selecionada mostra contexto e subpainel correto; incidente selecionado mostra incidente e aba.
- [x] Trocar seleção, aba ou seção não deixa breadcrumb obsoleto.
- [x] Rótulos têm wrapping e nomes acessíveis testados com `egui_kittest`, sem depender de imagens.
- [x] Não se declara AT-SPI nem fluxo X11 verificado sem evidência própria.
