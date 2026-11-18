<!-- doc-type: decision | status: accepted -->
# ADR-0029 - Reavaliação da interface e manutenção de eframe/egui

## Status

ACEITA - 2026-11-18. Mantém a ADR-0017, sem alterar RF ou escopo.

## Contexto

O operador pediu uma revisão do repositório, das instruções e de toda a GUI para decidir se os problemas práticos justificavam trocar o framework. A auditoria usou Linux x86_64, janela cliente de 960×600 e dados isolados/sintéticos.

Os defeitos reproduzidos foram: conteúdo vertical sem rolagem, duas viewports concorrendo em Detecções, foco fora da área visível, hash só alcançável por tooltip, salts de `ComboBox` descartados, respostas atrasadas aplicadas ao incidente errado, loading sem evento após panic e exportação que podia truncar um arquivo existente. Nenhum deles exigiu trocar a biblioteca para ser corrigido.

A decisão vigente exige interface nativa no mesmo binário Rust, chamada direta a `src/service/`, operação local/offline, nenhum servidor HTTP ou toolchain de frontend, dependências justificadas e testes reproduzíveis. O hardware de referência também desencoraja uma reescrita sem ganho demonstrado.

## Alternativas consideradas

| Alternativa | Benefício possível | Custo e evidência ausente |
|---|---|---|
| Manter `eframe/egui` 0.37 | Conserva serviços, tabelas virtualizadas, backend assíncrono, `egui_kittest` e a árvore já validada | Exige disciplina de IDs, foco e viewports, aplicada nesta rodada |
| Slint | API Rust, componentes declarativos e propriedades de acessibilidade documentadas | Reescreve os módulos da GUI e seus testes, adiciona DSL/build/dependências e não tem protótipo ou validação AT-SPI neste projeto |
| GTK4 ou Qt | Widgets nativos e outra integração de acessibilidade | Nova stack de sistema/empacotamento, adaptação de threading e testes, revisão de supply chain e ausência de protótipo comparável |
| Iced ou outro toolkit Rust | Reescrita em Rust sem retornar ao navegador | Migração e validação equivalentes, sem evidência de que resolveria melhor os defeitos já corrigidos |
| Tauri, Electron ou Axum + frontend | Reuso de ecossistema web | Reintroduz frontend e fronteiras removidas pela ADR-0017, sem ganho necessário para os fluxos atuais |

Não foi executado benchmark entre frameworks. Portanto, nenhuma superioridade de desempenho é presumida.

## Decisão

Manter `eframe/egui/egui_extras` 0.37 e `rfd`. Não iniciar migração de framework.

As regras de implementação são:

1. Uma única viewport vertical por fluxo, salvo necessidade demonstrada. Detecções usa a viewport externa de Evidência.
2. Controles fora da viewport devem ser revelados ao ganhar foco, com regressão que comece fora da área visível.
3. IDs persistentes e nomes acessíveis são responsabilidades separadas. `enum_combo` usa `ComboBox::new(id, label)` e o teste estrito não tolera o glifo 🔥 do overlay de colisão.
4. Informação essencial não pode depender apenas de hover. O hash de auditoria tem tooltip e seleção acessível com o valor integral.
5. Todo comando do backend tem evento de erro roteado, inclusive após panic, para encerrar loading e exibir a falha.
6. Exportação JSON recusa destino existente com `create_new(true)`, sem `exists()+write()` e sem truncamento acidental.

## Limite de acessibilidade

A feature `accesskit` do `eframe`, que ativa as implementações de APIs de acessibilidade da plataforma, está deliberadamente desativada no grafo atual. A árvore interna usada pelo `egui_kittest` não é uma ponte AT-SPI e não comprova compatibilidade com Orca ou outro leitor de tela.

Esta decisão não declara suporte a leitor de tela externo, Windows, aarch64 ou outras plataformas não executadas. Se AT-SPI virar critério obrigatório de aceite, a decisão deve ser reavaliada. A comparação deverá incluir uma dessas opções:

- reativar a integração do `eframe` com versões seguras e uma revisão de supply chain;
- protótipo equivalente em Slint, GTK4 ou outra stack, com fluxo real, testes de teclado e leitor de tela externo.

A escolha entre essas opções exige evidência prática e não pode ser feita apenas pela lista de recursos do toolkit.

## Consequências

A camada `src/service/`, o engine, o protocolo e a estratégia de testes permanecem inalterados. As correções ficam pequenas e rastreáveis, sem nova dependência de produção. O projeto evita o risco de uma reescrita completa antes de concluir os fluxos do protótipo.

A limitação de AT-SPI permanece explícita e não é mascarada pela decisão de manter o framework.

## Evidência desta reavaliação

- `cargo test -p dfir-app --locked gui::`: 114/114 após recompilação limpa do pacote.
- Regressão de colisão de `ComboBox`: o panic 🔥 ocorreu antes da correção e o teste estrito passou depois, sem `allow_missing_glyphs`.
- Regressão de foco: o último controle começa fora da viewport e fica visível após foco.
- Backend: 4/4, incluindo panic roteado e preservação de arquivo existente na exportação.
- Incidentes: 8/8, incluindo tooltip e seleção acessível do hash.
- `cargo fmt --all -- --check`, `git diff --check` e clippy da GUI: verdes.
- Documentação externa consultada: feature `accesskit` em https://docs.rs/eframe/0.37.0/eframe/ e propriedades de acessibilidade em https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/.

A validação de capturas foi limitada ao contexto sintético e isolado. Após a sinalização de um provedor de visão, novas leituras de imagens por agentes foram interrompidas; nenhuma captura é necessária para sustentar esta decisão arquitetural.
