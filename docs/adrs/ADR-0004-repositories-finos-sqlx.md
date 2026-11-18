<!-- doc-type: decision | status: partially-superseded -->
> **Status:** SUPERSEDED BY ADR-0007 (2026-08-20).
> The following document is retained as historical design rationale and is not an active runtime requirement.

# ADR-0004

> **Status: PARTIALLY SUPERSEDED by ADR-0007.** User/authentication repositories described below are historical and are not part of the active local forensic runtime; the SQLx/SQLite repository principles remain applicable. — Repositories finos sobre SQLx, sem ORM

## Status
ACEITA — 2026-08-17

## Contexto
O projeto precisa reduzir repetição no acesso SQLite sem criar uma camada que esconda SQL, transações ou decisões de autorização. A especificação rejeita a adoção de um ORM obrigatório no MVP e prioriza rastreabilidade, operação offline e revisão explícita das queries.

## Decisão
Adotar repositories finos na camada `persistence`, implementados sobre SQLx:

- `SqliteUserRepository` para usuários e estado de autenticação;
- `SqliteSessionRepository` para sessões;
- `SqliteIncidentAccessRepository` para acesso por incidente.

Os repositories:

- mantêm SQL explícito e parametrizado;
- centralizam consultas de persistência repetidas;
- centralizam fronteiras de transação sensíveis;
- não contêm regras de autorização ou negócio;
- não geram SQL automaticamente;
- não introduzem entidade/unit-of-work/sessões automáticas de ORM;
- não introduzem novas dependências.

A auditoria continua em módulo próprio (`persistence/audit.rs`) porque possui invariantes específicas de cadeia de integridade.

## Consequências
Positivas:

- menos SQL espalhado por application/security/web;
- revisão de segurança mais localizada;
- transações de autenticação permanecem visíveis;
- nenhuma dependência ou runtime novo;
- mantém compatibilidade com SQLx e SQLite offline.

Negativas:

- ainda existe algum código de infraestrutura repetitivo;
- repositories podem crescer se não houver disciplina de escopo;
- consultas complexas continuam exigindo SQL manual.

## Critério de reavaliação
A adoção de ORM só será considerada se benchmark, volume de CRUD ou custo de manutenção demonstrar benefício concreto sem degradar segurança, rastreabilidade, compatibilidade offline ou controle das transações. Qualquer substituição exige novo ADR.
