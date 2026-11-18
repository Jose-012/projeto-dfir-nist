> HISTORICAL AUDIT — superseded by `docs/archive/FINAL_REVIEW_2026-08-18.md`.

# Auditoria final preventiva pré-congelamento — 2026-08-17

## Objetivo

Revisar o projeto Fase 0 integralmente antes do primeiro ciclo de validação Linux/WSL2, procurando problemas de arquitetura, segurança, coerência documental, toolchain, supply chain, frontend, runtime e simplificação.

## Correções aplicadas nesta auditoria final

1. O backoff de autenticação passou a ser efetivamente aplicado antes da verificação da senha real; contas temporariamente bloqueadas passam por verificação dummy e continuam rejeitadas.
2. Foi adicionado teste de regressão para garantir que uma conta bloqueada não autentique mesmo com a senha correta.
3. Dados JSON inseridos em `<script type="application/json">` passaram a ser escapados para o contexto HTML, reduzindo risco de quebra de contexto/XSS quando a timeline receber dados reais.
4. Páginas de login e painel autenticado usam `Cache-Control: no-store`.
5. A sessão de login somente é criada depois que a auditoria de sucesso foi gravada.
6. `DFIR_HTTPS=false` passou a ser proibido por padrão em binds não-loopback. O perfil de desenvolvimento Compose possui `DFIR_ALLOW_INSECURE_HTTP=true` como opt-in explícito; o perfil de produção/Quadlet não o utiliza.
7. A verificação do frontend compara `frontend/dist/app.js` versionado contra uma transpile independente em diretório temporário; não existe mais risco de o próprio gate sobrescrever o artefato e depois considerá-lo válido.
8. Deno foi atualizado para a referência 2.9.4.
9. A imagem de build/runtime foi atualizada para Rust 1.97.1 + Alpine 3.24.0, mantendo a imagem final enxuta e alinhada às versões atuais verificadas.
10. O teste de configuração foi atualizado para a baseline histórico v6.4.1.
11. A revisão anterior foi promovida ao baseline histórico v6.4.1 para registrar essas correções sem alterar a arquitetura de fundo.

## Estado da arquitetura

### Runtime de produção

- `dfir-app` em Rust
- `dfir-engine` em Rust, processo separado
- SQLite + SQLx + repositories finos
- JSONL por stdin/stdout entre aplicação e engine
- bubblewrap por job
- assets frontend estáticos compilados
- Typst e demais componentes necessários explicitamente empacotados

### Build

- Rust/Cargo
- Deno 2.9.4 para TypeScript

### Engenharia/QA

- CPython 3.14.6
- Shell
- fixtures, benchmarks e scripts de validação

Python e Deno não fazem parte da imagem/runtime de produção.

## Simplificações preservadas

- sem ORM;
- sem Node/Bun/Deno no runtime;
- sem backend TypeScript;
- sem Python no caminho de confiança de produção no MVP;
- sem WebSocket no MVP;
- HTTP/JSON como transporte padrão browser;
- SSE apenas quando houver necessidade real de eventos unidirecionais;
- sem `vis-timeline`/D3 obrigatório na Fase 0;
- sem microserviços;
- sem PostgreSQL/Redis/message broker;
- sem integração remota privilegiada no MVP.

## Verificações executáveis neste ambiente

PASS:

- `scripts/validate_fase0.py`
- `tests/security/fase0_config.py`
- `tests/security/rejected_external_tools.py`
- sintaxe Bash dos scripts principais
- sintaxe Python dos scripts de QA
- parsing TOML/JSON
- parsing de todos os blocos XML estruturados da especificação
- igualdade byte-a-byte das duas cópias da especificação
- ausência de referências ativas à especificação antiga fora do arquivo histórico

PENDENTE por ausência das ferramentas no ambiente de auditoria:

- `cargo metadata --locked`
- `cargo tree --locked -e features`
- `cargo tree --locked -d`
- `cargo fmt --all -- --check`
- `cargo check --locked --all-targets`
- `cargo clippy --locked --all-targets --all-features -- -D warnings`
- `cargo nextest run --locked`
- `cargo audit`
- `cargo deny check`
- `cargo vet`
- execução real do Deno
- build/run real do Podman
- execução real do bubblewrap
- testes adversariais Linux de filesystem/rede/protocolo
- geração do SBOM pelo workspace realmente compilado
- digest final da imagem realmente testada

## Conclusão

Não foram identificadas, nesta revisão, novas mudanças arquiteturais de baixo custo e alto benefício que justifiquem reabrir o desenho antes dos testes. O próximo ciclo deve ser orientado por evidência de execução. Qualquer nova mudança após este ponto deve ser motivada por falha de gate, benchmark, finding de segurança, requisito novo ou ADR explícita.
