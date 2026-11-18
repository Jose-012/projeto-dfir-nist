<!-- doc-type: review | status: historical -->
> **Status: HISTÓRICO / SUPERSEDED.** Este documento registra uma revisão anterior e não é fonte normativa do runtime atual. Consulte `docs/PROJECT_SPEC_v6_4_5.md` e `docs/FASE0_STATUS.md`.

# Revisão meticulosa de código — 2026-08-20

## Escopo

Passe final sobre o projeto `dfir-app` após a simplificação para ferramenta forense local/offline. A revisão cobriu Rust, protocolo JSONL, process boundary Linux/Windows, persistência/auditoria, migrações, frontend Bun, scripts QA, configuração, documentação e consistência entre especificação e código.

## Correções desta rodada

- `StoragePaths` agora provisiona `runtime`.
- Diretórios de armazenamento criados pelo aplicativo recebem modo `0700` em Unix.
- `setup.ps1` e `setup.sh` foram sincronizados com `runtime` e permissões locais.
- O smoke test agora cria um sentinel fora da área de output, exige que o engine escreva um marcador dentro da área autorizada e falha caso o parser consiga ler o sentinel proibido.
- Linux passa explicitamente `DFIR_ENGINE_INPUT_ROOT=/input` e `DFIR_ENGINE_OUTPUT_ROOT=/output`; no smoke, o caminho proibido fica fora da árvore montada.
- Windows passa os mesmos roots e um `DFIR_ENGINE_SMOKE_FORBIDDEN_PATH` apenas durante o smoke.
- Removido o teste Windows baseado em `cmd.exe`: era um teste de uma implementação auxiliar, não do `dfir-engine` real. A validação autoritativa passa pelo binário do engine.
- Corrigida a filtragem de `TEMP`/`TMP` no ambiente Windows para comparar `OsString` de forma robusta.
- O QA passou a incluir dependências declaradas em tabelas `target.*` do Cargo.toml; agora detecta explicitamente que `` ainda não está no `Cargo.lock`.
- `release_gate.py` deixou de gerar traceback quando uma ferramenta obrigatória (`rustc`, `cargo`, `bun`, etc.) não está instalada; agora informa a pré-condição ausente de forma determinística.
- A decisão histórica de backoff de autenticação foi marcada como `SUPERSEDED` em favor do escopo local atual.

## Verificações executadas nesta sessão

- `python3 -m py_compile scripts/*.py tests/security/*.py` — PASS
- `python3 scripts/validate_fase0.py` — PASS, 0 erros; 3 avisos conhecidos de ambiente/lockfile
- `python3 scripts/doc_gate.py` — PASS
- `python3 tests/security/fase0_config.py` — PASS
- `python3 tests/security/rejected_external_tools.py` — PASS
- migrações SQL executadas em SQLite via Python — PASS
- cópias canônicas da especificação byte-a-byte idênticas — PASS
- varredura de APIs web perigosas no frontend (`innerHTML`, `eval`, `WebSocket`, etc.) — PASS
- varredura de rotas/configuração de autenticação removidas do runtime — PASS
- verificação de referências residuais do teste `cmd.exe` antigo — PASS

## Atualização de evidência — 2026-08-20

- `cargo check --workspace --locked --all-targets` — PASS, conforme execução fornecida pelo desenvolvedor no Windows.
- `Cargo.lock` contém ``; a pendência anterior de lockfile não é mais aplicável à árvore atual.
- Os dois warnings de compilação identificados nesta revisão foram corrigidos por condicionamento de plataforma, sem `#[allow(dead_code)]`.

## Pendências que exigem ambiente real

- `cargo test --workspace --locked`, Clippy e Nextest ainda precisam ser executados pelo desenvolvedor.
- `bun build` e `bun test` não foram executados porque Bun não está disponível nesta sessão.

