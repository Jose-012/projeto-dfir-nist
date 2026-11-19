<!-- doc-type: overview | status: current -->
# DFIR/NIST - ferramenta forense local

Aplicação local/offline para preservar e analisar evidências, reconstruir timelines e produzir entregáveis alinhados ao NIST CSF 2.0.

Repositório remoto privado: [Jose-012/projeto-dfir-nist](https://github.com/Jose-012/projeto-dfir-nist).

## Arquitetura

```text
dfir-app - Rust + eframe/egui + SQLite
  │
  │ stdin/stdout + JSONL
  ▼
dfir-engine - Rust + evtx + yara-x (um processo por job)
```

A aplicação é nativa, local e offline, sem contas, login, sessões, RBAC, servidor HTTP ou porta de rede. A GUI chama `src/service/` diretamente. O parser recebe evidência como entrada não confiável e roda em processo separado, com caminhos validados e storage autorizado (ADR-0017 e ADR-0024).

## Linha de comando (headless)

Sem subcomando, `dfir-app` abre a interface nativa. Os subcomandos são não interativos, escrevem JSON em stdout e logs em stderr.

| Subcomando | Uso | O que faz |
|---|---|---|
| `import` | `import <ARQUIVO> [--source ..] [--provided-by ..] [--collection-method ..] [--collection-tool ..] [--collection-tool-version ..] [--collected-at AAAA-MM-DD] [--authorization ..] [--personal-data]` | preserva, calcula hash e registra custódia/proveniência declarada |
| `evidence-list` | `evidence-list` | lista evidências ativas |
| `analyze` | `analyze <ID> [--kind EVTX_TIMELINE ou YARA_SCAN] [--max-output-bytes N] [--max-items N]` | executa análise; o padrão é `EVTX_TIMELINE` |
| `results` | `results <JOB_ID> [--offset N] [--limit N] [--filter TEXTO]` | pagina a timeline; aceita texto livre e `campo:valor` |
| `findings` | `findings <ID>` | lista resultados analíticos da evidência |
| `detect` | `detect <ID>` | cria candidatos de detecção pendentes |
| `candidates` | `candidates <ID> [--status PENDING, CONFIRMED ou REJECTED]` | lista candidatos |
| `candidate-confirm` | `candidate-confirm <CANDIDATO> --incident <ID> [--note ..]` | confirma candidato e cria achado/IOCs no incidente |
| `candidate-reject` | `candidate-reject <CANDIDATO> --reason ..` | rejeita com motivo auditado |
| `link-evidence` | `link-evidence <ID> --incident <INCIDENTE>` | vincula evidência ao incidente de forma idempotente |
| `incident-evidence` | `incident-evidence <INCIDENTE>` | lista evidências vigentes do incidente |
| `custody` | `custody <ID>` | mostra a cadeia de custódia |
| `audit` | `audit [--limit N]` | mostra eventos da trilha de auditoria |
| `destroy` | `destroy <ID> --confirm-filename <NOME>` | descarta evidência após confirmação exata |
| `deliverables` | `deliverables <INCIDENTE> [--out PASTA] [--no-redaction]` | gera cinco documentos Markdown e `manifest.json`, mascarados por padrão |
| `demo` | `demo <ARQUIVO.evtx>` | registra o incidente sintético de demonstração, rotulado DEMO |
| `diagnostics` | `diagnostics` | emite diagnóstico seguro, sem nomes/conteúdo de evidências |
| `audit-verify` | `audit-verify` | verifica audit chain e custódia |
| `engine-smoke` | `engine-smoke` | executa smoke test do engine |
| `backup-verify` | `backup-verify <PASTA>` | reverifica cópia do banco feita antes de migration |

O fluxo ponta a ponta usa `just e2e caminho/arquivo.evtx`. Operação, volume protegido e backup: `docs/OPERATION.md`. Demonstração: `docs/DEMO.md`.

## Logs, timeline e diagnóstico

- Logs ficam em `<DFIR_DATA_ROOT>/logs/dfir.log`, com rotação e permissão restrita. Contêm ids, contagens, durações e códigos de erro, nunca nome ou conteúdo de evidência. Não há telemetria.
- `dfir-app diagnostics` produz JSON seguro para suporte.
- A timeline fica no SQLite, paginada e filtrável no banco. Ela reaparece sem reanálise.
- `DFIR_MIN_FREE_DISK_BYTES` protege importação e análise contra falta de espaço. Avisos do parser ficam no job e na timeline.
- Testes de GUI usam `egui_kittest`; evidência de execução real fica em `docs/EVIDENCE_REGISTER.md`.

## Limites de saída

A análise não trunca a timeline por padrão (ADR-0020). Tempo, memória, tamanho de importação, espaço livre e profundidade JSON têm limites de segurança. `--max-output-bytes` e `--max-items` permitem tetos opcionais por análise, com erro explícito ao estourar.

## Requisitos e escopo

A demanda original está em `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md` e é rastreada em `docs/DEMAND_TRACE.md`. A especificação canônica é `docs/PROJECT_SPEC_v6_5_1.md`.

O protótipo analisa **EVTX**. Outros arquivos podem ser preservados, mas não são analisados. Importação CSV/JSONL e coletas de rede estão adiadas (ADR-0025).

- Estado de requisitos e tickets: `docs/rtm.md`.
- Gates e evidências executadas: `docs/RELEASE_GATE.md` e `docs/EVIDENCE_REGISTER.md`.
- Trabalho remanescente: `docs/FASE1_BACKLOG.md`.

A detecção só enxerga o que o log registrou. Ausência de candidato não prova ausência de ataque e, sem log de rede, exfiltração não pode ser confirmada. Fixture e cenário DEMO não substituem incidente real.

## Capacidades

- SQLite/WAL, migrations e cópia verificada antes de migration;
- audit chain append-only com BLAKE3-256-v2-JCS e verificação da versão anterior;
- preservação com SHA-256, custódia e proveniência declarada;
- engine separado, JSONL versionado e sandbox Linux;
- timeline EVTX com campos estruturais e filtro `campo:valor`;
- YARA-X e detecção determinística com confirmação/rejeição pelo analista;
- NIST Current/Target/Gap e visão por Function;
- ATT&CK, caminho crítico, impacto, IOCs, achados e recomendações;
- plano nos oito domínios, lacunas de fonte e redação de dados pessoais;
- cinco documentos Markdown com `manifest.json` e cenário DEMO.

O parser usa `evtx 0.15.1` com `default-features = false`. BinXML não é reimplementado.

## Interface

A interface usa `eframe`/`egui`/`egui_extras` e roda na thread principal. SQLite e subprocessos ficam na thread de backend (`docs/NATIVE_UI_ARCHITECTURE.md`). O seletor nativo de arquivos é apenas a entrada interativa do caminho, que continua sendo validado.

## Configuração

Variáveis principais:

- `DFIR_DATA_ROOT`
- `DFIR_ENGINE_PATH`
- `DFIR_ENGINE_MAX_LINE_BYTES`
- `DFIR_ENGINE_TIMEOUT_SECONDS`
- `DFIR_ENGINE_MAX_CONCURRENT_JOBS`
- `DFIR_ENGINE_MAX_MEMORY_BYTES`
- `DFIR_ENGINE_SANDBOX`
- `DFIR_MAX_UPLOAD_BYTES`
- `DFIR_MIN_FREE_DISK_BYTES`
- `RUST_LOG`
- `DFIR_ENGINE_INPUT_FILE`, `DFIR_ENGINE_INPUT_ROOT`, `DFIR_ENGINE_OUTPUT_ROOT` e `DFIR_ENGINE_MEMORY_LIMIT_BYTES` são internas e não devem ser configuradas manualmente.

Valores, padrões e fallback de instalação: `docs/OPERATION.md` e `src/config.rs`. Toolchain de desenvolvimento: `docs/TOOLCHAINS.md`.

## Validação

```bash
just doctor
just doc-gate
just verify
just supply-chain
just e2e tests/fixtures/senai-demo.evtx
```

`just verify` executa nextest e doc-tests, sem rodar unitários/integração duas vezes. `just test` permanece disponível para o runner padrão completo.

O workflow `.github/workflows/ci.yml` executa esses gates no GitHub Actions, em `ubuntu-24.04`, com Rust 1.100.0 e CPython 3.14.6. A execução [44567609855](https://github.com/Jose-012/projeto-dfir-nist/actions/runs/44567609855) terminou com nextest 536 passed/1 skipped, supply chain e doc-gate verdes e E2E 110/110 (`EVD-054`). Esse resultado cobre Linux x86_64 e não substitui as verificações externas ou nativas pendentes.

Testes adicionais de runtime:

```bash
just runtime-test
just runtime-security
cargo run -- audit-verify
cargo run -- engine-smoke
```

Nenhum gate deve ser descrito como executado sem evidência real do ambiente correspondente.

## Segurança

```text
arquivo de evidência
      ↓
engine (processo separado; tempo/memória limitados; saída sem truncamento, com tetos opcionais)
      ↓
resultado JSONL validado e persistido sem truncamento
      ↓
aplicação local
```

A evidência original é preservada. A cadeia de auditoria é tamper-evident e serializa writers com `BEGIN IMMEDIATE`. Importação, evento `EVIDENCE_IMPORTED` e primeira custódia são gravados na mesma transação. A destruição governada preserva a auditoria e a custódia.
