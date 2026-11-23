<!-- doc-type: decision | status: accepted -->
# ADR-0030 — Reintrodução delimitada de melhorias de inspeção, integridade e operação

## Status

ACEITA — 2026-11-20, por pedido explícito do operador na missão overnight de conclusão do MVP. Supersede parcialmente ADR-0018 D-2 somente quanto a assinatura de relatório, âncora extra de custódia, grafo SVG, breadcrumbs e cancelamento de job. Não reabre `cargo vet`, `.dfirpkg`, atualização offline, Ansible, importação CSV/JSONL ou demais escopos adiados.

## Contexto

ADR-0018 retirou os cinco itens do MVP por custo e prioridade. O operador agora pediu expressamente que eles sejam acrescentados às fases que os necessitam, junto com as fatias de `docs/MVP_IMPROVEMENT_PLAN.md`. Essa é uma decisão de arquitetura/escopo (nível 3), não um requisito novo da demanda SENAI e não uma evidência de implementação ou verificação.

A base atual já tem manifest de entregáveis com SHA-256, cadeia de custódia com hashes encadeados, modelo de caminho crítico, navegação nativa e supervisão do engine com `kill_on_drop`. Faltam, respectivamente, autenticidade destacada, detecção de remoção da cauda da custódia, exportação vetorial, contexto de navegação e cancelamento explícito e persistido.

## Decisão

### D-1 — Fases e rastreabilidade

- **Fase 1:** F1-36, âncora de cabeça/contagem da custódia (`RF-004`/`RF-029`); F1-37, breadcrumbs contextuais (`RF-046`); F1-38, cancelamento de job (`RF-037`/`RF-041`/`RF-042`).
- **Fase 2:** F1-39, SVG determinístico do caminho crítico (`RF-018`/`RF-046`).
- **Fase 3:** F1-40, assinatura destacada dos entregáveis (`RF-013`/`RF-015`/`RF-038`).
- As melhorias previamente propostas para timeline, histórico NIST, linhagem analítica, plano de hardening e regras EVTX ficam ativadas nos tickets existentes. RTK permanece Engineering/QA Tooling, sem ticket de produto.

Todos começam como trabalho a implementar. Nenhum estado `IMPLEMENTED`, `VERIFIED` ou `ACCEPTED` decorre desta ADR.

### D-2 — Assinatura destacada do conjunto de entregáveis

Como o pipeline atual gera Markdown e não PDF, a assinatura cobre o `manifest.json`, que por sua vez contém SHA-256 de cada documento e do SVG incluído. O algoritmo único é Ed25519 sobre os 32 bytes do SHA-256 do manifest, com artefato `manifest.json.sig` e identificador público derivado da chave. Preview nunca assina.

A chave privada deve ser escolhida pelo operador fora do armazenamento interno e da pasta exportada, com caminho canonicalizado e folha sem symlink/reparse point. Não há chave embutida, envio remoto, registro de segredo em log nem geração silenciosa. Gerar com assinatura exige verificação local bem-sucedida antes de auditar a conclusão. Uma CLI de verificação deve validar manifest, hashes e assinatura. A assinatura é integridade/autenticidade técnica, não declaração jurídica.

### D-3 — Âncora de custódia

Uma migration aditiva cria estado por evidência com contagem e hash da cabeça, preenchido a partir das entradas existentes. Cada append atualiza a entrada e a âncora na mesma transação. A verificação compara contagem, cabeça, sequência e hashes; remover a cauda sem ajustar a âncora passa a falhar.

A âncora local é tamper-evident, não inviolável: um atacante capaz de alterar banco e âncora ainda pode forjar ambos. Remoções anteriores à migration não podem ser inferidas retroativamente. Assinatura externa e backup permanecem controles distintos.

### D-4 — SVG, breadcrumbs e cancelamento

- O SVG usa o `CriticalPathView` como fonte, tem layout determinístico, escaping XML e alternativa textual existente. Não adiciona JavaScript, recurso externo, browser nem biblioteca gráfica de runtime.
- Breadcrumbs refletem o estado real da UI nativa e o incidente/evidência selecionados, sem imagens obrigatórias e sem afirmar suporte AT-SPI.
- O cancelamento usa um sinal interno por job, interrompe espera e processo supervisionado sem shell, persiste `CANCELLED` com código específico e auditoria/custódia, e não indexa resultados parciais como concluídos. Uma corrida já concluída permanece `COMPLETED`. Recovery de `PENDING`/`PROCESSING` continua inalterado.

## Alternativas consideradas

- **Assinar cada Markdown separadamente ou usar GPG externo.** Rejeitado: multiplica artefatos e adiciona runtime; a assinatura do manifest cobre o conjunto com um formato único e verificável offline.
- **Considerar a cadeia atual suficiente.** Rejeitado pelo teste de remoção da cauda, que hoje não é detectado.
- **Somente marcar o job como `FAILED`.** Rejeitado: apaga a distinção operacional entre cancelamento explícito e falha do engine.
- **Introduzir uma biblioteca SVG ou um novo serviço de fila.** Rejeitado por necessidade não demonstrada; o modelo existente e o processo supervisionado bastam.

## Consequências

A especificação, a RTM, os tickets F1-36..F1-40, a documentação de operação e os testes devem mudar junto com cada implementação. Novas migrations são históricas e não reeditam migrations anteriores. A dependência criptográfica, quando introduzida, exige `just supply-chain` e inventário atualizado.

Windows, Linux aarch64, incidente real e aceite de produção continuam bloqueadores independentes. Em 2026-11-23, `EVD-050` confirmou em Linux x86_64 o fluxo X11 delimitado de navegação, importação manual, análises, timeline e abrir/cancelar seletor; `EVD-060` acrescentou seleção real no Zenity e regressão manual no commit `370411f`. Não se declaram outros backends, AT-SPI, latência ou aceite visual. `EVD-059` registra os gates integrados do commit `63ac821`. Esta decisão não fecha F1-12 nem converte waivers em verificação.
