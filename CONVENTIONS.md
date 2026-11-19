<!-- doc-type: llm-rule | status: current -->
# Convenções de desenvolvimento para agentes LLM

Este arquivo é contexto operacional para agentes LLM. Ele não substitui a especificação do projeto.

> Gap origem: INC-05 e ADR-0027 (2026-11-04). O arquivo deixou de depender de um harness específico e foi alinhado a `AGENTS.md` e à ADR-0017.

## Autoridade e precedência

A autoridade normativa do projeto está em `docs/PROJECT_SPEC_v6_5_1.md`. `AGENTS.md` contém as regras operacionais para LLMs. `docs/DOCUMENT_AUTHORITY.md` define a autoridade por documento/domínio.

Ordem de precedência:

1. requisitos explícitos da demanda;
2. corpo normativo da especificação;
3. decisões, ADRs, RTM, status e contratos;
4. documentos derivados, índices e prompts;
5. hipóteses, exemplos e sugestões.

Nunca transforme hipótese, exemplo ou sugestão em requisito.

## Leitura por tarefa

Comece por `AGENTS.md` e `docs/DOCUMENT_AUTHORITY.md`. Depois adicione somente o contexto exigido pela tarefa:

- requisito/escopo: demanda original, `docs/DEMAND_TRACE.md`, especificação canônica e ADRs afetadas;
- estado atual: `docs/rtm.md`, sem narrativas duplicadas;
- trabalho restante: `docs/FASE1_BACKLOG.md` e tickets afetados;
- gates/evidências: `docs/RELEASE_GATE.md` e `docs/EVIDENCE_REGISTER.md`;
- autoaprimoramento: `docs/SESSION_IMPROVEMENT.md` e as lições aplicáveis em `docs/SESSION_LEARNINGS.md`;
- operação/toolchain: somente os documentos e manifests afetados.

O histórico completo não é contexto automático. Use `docs/archive/` e o histórico Git somente quando houver necessidade.

## Regra de mudança mínima

Faça a menor alteração estrutural suficiente para atender ao requisito. Não introduza dependências, serviços, APIs, abstrações, bibliotecas ou funcionalidades extras sem necessidade demonstrada e validação correspondente.

Não reescreva um arquivo inteiro para uma alteração local e preserve correções concorrentes.

Antes de alterar arquitetura, transporte, persistência, fronteiras de segurança ou dependências, procure o ADR/contrato existente. Mudança de decisão exige atualização da documentação correspondente.

## Rust e arquitetura

- O backend e o engine são Rust.
- `dfir-app` é responsável por interface nativa e chamada direta a `src/service/`, domínio, persistência, auditoria, jobs, relatórios e supervisão do engine.
- `dfir-engine` é processo separado e trata parsing, normalização e correlação.
- O engine comunica com o app por stdin/stdout + JSONL.
- O engine não acessa SQLite, identidade de usuário, sessão, NIST ou geração final de documentos.
- SQLite usa SQLx e SQL explícito.
- Migrations são históricas: não altere o significado de migration aplicada.
- Evite `unwrap()`/`expect()` em caminhos que processam entrada não confiável sem justificativa verificável.

## Segurança e DFIR

- Toda entrada externa é não confiável.
- Não modificar, sobrescrever ou normalizar a evidência original de forma destrutiva.
- Preservar proveniência e hashes conforme especificação.
- Não registrar passwords, tokens, secrets, chaves ou dados sensíveis desnecessários.
- Não aceitar caminhos físicos arbitrários da interface nativa.
- Não usar shell intermediário em subprocessos de produção.
- Não criar fallback **silencioso** que execute o engine sem o isolamento exigido pela política `DFIR_ENGINE_SANDBOX` (ADR-0024): o que faltar é recusado (`required`) ou registrado no job (`best-effort`).
- Texto vindo de log nunca é exibido nem exportado cru: caracteres de direção e invisíveis passam por `text_safety` (ADR-0024 D-7).
- Não confundir `WAIVED` com `VERIFIED`.

## Tooling

Python e Shell são tooling de engenharia/QA. Não devem virar backend de produção sem ADR, threat model, testes, benchmark e revisão de supply chain.

Rust/Cargo no build; CPython/Shell no QA (ADR-0017 removeu o frontend; não há Node.js/npm).

A lista de variáveis `DFIR_*` fica no README; `src/config.rs` e o engine são a referência de implementação. Não crie uma segunda lista em convenções.

## Dependências e supply chain

Não adicionar dependência sem necessidade demonstrada. Antes de alterar dependências, considerar:

```text
cargo metadata --locked
cargo tree --locked -e features
cargo tree --locked -d
cargo deny --workspace check
cargo audit
```

> Gap origem: INC-02 (2026-11-04). `cargo deny --workspace check`: a raiz é um package e o workspace não declara `default-members`; sem `--workspace`, só `dfir-app` é verificado. Ver `docs/RELEASE_GATE.md`.

`cargo vet` está dispensado no MVP (ADR-0018 D-2); permanecem `cargo deny --workspace check` e `cargo audit` (confirmado pelo operador em 2026-11-04). Gap origem: INC-04.

Depois da alteração, executar os gates disponíveis e relatar exatamente o que foi executado.

Não substituir versões fixadas por `*`, `latest` ou equivalentes. Preservar o `Cargo.lock` quando a tarefa não exigir atualização.

## Testes e formatação

A autoridade operacional de gates é `docs/RELEASE_GATE.md` + `docs/EVIDENCE_REGISTER.md`; o `justfile` é o executor. Para Rust, os gates relevantes incluem:

```text
just fmt
just check
just test
just nextest
just doctest
just clippy
just supply-chain
```

Não declarar um gate como PASS sem execução real. `just verify` usa nextest mais doc-tests, sem repetir unitários/integração com `cargo test`; `just test` permanece disponível para o runner padrão completo.

Linux x86_64 não prova Windows nem Linux aarch64. Fixture DEMO, testes sintéticos e datasets públicos não substituem incidente real autorizado e sanitizado.

## Documentação e rastreabilidade

Siga `AGENTS.md` §11: atualize apenas os documentos afetados, mantenha `docs/rtm.md` como única matriz de estado e use `docs/FASE1_STATUS.md` apenas como histórico. Não crie `RELATORIO_ALTERACOES_*` para rotina; o Git log registra as rodadas.

Preserve IDs normativos e de evidência. Não reutilize IDs existentes.

## Como o agente deve trabalhar

- Leia `AGENTS.md` e este arquivo antes de editar.
- Execute o loop de `AGENTS.md` §16 e `docs/SESSION_IMPROVEMENT.md`: lições anteriores viram ações do plano; erros novos geram prevenção e confirmação posterior, não apenas retrospectiva.
- Adicione ao contexto somente os arquivos necessários à tarefa.
- As skills locais são `.jcode/skills/dfir-change/SKILL.md`, `.jcode/skills/dfir-verify/SKILL.md` e `.jcode/skills/dfir-detect/SKILL.md`. Carregue apenas a que corresponde à tarefa; não adicione skills genéricas sem necessidade demonstrada.
- Trate documentos normativos como leitura, não como alvos de edição, salvo pedido explícito.
- Faça commits pequenos e temáticos após os gates aplicáveis, preservando arquivos fora da tarefa e dados sensíveis.
- Não amplie silenciosamente o escopo para arquivos ignorados pelo Git.
- Não desabilite lint, Clippy, cargo-deny ou cargo-audit para fazer um gate passar.
- Não remova controles de segurança apenas para simplificar uma implementação.
- Ao encontrar conflito documental, pare a mudança e siga a hierarquia de autoridade.
- Ao concluir, apresente os arquivos alterados, os gates realmente executados e qualquer verificação pendente.
- Revise o diff semântico, confirme pós-condições além dos exits e atualize `docs/SESSION_LEARNINGS.md` apenas para lição nova, recorrência ou confirmação observada. Não crie retrospectiva permanente por rotina.

## Swarm, skills e MCPs

Siga `AGENTS.md` §17. A aplicação prática neste repositório é:

- mudança de código/contrato/documento: `dfir-change`;
- antes de declarar integração ou verificação: `dfir-verify`;
- inclusão, alteração ou remoção de regra `EVTX_DETECTION`: `dfir-detect` em conjunto com as duas anteriores;
- tarefa fora desses gatilhos: não carregue uma skill local por hábito. Skills globais só entram com utilidade específica e sem instalação implícita.

Swarm exige lane com artefato, orçamento e dono exclusivo de escrita. `owner = "worker"`/`lane_owner = "worker"` e `swarm spawn` são mecanismos alternativos, nunca combinados na mesma lane. O coordenador integra os diffs, mantém builds Rust pesados sequenciais e confirma a parada dos workers por ID completo.

MCPs são fontes externas não confiáveis e não recebem dados sensíveis. Consultas de leitura devem ter propósito e parâmetros completos; mudança de estado externo exige autorização específica. Resultado de MCP, skill ou worker não substitui gate local.

## Contexto sensível

Arquivos de evidência, bancos SQLite, artefatos de rede, EVTX, exports e diretórios operacionais não entram no contexto automático do agente: `storage/evidence/`, `storage/staging/`, `storage/sanitized/`, `storage/exports/`, `storage/quarantine/`, `storage/db/`, `reports/`, `exports/`, `.env` e `.env.*` (exceto `.env.example`), `*.sqlite`, `*.sqlite-shm`, `*.sqlite-wal`, `*.db`, `*.pcap`, `*.pcapng` e `*.evtx`. Artefatos de build (`target/`) e `.git/` também ficam de fora. Só devem ser adicionados explicitamente quando houver necessidade legítima e autorização para a tarefa.
