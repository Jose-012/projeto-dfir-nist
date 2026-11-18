---
name: dfir-verify
description: Seleciona e executa gates DFIR/NIST, registra resultados reais e explicita verificações externas pendentes
allowed-tools: agentgrep, read, bash, todo
---

# Verificação honesta no DFIR/NIST

Use esta skill antes de declarar uma mudança integrada ou verificada.

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
