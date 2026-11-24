---
name: dfir-verify
description: Seleciona e executa gates DFIR/NIST, registra resultados reais e explicita verificações externas pendentes
allowed-tools: agentgrep, read, bash, todo, edit
---

# Verificação honesta no DFIR/NIST

Use esta skill antes de declarar uma mudança integrada ou verificada. Em sessões com mudanças ou gates, use-a junto com `dfir-improve`, que cobre o loop da sessão; esta skill cobre somente a verificação.

## Diagnóstico

- Rode `just doctor` quando o ambiente for relevante.
- Leia avisos como avisos. A referência de toolchain não prova a versão executada.
- Não instale ferramentas nem altere o ambiente apenas para fazer um gate parecer verde.

## Gates por mudança

```text
Documentação/contratos: just doc-gate
Código Rust:           just verify
Dependências:          just supply-chain
Fluxo com engine real: just e2e tests/fixtures/senai-demo.evtx
```

- `just verify` cobre fmt, check, nextest, doc-tests e clippy. `just test` é a opção explícita do runner padrão completo.
- Antes de mudar dependências, inspecione `cargo metadata --locked`, `cargo tree --locked -e features` e `cargo tree --locked -d`.
- Use os mesmos comandos do `justfile` e de `docs/RELEASE_GATE.md`. Não desabilite lint, testes ou advisories.
- Se uma ferramenta faltar, registre o gate correspondente como pendente ou bloqueado. Não o declare executado.

## Evidência e escopo

1. Registre comandos, resultados e contagens relevantes. Consulte `docs/EVIDENCE_REGISTER.md` antes de criar um `EVD-*` novo.
2. Não reutilize IDs normativos ou de evidência. `WAIVED` é exceção de governança, não prova de verificação.
3. Linux x86_64 não prova Windows nem Linux aarch64. Fixture DEMO não substitui incidente real.
4. Não leia dados de caso para validar uma tarefa que já tem fixture sanitizada adequada.
5. Rode `git diff --check` e revise `git status --short` para separar arquivos da tarefa de dados operacionais ou mudanças alheias.
6. Informe exatamente o que passou, o que falhou e o que ficou pendente. Não declare conclusão apenas porque o código parece correto.
7. Ao criar ou alterar um gate, lint ou teste de presença, execute também um controle negativo em cópia sanitizada do conteúdo rastreado: viole exatamente a condição nova, confirme a falha esperada e leia seu diagnóstico. Um PASS somente no caminho válido não prova detecção da regressão. Use `edit` somente nessa fixture em scratch, nunca para induzir falha no checkout real.
8. Exija caminho absoluto e leitura posterior do log de cada gate, incluindo warnings e contagens; `exit 0` e relatório de worker não bastam. Testes executados concorrentemente no mesmo checkout são `STALE` até uma repetição sequencial. Preserve no registro warnings que ocorreram antes da correção, sem convertê-los retroativamente em execução limpa.
9. Use `reproduced` somente para finding com controle executado, exit e diagnóstico compatíveis com a claim; leitura estática permanece `static_candidate`. Antes de corrigir shell, teste códigos distintos por etapa. Antes de alterar documentação de versões, separe estado vigente de histórico; antes de remover arquivo, confirme sua presença no inventário rastreado. Registre também hipóteses rejeitadas pelo controle, sem remediar código correto.

## Árvore limpa com artefatos preservados

Quando artefatos preservados no checkout alterarem o resultado de um gate, não os edite, mova ou apague e não relaxe o validador para acomodá-los.

1. Construa uma árvore com `git archive HEAD`, aplique um diff explicitamente delimitado aos arquivos da tarefa e copie cada arquivo novo pelo nome. Não use um `git diff` amplo que possa incluir trabalho alheio.
2. Confirme com `cmp` que a árvore executada corresponde ao changeset e que os artefatos preservados estão ausentes. Leia os logs e todos os diagnósticos de setup.
3. Um archive não contém `.git`: não apresente `git status` como pós-condição nessa árvore nem oculte `fatal: not a git repository`. A identidade da fonte é demonstrada pelo archive, pelo patch e pelas comparações.
4. O PASS vale somente para o HEAD+diff identificado, não para arquivos extras do checkout original. Se a cópia, o patch ou uma comparação falhar, o setup está `BLOCKED`, mesmo que um comando posterior retorne zero.
