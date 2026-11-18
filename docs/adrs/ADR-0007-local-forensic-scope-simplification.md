<!-- doc-type: decision | status: accepted -->
# ADR-0007 - Simplificação para ferramenta forense local

## Status

Accepted - 2026-08-20

## Contexto

A demanda original SAGA SENAI 12412 descreve uma avaliação técnica e forense de incidente cibernético, com análise de evidências, timeline, identificação de vulnerabilidades e produção de recomendações NIST CSF 2.0, hardening e criptografia. Ela não exige um produto multiusuário, servidor remoto ou gerenciamento de identidades.

A versão anterior da arquitetura adicionou autenticação, sessões, RBAC, autorização por incidente, CSRF e controles associados ao conceito de aplicação servidor. Isso aumentou a superfície de código sem contribuir diretamente para os entregáveis da demanda.

## Decisão

O produto passa a ser explicitamente uma ferramenta local/offline.

Removidos do runtime:

- contas de usuário;
- login/logout;
- sessões/cookies/CSRF;
- RBAC e autorização por incidente;
- bootstrap administrativo;
- rate limiting HTTP;
- modo servidor/multiusuário;
- TLS/reverse proxy como parte do produto.

Mantidos:

- bind loopback por padrão;
- SQLite local;
- audit chain tamper-evident;
- preservação e storage de evidências;
- parser em processo separado;
- processo separado por job;
- limites de recursos;
- Bun apenas como toolchain de frontend.

## Consequências

A aplicação fica menor, mais apropriada ao fluxo de um analista e mais fácil de validar offline. Se no futuro houver requisito de operação remota/multiusuário, autenticação e autorização deverão ser reintroduzidas por uma nova decisão arquitetural explícita.

## Adendo — 2026-08-22 (remoção de containerização)

Por decisão do operador do projeto, o empacotamento em container (Podman rootless, Quadlet, `compose.yaml`) — que esta ADR já classificava como opcional — foi removido por completo do produto, não apenas tornado opcional.

Removidos do repositório:

- `containers/Containerfile.app`;
- `quadlet/dfir-app.container`;
- `compose.yaml`;
- o gate de release específico de imagem/digest (`scripts/release_gate.py` e o alvo `release-gate` do `justfile`).

O modo de execução único e normativo passa a ser o binário nativo (`dfir-app`/`dfir-engine`), compilado via `cargo build --release --locked --workspace` e executado diretamente pelo operador local. A separação de processo entre `dfir-app` e `dfir-engine` — o limite de segurança que importa para uma ferramenta que trata evidência como entrada não confiável — não dependia de isolamento por container e permanece inalterada.

Se um requisito concreto justificar empacotamento em container no futuro (distribuição, hardening de host compartilhado etc.), isso exige nova evidência técnica e uma nova ADR.
