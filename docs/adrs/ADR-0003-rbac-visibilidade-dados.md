<!-- doc-type: decision | status: superseded -->
> **Status:** SUPERSEDED BY ADR-0007 (2026-08-20).
> The following document is retained as historical design rationale and is not an active runtime requirement.

# ADR-0003 — Separação entre RBAC e visibilidade de dados

- **Status:** Aceita
- **Data:** 2026-08-17
- **Escopo:** Fase 0 / base de autorização

## Contexto

A especificação já exigia os papéis `ADMIN`, `ANALYST`, `VIEWER` e `AUDITOR`, além de `IncidentAccess` e permissões de domínio. A revisão identificou uma lacuna: uma permissão de leitura, sozinha, não expressa quanto do conteúdo sensível pode ser visualizado.

Em uma plataforma DFIR, logs e evidências podem conter credenciais, tokens, PII, caminhos internos, comandos e detalhes de exploração. Portanto, "somente leitura" não pode significar acesso irrestrito ao conteúdo bruto.

## Decisão

Separar explicitamente:

1. **permissão da operação** — o que o usuário pode fazer;
2. **nível máximo de visibilidade** — quão detalhado pode ser o dado consultado;
3. **projeção segura** — qual representação de dados a aplicação entrega à apresentação.

Níveis:

```text
SUMMARY
SANITIZED
DETAILED
RAW
```

Teto por papel:

```text
VIEWER  → SUMMARY
AUDITOR → SANITIZED
ANALYST → DETAILED
ADMIN   → RAW
```

`IncidentAccess` passa a possuir `max_visibility`. Uma concessão nunca pode ultrapassar o teto do papel.

`AUDITOR` recebe `AUDIT_READ` e `AUDIT_VERIFY`, mas não `EVIDENCE_READ` por padrão. `VIEWER` continua com consulta mínima e não recebe permissões de auditoria. `ANALYST` investiga dentro do teto `DETAILED`; `RAW` fica reservado ao `ADMIN` no modelo base.

## Consequências

- A Fase 0 ganha um contrato explícito de schema e autorização para futuras rotas de evidência e logs.
- A camada de apresentação não deve ser responsável por ocultar campos sensíveis depois que o dado bruto atravessou a fronteira de aplicação.
- Fases posteriores deverão implementar e testar as projeções `SUMMARY`/`SANITIZED`/`DETAILED`/`RAW`; a existência da coluna `max_visibility` no schema não é evidência de que essas projeções já estejam implementadas.
- Uma nova permissão ou nível de visibilidade exige atualização da RTM, testes e documentação.

## Não-objetivo

Esta ADR não implementa parsing, sanitização forense, DLP ou classificação jurídica dos dados. Ela estabelece somente a fronteira de autorização e visibilidade necessária para que essas capacidades futuras sejam implementadas sem pressupor que `read-only` seja seguro.
