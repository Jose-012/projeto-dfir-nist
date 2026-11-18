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

## Regra de mudança mínima

Faça a menor alteração estrutural suficiente para atender ao requisito. Não introduza dependências, serviços, APIs, abstrações, bibliotecas ou funcionalidades extras sem necessidade demonstrada e validação correspondente.

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
just clippy
just supply-chain
```

Não declarar um gate como PASS sem execução real.

## Documentação e rastreabilidade

Toda alteração relevante atualiza, conforme aplicável (lista de `AGENTS.md` §11): `README.md`; `docs/FASE0_STATUS.md`; `docs/rtm.md`; `CHANGELOG.md`; `docs/adrs/`; `docs/RELEASE_GATE.md`; `docs/DOCUMENT_AUTHORITY.md`; `PROMPT_NOVA_CONVERSA.md`; índice/XML e Apêndice F da especificação; `docs/FASE1_STATUS.md` (log cronológico); `docs/FASE1_BACKLOG.md`; `docs/DEMAND_TRACE.md`; `docs/DEMO.md`; `docs/OPERATION.md`; `docs/DETECTION_RULES.md`; `docs/NATIVE_UI_ARCHITECTURE.md`; `docs/API.md`; `docs/DEFINITION_OF_READY.md`; `docs/SUPPLY_CHAIN.md`; `docs/TOOLCHAINS.md`; este arquivo; `RELATORIO_ALTERACOES_*`.

Preserve IDs normativos e de evidência. Não reutilize IDs existentes.

## Como o agente deve trabalhar

- Leia `AGENTS.md` e este arquivo antes de editar.
- Adicione ao contexto somente os arquivos necessários à tarefa.
- Trate documentos normativos como leitura, não como alvos de edição, salvo pedido explícito.
- Não commite alterações automaticamente.
- Não amplie silenciosamente o escopo para arquivos ignorados pelo Git.
- Não desabilite lint, Clippy, cargo-deny ou cargo-audit para fazer um gate passar.
- Não remova controles de segurança apenas para simplificar uma implementação.
- Ao encontrar conflito documental, pare a mudança e siga a hierarquia de autoridade.
- Ao concluir, apresente os arquivos alterados, os gates realmente executados e qualquer verificação pendente.

## Contexto sensível

Arquivos de evidência, bancos SQLite, artefatos de rede, EVTX, exports e diretórios operacionais não entram no contexto automático do agente: `storage/evidence/`, `storage/staging/`, `storage/sanitized/`, `storage/exports/`, `storage/quarantine/`, `storage/db/`, `reports/`, `exports/`, `.env` e `.env.*` (exceto `.env.example`), `*.sqlite`, `*.sqlite-shm`, `*.sqlite-wal`, `*.db`, `*.pcap`, `*.pcapng` e `*.evtx`. Artefatos de build (`target/`) e `.git/` também ficam de fora. Só devem ser adicionados explicitamente quando houver necessidade legítima e autorização para a tarefa.
