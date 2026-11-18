<!-- doc-type: status | status: superseded -->
# Fase 0 — histórico de correções (2026-08-20 a 2026-08-25)

> **HISTÓRICO / SUPERSEDED.** Movido de `docs/FASE0_STATUS.md` em 2026-10-02 (ADR-0018). Descreve a era Axum + `frontend/` TypeScript + rotas HTTP, removida pela ADR-0017. Os caminhos `src/web/`, `frontend/` e `DELETE /api/evidence/...` citados aqui **não existem mais**. O estado atual está em `docs/FASE0_STATUS.md`, `docs/FASE1_STATUS.md` e `docs/rtm.md`.

## Correção do gate Windows — 2026-08-21
- `scripts/run_runtime_test.py` passou a iniciar o `fase0_runtime.ps1` com `-ExecutionPolicy Bypass` no processo filho do PowerShell.
- A alteração é exclusivamente de QA/tooling e não modifica a Execution Policy persistente do usuário/máquina nem o runtime do produto.
- O objetivo é eliminar o bloqueio `AuthorizationManager check failed` observado no teste não interativo do Windows.
- A validação final do runtime permanece `PENDING` até execução real de `just verify-runtime` no ambiente Windows de destino.

## Evidência recente — 2026-08-20
- Auditoria profunda — correção do mapeamento `evidence/...` e contenção por arquivo no validação de escopo do processo adicionada; compilação Rust e runtime continuam pendentes de execução final após esta alteração.

- `cargo check --workspace --locked --all-targets` — PENDING-HUMAN: a execução Windows de 2026-08-24 reproduziu falhas de compilação na timeline; a evidência PASS anterior foi invalidada como STALE.
- O `Cargo.lock` desta árvore contém `dependência Windows de processo separado`; a pendência anterior de lockfile materializado está resolvida.
- Permanecem pendentes os testes e gates de lint/QA/runtime listados acima; esta atualização não os considera executados.
## Correções de operação da timeline — 2026-08-24
- Corrigido o bundle versionado `frontend/dist/app.js`, que ainda continha a implementação anterior e continuava exibindo `record_offset` apesar de `src/web/mod.rs` já ter removido o campo.
- O parser frontend agora normaliza explicitamente o contrato permitido e descarta campos obsoletos, impedindo que `record_offset` volte a chegar à UI.
- O filtro da timeline agora acompanha o conjunto atual de eventos após uma análise, em vez de permanecer preso à lista inicial renderizada no carregamento da página. A busca inclui título, descrição, Event ID, provider, channel, evidência e referência bruta.
- Adicionada operação local explícita `DELETE /api/evidence/{evidence_id}` para descarte de evidência importada: remove o arquivo físico e resultados de análise associados, marca `preservation_status=DESTROYED` e registra `EVIDENCE_DESTROYED` na audit chain e `DESTROYED` na chain of custody. Eventos de auditoria/custody não são apagados.
- A operação é deliberada no modo local por confirmação do nome exato do arquivo. Como o runtime local não possui autenticação/RBAC, não é apresentada como autorização remota/ADMIN; eventual modo multiusuário deverá impor essa política na fronteira de autorização.
- Evidências `DESTROYED` deixam de aparecer no catálogo e não podem voltar a ser analisadas.

## Correção da timeline EVTX — 2026-08-24
- Corrigida a fronteira `evtx 0.12.2` → resultado normalizado: o engine usa o `EvtxRecord` apenas para os metadados suportados pelo contrato e não expõe `binxml_offset` como se fosse um offset físico global do arquivo.
- A timeline do app agora ordena pelo timestamp RFC 3339 normalizado em UTC e usa `evidence_id + raw_reference` como identidade estável para desempate/deduplicação.
- Cada evento da timeline passa a expor descrição, Event ID, provider, channel, evidência e referência bruta; não expõe `binxml_offset` como posição física do arquivo.
- A apresentação do frontend deixa de converter silenciosamente o timestamp para o fuso local e passa a exibir UTC explicitamente.
- Regressões adicionadas para identidade física, ordenação, deduplicação e descrição.
- Cargo/Bun permanecem PENDING-HUMAN neste ambiente; os gates documentais e estáticos executados em 2026-08-24 passaram.
- 2026-08-24: timeline EVTX passa a usar TimeCreated/SystemTime como timestamp semântico; provider passa a extrair Name de #attributes; regressões adicionadas.

## Substituição de Bun por Node.js/npm — 2026-08-25
- Bun foi removido da toolchain do frontend por não ser instalável neste ambiente de execução (rede restrita, sem acesso a `bun.sh`); Node.js 22/npm eram disponíveis e permitiram execução real dos gates de frontend, algo que o gate "Bun real" nunca havia alcançado nesta árvore.
- `frontend/build.ts` (`Bun.build`) foi substituído por `frontend/build.mjs` (`esbuild`); `bun:test` foi substituído por `vitest` (API compatível, mudança de import de uma linha em `app.test.ts`); `frontend/tsconfig.json` foi adicionado (inexistente sob Bun).
- Executados nesta árvore, com evidência real (não PENDING): `npm run typecheck` (limpo), `npm run build` (`dist/app.js` 11.9kb, IIFE, `dist/app.css` byte-idêntico a `src/app.css`), `npm test` (8/8 testes), `python3 scripts/verify_frontend.py` (rebuild independente via esbuild, comparação de bytes, checagem de texto Deno/Bun obsoleto — PASS), `python3 scripts/validate_fase0.py` (0 erros, 1 warning — apenas Cargo indisponível neste ambiente).
- Todos os documentos normativos que citavam Bun (`AGENTS.md`, `CONVENTIONS.md`, `PROMPT_NOVA_CONVERSA.md`, `docs/DOCUMENT_AUTHORITY.md`, `docs/RELEASE_GATE.md`, `docs/SUPPLY_CHAIN.md`, `docs/FRONTEND_ARCHITECTURE.md`, `docs/TOOLCHAINS.md`, `toolchains.toml`, `justfile`, `README.md`, `CONTRIBUTING.md`, `frontend/README.md`, `docs/DEFINITION_OF_READY.md`, `docs/FASE1_BACKLOG.md`, `tests/frontend/fase0.md`) foram atualizados; a especificação canônica (`docs/PROJECT_SPEC_v6_4_5.md` + cópia raiz, mantidas byte-idênticas) recebeu uma nota de implementação superseded e a decisão normativa `D46` (supersede `D45`), seguindo o mesmo mecanismo já usado para a remoção do Podman/Quadlet.
- Detalhes completos e alternativas consideradas: `docs/adrs/ADR-0009-substituicao-bun-por-node-npm.md`.
- Cargo/Rust permanecem PENDING-HUMAN neste ambiente (sem `cargo`/`rustc` instalados); nenhuma alteração de código Rust nesta seção altera esse estado.
