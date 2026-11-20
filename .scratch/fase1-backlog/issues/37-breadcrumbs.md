# 37 — Breadcrumbs contextuais da interface nativa

**What to build:** Trilha textual que mostra seção, incidente/evidência e subvisão atuais, atualizada pelo estado real da navegação.

**Blocked by:** None.

**Status:** PLANNED (ativado pelo operador em 2026-11-20, ADR-0030 D-1/D-4).

**Origem:** RF-046 e seção 30.3; melhoria de engenharia, não requisito novo da demanda.

- [ ] Evidência selecionada mostra contexto e subpainel correto; incidente selecionado mostra incidente e aba.
- [ ] Trocar seleção, aba ou seção não deixa breadcrumb obsoleto.
- [ ] Rótulos têm wrapping e nomes acessíveis testados com `egui_kittest`, sem depender de imagens.
- [ ] Não se declara AT-SPI nem fluxo X11 verificado sem evidência própria.
