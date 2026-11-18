<!-- doc-type: decision | status: partially-superseded -->
> **Status:** PARTIALLY SUPERSEDED BY ADR-0017 (2026-09-22).
> The "interface web rica" class below (Axum + HTML server-rendered + frontend) is superseded — the runtime now uses a native eframe/egui interface with no HTTP layer. The Production Runtime (Rust/SQLite/SQLx) and Engineering/QA Tooling classes remain applicable.

# ADR-0005 — Stack híbrida por classes de runtime

## Status
ATIVA — 2026-08-17

## Contexto

A unificação do backend em Rust reduziu complexidade do caminho confiável, mas tornou importante separar três problemas diferentes: runtime de produção, interface web rica e tooling de engenharia/QA.

## Decisão

### Production Runtime

- Rust + Axum para `dfir-app`;
- Rust para `dfir-engine`;
- SQLite + SQLx;
- HTML server-rendered + CSS próprio da aplicação + assets JavaScript estáticos;
- Fase 0 não distribui HTMX, Alpine.js ou Pico.css; essas camadas não são necessárias para a interface atual.

### Build Toolchain

- Rust/Cargo;
- Bun 1.3.14 para build/test do frontend;
- TypeScript padrão de browser, sem APIs proprietárias do runtime.

### Engineering/QA Tooling

- CPython 3.14.6;
- Shell;
- scripts para fixtures, validação, fuzzing/orquestração, benchmarks e release.

Node, Bun e Deno não entram na imagem de produção; Bun é toolchain de build/test. Python também não entra por padrão.

### Regra de produção

*(Sem conteúdo preenchido nesta revisão — a regra efetiva de produção está descrita acima, em "Production Runtime", hoje parcialmente superseded por ADR-0017.)*

## Consequências

Positivas:

- runtime de produção menor;
- frontend mais apropriado a visualização rica;
- Python continua disponível para automação/QA sem virar backend;
- fronteiras de segurança permanecem concentradas no Rust;
- build frontend pode evoluir sem alterar o backend.

Negativas:

- mais de um toolchain no ambiente de desenvolvimento;
- supply chain de frontend precisa de gate próprio;
- equipe precisa entender claramente a diferença entre build/tooling e runtime.

## Critério de reavaliação

A decisão pode ser revista somente se benchmark, threat model, necessidade de distribuição ou custo operacional demonstrar vantagem de outro runtime no caminho de produção.

## Adendo — 2026-08-25 (substituição de Bun por Node.js/npm)

A linha "Bun 1.3.14 para build/test do frontend" em **Build Toolchain**, acima, está superseded. Bun não era instalável em todos os ambientes de execução usados no projeto (rede restrita, sem acesso a `bun.sh`), o que é exatamente o tipo de custo operacional previsto no critério de reavaliação desta ADR. A toolchain de build/test do frontend passa a ser **Node.js/npm** (`esbuild` para bundling, `vitest` para testes). As demais decisões desta ADR (Production Runtime, Engineering/QA Tooling, regra de que Node/Bun/Deno não entram na imagem de produção) permanecem inalteradas — apenas o nome da ferramenta de build/test muda, não o papel que ela ocupa na arquitetura.

Decisão completa, alternativas consideradas e evidência de execução: `docs/adrs/ADR-0009-substituicao-bun-por-node-npm.md`.
