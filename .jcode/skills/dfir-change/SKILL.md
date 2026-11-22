---
name: dfir-change
description: Guia mudanças pequenas e rastreáveis no projeto DFIR/NIST, com autoridade, segurança e documentação mínima
allowed-tools: agentgrep, read, edit, apply_patch, write, bash, todo
---

# Mudança controlada no DFIR/NIST

Use esta skill ao alterar código, contratos ou documentação deste repositório. Em sessões com mudanças ou gates, use-a junto com `dfir-improve`, que cobre o loop da sessão; esta skill cobre somente a mudança controlada.

## Procedimento

1. Leia `AGENTS.md` e `docs/DOCUMENT_AUTHORITY.md`.
2. Identifique o requisito ou ADR. Leia somente os documentos afetados:
   - estado e rastreabilidade: `docs/rtm.md` e `docs/DEMAND_TRACE.md`;
   - escopo: demanda original, especificação canônica e ADRs citadas;
   - operação ou gates: somente quando a mudança os afeta.
3. Classifique o alvo como Production Runtime, Build Toolchain ou Engineering/QA Tooling.
4. Rode `git status --short` e preserve mudanças que não pertencem à tarefa.

## Edição

- Faça a menor mudança suficiente. Não reescreva um arquivo inteiro para uma edição local.
- Releia o trecho atual antes de editar. Não suponha que uma leitura antiga ainda representa o arquivo.
- Não reduza o rigor de um teste para acomodar a implementação: trocar `run()` por `step()`, remover espera/pós-condição ou enfraquecer rótulo/assert exige justificativa comportamental e revisão do integrador. Corrija a causa da falha, não a consulta que a revelou.
- Antes de usar uma API nova ou incerta, confirme assinatura, unidade, tipo de retorno e comportamento na fonte da versão resolvida pelo `Cargo.lock` e na toolchain efetiva; código que compila em outra versão ou conversão de `bool` para `0/1` não prova animação nem comportamento equivalente. Catálogos externos de boas práticas Rust são hipóteses secundárias, não autoridade para refatoração, lint, otimização ou dependência nova.
- Não introduza dependência, serviço, camada, API ou documento permanente sem necessidade.
- Preserve a interface nativa, o processo separado do engine e o transporte stdin/stdout + JSONL.
- Não use shell intermediário em subprocessos de produção.
- Canonicalize caminhos contra a raiz confiável e preserve as checagens de symlink/reparse point.
- Preserve limites de execução, profundidade JSON, audit chain, backup e testes de segurança.
- Não registre secrets, nome de evidência ou conteúdo de eventos em logs e diagnósticos.

## Fechamento

1. Atualize testes e somente os documentos afetados. `docs/rtm.md` é a única matriz de estado; `docs/FASE1_STATUS.md` é histórico.
2. Se alterar a especificação canônica, mantenha `docs/PROJECT_SPEC_v6_5_1.md` e `nist-senai-proj_integrador_v6_5_1.md` byte a byte equivalentes.
3. Execute a skill `dfir-verify` ou os gates aplicáveis.
4. Faça um commit pequeno e temático somente após a validação, sem dados de caso, exports, bancos operacionais ou arquivos alheios à tarefa.
5. Relate arquivos, comandos, resultados e pendências. Não transforme `WAIVED` em `VERIFIED`.
