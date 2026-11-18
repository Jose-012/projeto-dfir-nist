<!-- doc-type: decision | status: superseded -->
> **Status:** SUPERSEDED BY ADR-0017 (2026-09-22).
> The frontend toolchain (Bun, then Node.js/npm) this decision covers was removed entirely along with `frontend/`, `templates/` and `src/web` — there is no frontend build/test step anymore. Retained as historical design rationale.

# ADR-0009 - Substituição de Bun por Node.js/npm na toolchain de build/test do frontend

## Status

ACEITA — 2026-08-25

## Contexto

Desde a ADR-0005, Bun 1.3.14 era a toolchain de referência para build (`Bun.build`) e testes (`bun:test`) do frontend TypeScript. A decisão D45 (Apêndice F da especificação canônica) registrava essa escolha.

Na prática, Bun não pôde ser instalado em todos os ambientes de execução usados no ciclo de vida deste projeto: alguns têm rede restrita a um conjunto fixo de domínios (registries de pacote como npm/crates.io, mas sem acesso a `bun.sh`), o que impede a instalação do binário. Nesses ambientes, os gates de frontend (`bun run --cwd frontend build`, `bun test frontend/src/app.test.ts`, `scripts/verify_frontend.py`) nunca puderam ser executados de fato — o próprio `docs/FASE0_STATUS.md` registrava "Bun real" como gate pendente, e o script de verificação de ambiente (`scripts/validate_fase0.py`) já previa esse caso como aviso ("Bun is unavailable in this environment").

Node.js e npm, por outro lado, estavam disponíveis nesses mesmos ambientes restritos, com acesso ao registry npm já presente na lista de domínios permitidos do projeto. Isso torna Node.js/npm capazes de executar os gates de frontend de ponta a ponta em qualquer ambiente onde o Rust workspace também é validado — o que Bun nunca alcançou.

Este é exatamente o tipo de "custo operacional" previsto no critério de reavaliação da ADR-0005 ("A decisão pode ser revista somente se... custo operacional demonstrar vantagem de outro runtime").

## Decisão

A toolchain de build/test do frontend passa a ser **Node.js 22 + npm**, com:

- **`esbuild`** substituindo `Bun.build` para o bundle (`frontend/build.mjs` substitui `frontend/build.ts`): mesmo contrato de saída — bundle único IIFE, alvo browser, sem minificação, sem sourcemap, nome de arquivo fixo (`dist/app.js`).
- **`vitest`** substituindo `bun:test` para os testes (`frontend/src/app.test.ts`): API de `expect`/`test` compatível — a migração foi uma troca de uma linha de import (`bun:test` → `vitest`), sem alterar nenhuma asserção.
- **`typescript`** (`tsc --noEmit`) como gate de checagem de tipos novo, via `npm run typecheck` — Bun não expunha um gate de type-check equivalente no fluxo anterior; `frontend/tsconfig.json` foi criado (antes inexistente).

Todas as três dependências são `devDependencies` de `frontend/package.json`, instaladas localmente em `frontend/node_modules` via `npm install`, não globalmente — preservando o princípio de reprodutibilidade já adotado para o Rust workspace (versões fixadas: `esbuild@0.28.2`, `typescript@7.0.2`, `vitest@4.1.11`, sem `^`/`latest`).

`scripts/verify_frontend.py` foi reescrito para invocar `esbuild` diretamente (não via `build.mjs`, preservando a independência do rebuild em relação ao próprio script de build) e comparar bytes contra `frontend/dist/app.js`, exatamente como fazia com `bun build` antes — incluindo a checagem de texto obsoleto (agora cobre tanto "Deno" quanto "Bun").

O contrato de runtime de produção **não muda**: o build continua produzindo somente assets estáticos (`frontend/dist/app.js`, `frontend/dist/app.css`) servidos pelo `dfir-app`; Node.js não entra no runtime distribuído, exatamente como Bun não entrava. A regra "Node, Bun e Deno não pertencem à imagem/runtime de produção" permanece válida sem alteração.

### Evidência de execução

Todos os gates a seguir foram executados de fato nesta árvore (não apenas revisados estaticamente):

```text
npm install --prefix frontend        # 48 pacotes, 0 vulnerabilidades
npm run typecheck --prefix frontend  # tsc --noEmit — limpo
npm run build --prefix frontend      # dist/app.js 11.9kb, IIFE válido; dist/app.css byte-idêntico a src/app.css
npm test --prefix frontend           # vitest — 8/8 testes (app.test.ts)
python3 scripts/verify_frontend.py   # rebuild independente via esbuild + comparação de bytes — PASS
python3 scripts/validate_fase0.py    # 0 erros, 1 warning (Cargo indisponível neste ambiente)
```

### Alternativas consideradas

- **Manter Bun e aceitar gates permanentemente `PENDING` em ambientes sem acesso a `bun.sh`.** Rejeitado: o próprio objetivo do ADR-0005 era ter uma toolchain de frontend verificável; um gate que nunca pode ser executado em parte dos ambientes do projeto não cumpre esse objetivo, e "Bun real" já se arrastava como pendência desde a Fase 0.
- **Migrar para Deno.** Rejeitado sem investigação adicional: a especificação canônica já documenta uma migração histórica de Deno → Bun (changelog 6.4.0/6.4.1) pelos mesmos motivos de disponibilidade/maturidade de toolchain; não há indício de que Deno resolveria o problema de instalação em rede restrita.
- **Manter `Bun.build`/`bun:test` chamados via `npx`/wrapper.** Não é tecnicamente possível: o problema não é invocação, é a ausência do próprio binário Bun (e da possibilidade de instalá-lo) no ambiente — não há pacote `bun` publicado no registry npm para servir como wrapper local.

## Consequências

Positivas:

- gates de frontend executáveis (com evidência real, não `PENDING`) em qualquer ambiente onde o registry npm esteja acessível, incluindo os mesmos ambientes restritos usados para o Rust workspace;
- `npm run typecheck` adiciona uma checagem de tipos que não existia sob Bun;
- `package-lock.json` passa a existir (não havia lockfile do Bun versionado antes), melhorando reprodutibilidade de supply chain do frontend;
- API de teste (`expect`/`test`) manteve compatibilidade quase total com `bun:test`, minimizando a superfície de mudança em `app.test.ts`.

Negativas:

- mais uma dependência de toolchain (Node.js) precisa estar presente no ambiente de desenvolvimento, além do Rust; mitigado por já ser amplamente disponível e por não entrar no runtime de produção;
- `frontend/node_modules` é maior que a instalação equivalente do Bun (48 pacotes de dependências transitivas de dev); não afeta o artefato distribuído (`dist/app.js` permanece um bundle único de ~12kb).

## Critério de reavaliação

Esta decisão pode ser revista se uma alternativa demonstrar disponibilidade equivalente à do Node.js/npm nos ambientes de execução do projeto **e** vantagem concreta (benchmark, supply chain, manutenção) sobre `esbuild`/`vitest`/`typescript`.

## Adendo — 2026-08-27 (bug real de Windows em `verify_frontend.py`)

Execução real no ambiente de destino do operador (Windows, via `just frontend-verify`) confirmou: `npm run build`, `npm test` e `npm run typecheck` passam sem alteração. `scripts/verify_frontend.py` falhava com `OSError: [WinError 193] %1 não é um aplicativo Win32 válido`.

Causa: o script invocava `node_modules/.bin/esbuild` e `node_modules/.bin/vitest` diretamente via `subprocess.run(..., shell=False)`. No Windows, `.bin/esbuild` (sem extensão) é o script Unix (para Git Bash/WSL), não um executável Win32 — e mesmo o wrapper `.bin/esbuild.cmd` não seria invocável da mesma forma sem `shell=True`, já que `CreateProcess` não interpreta `.cmd` diretamente. Essa é uma limitação conhecida de `subprocess` no Windows para binários instalados via npm, não um bug do esbuild/vitest.

Correção: `verify_frontend.py` passa a invocar tudo através do executável `node` diretamente — `node <script>.mjs <args>` —, nunca através de um shim `.bin`. Como `vitest`'s `bin` já é um arquivo `.mjs` puro (`node_modules/vitest/vitest.mjs`), isso bastou para ele. Para o esbuild (cujo `bin/esbuild` pode ser um binário nativo específico de plataforma, ou o stub original se o `postinstall` da plataforma não rodou — ver nota abaixo sobre `allow-scripts`), foi criado `frontend/verify-build.mjs`: um script Node standalone, separado de `build.mjs` de propósito (para que um bug específico de `build.mjs` continue sendo pego pela comparação), que chama a API JS do `esbuild` (`import * as esbuild from "esbuild"`) — o mesmo caminho de código que já se provou funcionar em `npm run build`.

Nota lateral observada no mesmo log: o ambiente do operador usa um mecanismo de `allow-scripts` que bloqueia o `postinstall` do pacote `esbuild` (`npm warn allow-scripts ... esbuild@0.28.2 (postinstall: node install.js)`). Isso não impediu `npm run build` de funcionar — o binário de plataforma (`@esbuild/win32-x64`) é uma `optionalDependency` normal, instalada independente de scripts de ciclo de vida serem bloqueados; só o próprio script `postinstall` (usado por outros fins, não pela resolução do binário em si) fica pendente de aprovação. Nenhuma ação foi necessária além de rotear por `node` em vez do shim `.bin`.

Evidência real desta rodada (Windows, log do operador): `npm run build` (`dist\app.js` 11.9kb), `npm test` (vitest, 8/8), `npm run typecheck` (`tsc --noEmit`, limpo) — todos PASS. `verify_frontend.py` corrigido nesta rodada, ainda sem confirmação de execução real pós-correção (pendente do próximo relato do operador).
