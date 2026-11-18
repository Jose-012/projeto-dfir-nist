<!-- doc-type: decision | status: accepted -->
# ADR-0019 — CLI headless fino sobre a camada de serviço (sem TUI)

## Status

ACEITA — 2026-10-03. Não altera requisitos (RF) nem exige bump da especificação: é um adaptador de teste/CI sobre capacidades que já existem em `src/service/`. Complementa ADR-0017 (a GUI nativa continua sendo a interface do analista).

## Contexto

Até 2026-10-02 o único jeito de exercitar o pipeline real (importar → engine → timeline → YARA → auditoria → descarte) era pela GUI, que não roda em ambiente sem display. Resultado: os achados mais importantes da auditoria eram `[P]` (prováveis) por falta de execução, e a única verificação ponta a ponta possível era à mão pelo operador. O binário já tinha dois subcomandos de linha de comando (`audit-verify`, `engine-smoke`) e já depende de `clap` e `serde_json`.

## Decisão

1. Acrescentar subcomandos **headless, não interativos** a `dfir-app`: `import`, `evidence-list`, `analyze`, `results`, `findings` e `destroy` (e, desde a ADR-0023, `deliverables`). Saída: **JSON em stdout**; logs vão para **stderr**; falha = código de saída diferente de zero.
2. Regras de contenção:
   - adaptador **fino** sobre `service::*` e `application::engine`; nenhuma regra de negócio duplicada;
   - **nenhuma dependência nova** (`clap`, `serde_json` já presentes);
   - `destroy` exige `--confirm-filename` com o nome exato, como a GUI;
   - sem estado próprio, sem modo servidor, sem escuta de rede;
   - a CLI **não é requisito de aceitação** do MVP e não substitui a GUI.
3. **Sem TUI.** A demanda não pede; `ratatui`/`crossterm` adicionariam dependências e superfície de cadeia de suprimentos para um benefício que a CLI não interativa já entrega para testes.
4. Teste ponta a ponta reutilizável: `scripts/e2e_headless.py --evtx <arquivo>` (tooling de QA em Python, AGENTS §2) roda o binário real com o engine real num diretório de dados temporário e confere o contrato de cada etapa. Ele mede o estado **real**: etapas que dependem de correções ainda abertas (ex.: F1-18) aparecem como `FAIL` de verdade.

## Alternativas consideradas

- **Só testes de integração em Rust chamando `service::*`.** Continuam necessários (F1-16) e não são excluídos; mas não cobrem o binário, a inicialização do runtime nem o ambiente do processo. Os dois se complementam.
- **TUI completa.** Rejeitada (ver decisão 3).
- **Não fazer nada.** Rejeitada: o teste real encontrou, em minutos, um estouro de pilha no `import` e confirmou o limite de saída do engine (ver `docs/FASE1_STATUS.md`); nada disso era visível sem execução.

## Consequências

- Passa a existir um caminho automatizável de verificação ponta a ponta e uma base para o dataset de demonstração reproduzível (F1-11).
- O formato JSON dos comandos é o das structs de resposta do `service` (`EvidenceResponse`, `AnalyzeResponse`, `ResultsResponse`...): mudar essas structs muda o contrato da CLI; `scripts/e2e_headless.py` é o guarda.
  - **Nota (2026-11-04) — Gap origem: EVI-04.** `ResultsResponse` foi removido pelas ADR-0020 e ADR-0021; hoje `dfir-app results` devolve `TimelinePage` (`src/service/mod.rs`).
- O ponto de entrada sem subcomando continua abrindo a GUI.
