<!-- doc-type: overview | status: current -->
# DFIR/NIST - ferramenta forense local

Aplicação local/offline para apoiar investigação forense de incidentes, preservação e análise de evidências, reconstrução de timelines e produção de entregáveis alinhados ao NIST CSF 2.0.

## Arquitetura

```text
dfir-app - Rust + eframe/egui + SQLite
  │
  │ stdin/stdout + JSONL
  ▼
dfir-engine - Rust + evtx + yara-x (um processo por job)
```

A aplicação é um único binário nativo, local/offline, sem contas de usuário, login, sessões, RBAC, servidor HTTP ou porta de rede — nem mesmo em loopback (ADR-0017 substituiu o antigo par Axum + frontend TypeScript por uma interface nativa eframe/egui/egui_extras).

O parser recebe evidência como entrada não confiável e é executado em processo separado. A aplicação valida e canonicaliza os caminhos de entrada/saída e mantém toda a análise dentro do storage autorizado.

## Linha de comando (headless)

Sem subcomando, `dfir-app` abre a interface nativa. Os subcomandos abaixo são **não interativos**, escrevem JSON em stdout (logs em stderr) e existem para testes ponta a ponta e CI (ADR-0019; não há TUI):

| Subcomando | Uso | O que faz |
|---|---|---|
| `import` | `import <ARQUIVO> [--source ..] [--provided-by ..] [--collection-method ..] [--collection-tool ..] [--collection-tool-version ..] [--collected-at AAAA-MM-DD] [--authorization ..] [--personal-data]` | preserva, calcula hash, registra custódia e a proveniência declarada (o que não for informado fica como não informado) |
| `evidence-list` | `evidence-list` | lista as evidências ativas (JSON) |
| `analyze` | `analyze <ID> [--kind EVTX_TIMELINE ou YARA_SCAN] [--max-output-bytes N] [--max-items N]` | executa uma análise sobre a evidência (padrão `EVTX_TIMELINE`) |
| `results` | `results <JOB_ID> [--offset N] [--limit N] [--filter TEXTO]` | uma página da timeline, em ordem cronológica (limite padrão 200); o filtro aceita texto livre e termos `campo:valor` (ver `docs/OPERATION.md`) |
| `findings` | `findings <ID>` | resultados analíticos da evidência (ex.: YARA) |
| `detect` | `detect <ID>` | roda as regras de detecção sobre uma evidência EVTX e grava **candidatos** pendentes; nada vira achado sem confirmação |
| `candidates` | `candidates <ID> [--status PENDING, CONFIRMED ou REJECTED]` | lista os candidatos de detecção da evidência |
| `candidate-confirm` | `candidate-confirm <CANDIDATO> --incident <ID> [--note ..]` | confirma o candidato: cria o mapeamento ATT&CK, o achado e os IOCs no incidente informado |
| `candidate-reject` | `candidate-reject <CANDIDATO> --reason ..` | rejeita o candidato; o motivo é obrigatório e fica na auditoria |
| `link-evidence` | `link-evidence <ID> --incident <INCIDENTE>` | vincula a evidência ao incidente (idempotente; grava `EVIDENCE_LINKED_TO_INCIDENT` na auditoria) |
| `incident-evidence` | `incident-evidence <INCIDENTE>` | lista as evidências vigentes do incidente (sem janela de tempo) |
| `custody` | `custody <ID>` | cadeia de custódia da evidência (append-only) |
| `audit` | `audit [--limit N]` | eventos da trilha de auditoria com prefixo de hash encadeado |
| `destroy` | `destroy <ID> --confirm-filename <NOME>` | descarte local; exige o nome exato do arquivo |
| `deliverables` | `deliverables <INCIDENTE> [--out PASTA] [--no-redaction]` | 5 documentos Markdown + `manifest.json` em `<PASTA>/entregaveis-<INCIDENTE>/` (padrão: pasta de exportações); dados pessoais mascarados por padrão |
| `demo` | `demo <ARQUIVO.evtx>` | incidente de DEMONSTRAÇÃO: importa o EVTX, roda as análises reais e registra o cenário BEC sintético (rotulado DEMO); ver `docs/DEMO.md` |
| `diagnostics` | `diagnostics` | JSON para suporte: versões, contagens, falhas por código, disco, logs (sem nomes de arquivo) |
| `audit-verify` | `audit-verify` | verifica a audit chain e a chain of custody de todas as evidências armazenadas |
| `engine-smoke` | `engine-smoke` | teste de fumaça do engine |
| `backup-verify` | `backup-verify <PASTA>` | reverifica uma cópia do banco feita antes de migration (RF-032): SHA-256, integridade do SQLite, chaves estrangeiras e contagens; aceita o nome em `backups/` ou o caminho |

> Gap origem: INC-08 (2026-11-04). São 20 subcomandos, os mesmos de `src/main.rs`; a lista anterior mostrava 11.

Teste ponta a ponta com o engine real: `just e2e caminho/arquivo.evtx` (ou `python3 scripts/e2e_headless.py --evtx ...`). Cobre o pipeline de evidência e, sobre o incidente de demonstração, os cinco documentos (`docs/DEMO.md`). Operação (volume protegido, backup, atualização manual): `docs/OPERATION.md`.

## Logs, diagnóstico e testes de GUI (ADR-0022)

- **Logs:** `<DFIR_DATA_ROOT>/logs/dfir.log` (JSON, rotação de 5 × 5 MiB, permissão 0600) e stderr. Nível por `RUST_LOG` (padrão `info`; `RUST_LOG=dfir_app=debug` mostra cada página da timeline servida). Têm ids, tamanhos, contagens, durações e erros; **não** têm o nome do arquivo de evidência nem conteúdo de eventos. Panics também vão para o log. Não há telemetria enviada a lugar nenhum.
- **Diagnóstico:** `dfir-app diagnostics` gera um JSON seguro para anexar a um pedido de suporte (sem nomes de arquivo, caminhos de evidência nem mensagens de erro).
- **Testes de GUI:** `cargo test` roda testes sem janela com `egui_kittest` (digitar e clicar pela árvore de acessibilidade).

## Espaço em disco, timeline e avisos (ADR-0021)

- A timeline fica no SQLite (paginada e filtrada no banco): a memória não cresce com o tamanho do log, e ela reaparece ao selecionar a evidência, sem reanalisar.
- `DFIR_MIN_FREE_DISK_BYTES` (padrão 1 GiB; 0 desliga): recusa importar ou analisar abaixo da margem e interrompe a análise em andamento com `DISK_SPACE_LOW`. Medição de espaço só existe em Unix.
- Avisos do parser (ex.: `EVTX_TRUNCATED`, `EVTX_RECORDS_MISSING`) ficam gravados no job e aparecem em destaque na timeline: um job concluído com aviso não é uma conclusão limpa.

## Limites de saída

Desde a ADR-0020 a análise **não tem limite de saída nem trunca a timeline**: todos os eventos do EVTX chegam à tela. O que continua limitado, por segurança: tempo do engine (`DFIR_ENGINE_TIMEOUT_SECONDS`), memória virtual do processo, tamanho de importação (`DFIR_MAX_UPLOAD_BYTES`) e profundidade do JSON. Quem quiser um teto de saída pode pedi-lo por análise (`dfir-app analyze <ID> --max-output-bytes N --max-items N`); estourar o teto falha o job com erro explícito.

## Fonte de requisitos

A demanda original (SAGA SENAI 12412, Avivatec) está em `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md` e é rastreada item a item em `docs/DEMAND_TRACE.md`. A especificação canônica é `docs/PROJECT_SPEC_v6_5_1.md`; o escopo e o backlog foram realinhados à demanda pela ADR-0018 (`docs/FASE1_BACKLOG.md`).

A demanda tem dois blocos: investigar o incidente (preservar, analisar, linha do tempo) e transformar a investigação em um plano de fortalecimento. Os seis entregáveis são: relatório forense, vulnerabilidades exploradas, maturidade NIST CSF 2.0, recomendações, plano de hardening e plano de criptografia/proteção de dados.

## Estado do MVP

Protótipo (ADR-0023): o operador definiu o escopo como EVTX, sem outros formatos de entrada nesta fase.

- **Existe e funciona em código** (testado em **rustc 1.100.0, o pin atual**, em Linux x86_64, em **2026-11-18** (`EVD-012`): `just verify` — `fmt --check`, `check --workspace --locked`, `nextest` **426/426**, `clippy -D warnings` — e o e2e **110/110** com engine real e a fixture versionada `tests/fixtures/senai-demo.evtx`; em 2026-11-14 no pin anterior 1.99.1 o e2e também foi 110/110 com o `security.evtx` de referência, 2 261 eventos, `EVD-002`/`EVD-006`; a GUI foi exercitada por `egui_kittest` e numa janela real X11, `EVD-009`): preservação com hash e custódia; proveniência declarada na importação; EVTX com timeline completa, **campos estruturais e filtro `campo:valor`** (parser `evtx 0.15.1`); YARA-X; **detecção por regras sobre o EVTX com candidatos que o analista confirma ou rejeita** (19 regras, `docs/DETECTION_RULES.md`); NIST Current/Target/Gap e visão por Function; ATT&CK, caminho crítico, impacto, IOCs e recomendações; achados e causa-raiz; itens de plano nos oito domínios; lacunas de fonte; os cinco documentos Markdown com `manifest.json`, com máscara de dados pessoais por padrão; incidente de demonstração (`dfir-app demo`); **cópia verificada do banco antes de migration** (RF-032, ADR-0026).
- **Supply chain (2026-11-18, `EVD-012`).** `cargo audit` e `cargo deny --workspace check` passam com uma vulnerabilidade aceita por `WVR-004` (`RUSTSEC-2026-0459`, `libsqlite3-sys 0.37.0`) e duas exceções informativas documentadas: `RUSTSEC-2025-0141` (`bincode`, via `yara-x`) e `RUSTSEC-2026-0543` (`chrono 0.4.45`, sem CVE). Todas têm análise de alcance; SQLite e chrono têm revisão até 2026-12-31 (`docs/SUPPLY_CHAIN.md`).
- **Engine contido (ADR-0024).** Processo separado, ambiente reduzido, **um único arquivo de entrada**, timeout, `kill_on_drop`. **No Linux** o engine aplica a si mesmo, antes de ler a evidência, `no_new_privs`, **Landlock** (só lê o arquivo do job) e **seccomp** (sem rede, sem `execve`, sem `ptrace`...): provado com o engine real, que recebe `EACCES`/`EPERM` onde deve. **No Windows** há Job Object (memória, sem processo filho, sem interface), **ainda nunca executado lá**, e **não** há confinamento de arquivo e rede: todo job registra o aviso `SANDBOX_NOT_FULLY_APPLIED`. Isto contém falha do parser; não é proteção contra tudo, e para evidência de origem desconhecida a recomendação segue sendo uma máquina dedicada e offline.
- **Adiado por decisão do operador (ADR-0025):** importação CSV/JSONL (F1-27) e, portanto, fontes de rede, SIEM, dispositivos de segurança e e-mail/identidade na nuvem. A ferramenta analisa **EVTX**; qualquer outro arquivo é preservado, não analisado.
- **Limites que valem repetir:** a detecção só enxerga o que o log registrou (auditoria de processo, de compartilhamento e da plataforma de filtragem precisam estar ligadas) e **a ausência de candidato não prova ausência de ataque**; **sem log de rede não há como confirmar exfiltração**, só a tentativa de usar a ferramenta. As regras foram vistas em EVTX real sem ataque e em eventos sintéticos, **nunca em um incidente real**.
- **Nada disso está `ACCEPTED`:** falta o operador rodar em Windows, em Linux aarch64 e sobre um incidente real (`docs/FASE1_STATUS.md`).

## MVP

O fluxo funcional atual cobre:

- SQLite/WAL e migrations;
- storage local e staging;
- audit chain append-only;
- BLAKE3-256-v2-JCS para a cadeia de auditoria; BLAKE3-256-v1 permanece verificável; SHA-256 continua reservado aos fluxos de evidência/compatibilidade explicitamente exigidos.
- processo separado para o engine;
- protocolo JSONL versionado e streaming por batches;
- limites de execução: tempo (`DFIR_ENGINE_TIMEOUT_SECONDS`), memória (`DFIR_ENGINE_MAX_MEMORY_BYTES`), tamanho de importação (`DFIR_MAX_UPLOAD_BYTES`), margem de disco (`DFIR_MIN_FREE_DISK_BYTES`), profundidade do JSON (`MAX_JSON_DEPTH`, 256, em `dfir-protocol`) e concorrência (`DFIR_ENGINE_MAX_CONCURRENT_JOBS`); sem limite de saída nem truncamento (ADR-0020);
- processo separado para o parser, com validação de escopo de arquivos e limites de execução;
- importação local de evidências via diálogo nativo de arquivo, preservação em storage controlado e hash imediato BLAKE3 + SHA-256;
- catálogo de evidências e análise EVTX disparada pela interface;
- incidentes e análise de Fase 1 pela interface: NIST CSF (profiles e gap analysis), ATT&CK, caminho crítico, impacto, IOCs, recomendações e exportação de achados para arquivo JSON local;
- jobs persistidos e timeline gravada no SQLite, paginada e filtrada no banco (ADR-0021);
- interface nativa em Rust (eframe/egui/egui_extras) — sem navegador, sem toolchain de frontend.

> Gap origem: INC-09 (2026-11-04). A lista anterior citava limites de linha, saída e itens, removidos pela ADR-0020, e uma timeline "derivada dos resultados", substituída pela ADR-0021.

EVTX é parseado pela crate `evtx` (requisito `0.15.1`, lock em 0.15.1); BinXML não é reimplementado.

## Interface

A interface é nativa (`src/gui/`, eframe/egui/egui_extras) e chama a camada de serviço (`src/service/`) diretamente no mesmo processo — não há mais servidor, rota HTTP nem JSON no fio. Ver ADR-0017 para a decisão completa e `docs/NATIVE_UI_ARCHITECTURE.md` para a arquitetura da UI (thread de backend Tokio, canais Command/Event, telas).

## Configuração

Variáveis principais:

- `DFIR_DATA_ROOT`
- `DFIR_ENGINE_PATH` - caminho explícito do `dfir-engine`. Sem ela, o app procura `dfir-engine` (`dfir-engine.exe` no Windows) na **mesma pasta do próprio `dfir-app`** — o layout de `target/{debug,release}/` e de uma distribuição autocontida, sem administrador nem passo de instalação — e, no Linux/macOS, cai para `/usr/local/bin/dfir-engine`. O caminho resolvido é registrado no log de inicialização (ADR-0015).
- `DFIR_ENGINE_MAX_LINE_BYTES` (padrão e máximo: 8 MiB; moldura do protocolo, não limite de saída)
- `DFIR_ENGINE_TIMEOUT_SECONDS`
- `DFIR_ENGINE_MAX_CONCURRENT_JOBS`
- `DFIR_ENGINE_MAX_MEMORY_BYTES` - cap de memória do processo do engine, **acrescido do tamanho do arquivo de entrada** (ADR-0021: o YARA-X mapeia o arquivo em memória). Em Unix é `RLIMIT_AS`; no Windows o app passa o valor por `DFIR_ENGINE_MEMORY_LIMIT_BYTES` e o engine o aplica ao seu Job Object (ADR-0024).
- `DFIR_ENGINE_SANDBOX` - `required` (padrão no Linux: se uma camada do sandbox não puder ser aplicada, o engine não analisa), `best-effort` (padrão nos demais sistemas: aplica o que existe e o job registra o que faltou) ou `off` (depuração). Valor desconhecido **recusa a partida** (ADR-0024).
- `DFIR_MAX_UPLOAD_BYTES` - limite de importação de evidência (padrão 512 MiB).
- `DFIR_MIN_FREE_DISK_BYTES` - margem mínima de espaço livre no volume de dados (padrão 1 GiB; 0 desliga; ADR-0021).
- `RUST_LOG` - nível dos logs (padrão `info`; ADR-0022).
- `DFIR_ENGINE_INPUT_FILE`, `DFIR_ENGINE_INPUT_ROOT`, `DFIR_ENGINE_OUTPUT_ROOT` - **internas**: o app as define para cada processo do engine; não configure à mão.

## Validação

```bash
just validate-fase0
just fmt
just check
just test
just nextest
just clippy
```

Supply chain:

```bash
just supply-chain
```

Runtime:

```bash
just runtime-test
just runtime-security
```

Nenhum gate deve ser descrito como executado sem evidência real do ambiente correspondente.

## CLI

```bash
cargo run -- audit-verify
cargo run -- engine-smoke
```

Sem subcomando, `dfir-app` abre a interface nativa. `audit-verify` verifica tanto a audit chain quanto a chain of custody de todas as evidências armazenadas. A lista completa dos 20 subcomandos está em "Linha de comando (headless)", acima.

## Segurança

A segurança principal do produto está na fronteira de processamento da evidência:

```text
arquivo de evidência
      ↓
engine (processo separado por job; tempo/memória limitados; saída sem truncamento, com tetos opcionais)
      ↓
resultado JSONL validado e persistido sem truncamento
      ↓
aplicação local
```

Evidências originais não são apagadas pela aplicação. O staging possui lifecycle próprio. A cadeia de auditoria é tamper-evident e usa `BEGIN IMMEDIATE` para serializar writers.
A importação de evidência, seu evento `EVIDENCE_IMPORTED` e a primeira entrada de chain of custody são persistidos na mesma transação SQLite. As transições de início/conclusão/falha de análise também atualizam o job e suas trilhas de proveniência de forma atômica.
