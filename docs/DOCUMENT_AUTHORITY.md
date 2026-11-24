<!-- doc-type: reference | status: current -->
# Manifesto de autoridade documental — DFIR/NIST / v6.5.1

Este arquivo define a autoridade por domínio e as fronteiras entre documentação normativa, estado e evidência.

| Domínio | Autoridade | Derivados/auxiliares |
|---|---|---|
| Demanda original | `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md` (transcrição da demanda SAGA SENAI 12412; PDF original: PENDING-HUMAN) | `docs/DEMAND_TRACE.md` |
| Requisitos de negócio | demanda original + corpo normativo da especificação | `docs/DEMAND_TRACE.md`, RTM, índice, prompt |
| Escopo e poda | ADR-0018 + seção 39 da especificação | `docs/FASE1_BACKLOG.md` |
| Arquitetura | `docs/PROJECT_SPEC_v6_5_1.md` | ADRs, índice XML |
| UI nativa | `Cargo.toml` (`eframe`/`egui`/`egui_extras`/`rfd`) + evidência de toolchain efetiva | `src/gui`, `docs/NATIVE_UI_ARCHITECTURE.md` |
| Python tooling | documentação do ambiente + evidência de `python3 --version` | scripts/tests |
| Estado | `docs/rtm.md` (única matriz de estado de ticket e requisito) | `docs/FASE1_STATUS.md` (somente resumo cronológico; log completo até 2026-10-29 em `docs/archive/FASE1_STATUS_historico_2026-10.md`), `docs/FASE0_STATUS.md` (matriz de capacidades da Fase 0, histórico), prompt, changelog |
| Rastreabilidade | `docs/rtm.md` | testes, código, ADRs |
| Gates | `docs/RELEASE_GATE.md` + `docs/EVIDENCE_REGISTER.md` | `justfile`, CI quando existir |
| Regras LLM | `AGENTS.md` + seção 45 + Apêndice F; escolha de harness de agente: ADR-0027 | `CONVENTIONS.md`, `docs/SESSION_IMPROVEMENT.md`, `docs/SESSION_LEARNINGS.md`, `docs/COORDINATION_PLAYBOOK.md`, `.jcode/skills/`, `.jcode/swarm-prompt.md` |
| Toolchain declarada | `toolchains.toml` + `rust-toolchain.toml` | docs de tooling |
| Toolchain real | evidência `rustc`, `cargo`, `python3` | manifests de referência |

> Gap origem: AUS-03 (2026-11-04). `docs/EVIDENCE_REGISTER.md` passou a existir; os `EVD-*` e os `WVR-*` são registrados nele.

### Documentos de apoio

Documentos que apoiam a mudança somente quando são afetados (`AGENTS.md` §11). Não se cria narrativa duplicada de estado. Gap origem: AUS-01 e INC-03 (2026-11-04); simplificação em 2026-11-18.

| Documento | Papel | Autoridade que o governa |
|---|---|---|
| `docs/FASE1_STATUS.md` | log cronológico da Fase 1 | Estado: `docs/rtm.md` |
| `docs/MVP_IMPROVEMENT_PLAN.md` | propostas priorizadas de melhoria, sem estado ou requisito próprio | Demanda, especificação, ADRs e RTM; ativação exige decisão explícita |
| `docs/FINAL_ANALYSIS_CHECKLIST.md` | checklist consolidado das análises de código/documentação, com fatos, reproduções e destino de cada recomendação | Demanda, especificação, ADRs e RTM; ativação de cada item exige decisão explícita |
| `CONVENTIONS.md` | convenções de desenvolvimento para agentes LLM | Regras LLM: `AGENTS.md` |
| `.jcode/skills/dfir-improve/SKILL.md` | checklist derivado do loop e da seleção de skills | `AGENTS.md` §16/§17 + `docs/SESSION_IMPROVEMENT.md` |
| `.jcode/swarm-prompt.md` | template derivado para missão, lanes e integração de workers | Regras LLM: `AGENTS.md` §17; não autoriza escopo ou ações externas |
| `docs/COORDINATION_PLAYBOOK.md` | workflows derivados de coordenação Cricket/Fox/especialistas, ferramentas, evidência e recuperação | Regras LLM: `AGENTS.md` §15-17; não cria requisito, autorização ou aceite |
| `docs/SESSION_IMPROVEMENT.md` | procedimento obrigatório de aprendizado entre sessões | Regras LLM: `AGENTS.md` §16 |
| `docs/SESSION_LEARNINGS.md` | registro vivo e sanitizado de lições operacionais, sem estado de release | `AGENTS.md` §16 + `docs/SESSION_IMPROVEMENT.md` |
| `docs/DEFINITION_OF_READY.md` | ponteiro para o checklist histórico Fase 0 → Fase 1 | Gates: `docs/RELEASE_GATE.md` |
| `docs/DEMO.md` | dataset e roteiro de demonstração | Requisitos de negócio: demanda e especificação |
| `docs/OPERATION.md` | instalação de referência, backup e atualização manual (RF-030, RF-032) | Arquitetura: especificação e ADR-0026 |
| `docs/DETECTION_RULES.md` | regras de detecção (gerado de `dfir-engine --list-rules`) | Arquitetura: ADR-0025 |
| `docs/API.md` | lápide da API HTTP removida, com o mapa para o serviço | Arquitetura: ADR-0017 |
| `docs/SUPPLY_CHAIN.md` | política e registro de supply chain | Gates: `docs/RELEASE_GATE.md` |
| `README.md` | visão geral e uso | Estado: `docs/rtm.md` |
| `RELATORIO_ALTERACOES_*` | relatórios históricos; novos não são obrigatórios por rotina | narrativa; Git log registra as rodadas |
| `.scratch/fase1-backlog/issues/` | tickets `F1-NN`, um arquivo por ticket | Escopo e poda: ADR-0018; estado: `docs/rtm.md` |
| `tests/*/*.md` | documentação de testes da Fase 0 | Gates: `docs/RELEASE_GATE.md` |

## Classes de runtime


**Build Toolchain**: Rust/Cargo (Node.js/npm foi removido junto com o frontend TypeScript — ver ADR-0017; ADR-0009, sobre Bun→Node, fica só de valor histórico).

**Engineering/QA Tooling**: CPython e Shell.

A presença de uma ferramenta de build/QA não significa que ela deva existir na imagem de produção.

## Regras de conflito

1. Identificar o domínio da informação.
2. Consultar a autoridade daquele domínio.
3. Corrigir o artefato derivado.
4. Atualizar RTM/status/ADR quando o conflito for normativo.

Nunca escolher uma versão apenas porque um arquivo é mais recente.

## Staleness

Um documento de revisão pontual (`REVIEW_*`, `FINAL_REVIEW_*`, `IMPLEMENTATION_REVIEW_*`) cujos achados já foram todos incorporados ao estado corrente (`docs/FASE0_STATUS.md`, `docs/FASE1_STATUS.md`, ADRs) é histórico, não normativo. Ele deve carregar o aviso `> **Status: HISTÓRICO / SUPERSEDED.**` no topo e ser movido para `docs/archive/`, seguindo o mesmo padrão já usado para versões antigas da especificação. Uma decisão registrada em ADR que é substituída por uma decisão posterior (ex.: ADR-0017 sobre a interface) não é reescrita: recebe `supersedes`/nota "SUPERSEDED BY ADR-XXXX" no topo do arquivo, preservando o conteúdo histórico — mesmo mecanismo já usado no corpo da especificação canônica (notas de implementação superseded).

### Tag de indexação para LLM

Documentos fora da especificação canônica (ADRs, status, gates, README, AGENTS.md etc.) podem trazer uma linha `<!-- doc-type: ... | status: ... -->` logo após o título, para indexação rápida por LLMs/scripts (vocabulário fechado de `doc-type`: `overview`, `llm-rule`, `procedure`, `decision`, `status`, `backlog`, `gate`, `architecture`, `reference`, `log`, `review`, `source`; vocabulário fechado de `status`: `current`, `superseded`, `partially-superseded`, `historical`, `accepted`, `proposed`, `rejected`, `deferred`; valor fora desses conjuntos exige atualizar este documento — Gap origem: INC-17, 2026-11-04). Essa tag é derivada e não normativa — não substitui a tabela de autoridade acima nem os tipos semânticos (`requirement`, `decision`, `constraint`, etc.) definidos no Apêndice F da especificação canônica, que continuam sendo o vocabulário normativo para blocos dentro do corpo do documento.

## IDs

IDs normativos não são reutilizáveis. Decisões substituídas usam `supersedes`.

### Famílias de ID

Registro das famílias de ID usadas no projeto (Gap origem: INC-17 e AUS-01, 2026-11-04). As famílias normativas de `AGENTS.md` §10 (`RF-*`, `D*`, `LLM-*`, `ADR-*`) seguem definidas na especificação e em `docs/adrs/`.

| Família | Significado | Onde é definida | Natureza |
|---|---|---|---|
| `AUD-*` | achados da auditoria de 2026-10-02 | `docs/archive/REVIEW_2026-10-02-auditoria-demanda.md` (34 IDs; `AUD-03a`, `AUD-03b`, `AUD-35` e `AUD-36` são citados em CHANGELOG, FASE1_STATUS e ADR-0020 e não são definidos nesse documento) | local, não normativa |
| `DOC-*` | achados de documentação da mesma revisão (`DOC-01` a `DOC-06`) | mesmo documento | local, não normativa |
| `DEM-*` | itens da demanda original (`DEM-<grupo>-NN`; grupos `FONTE`, `ID`, `TL`, `CR`, `NIST`, `ENT`, `PLAN`) | `docs/DEMAND_TRACE.md` | rastreabilidade |
| `RB-*` | restrições da demanda original (`RB-001` a `RB-006`) | `docs/DEMAND_TRACE.md` | rastreabilidade |
| `F1-*` | tickets do backlog da Fase 1 (`F1-01` a `F1-35`) | `.scratch/fase1-backlog/issues/`; escopo em `docs/FASE1_BACKLOG.md`; estado em `docs/rtm.md` | local |
| `R-*` | regras de detecção (`R-<CATEGORIA>-NNN`) | `docs/DETECTION_RULES.md`, ADR-0025 | local do produto |
| `EVD-*` | evidência de execução de gate | `docs/EVIDENCE_REGISTER.md`; especificação, Apêndice F.9 | normativa (`AGENTS.md` §10) |
| `WVR-*` | waiver de gate | `docs/EVIDENCE_REGISTER.md`; especificação, Apêndice F.14 | normativa (`AGENTS.md` §10) |
| `GATE-*`, `TEST-*`, `CA-*` | gate, teste e critério de aceitação | reservadas por `AGENTS.md` §10 e pela especificação; nenhuma instância definida em 2026-11-04 (gap AUS-03) | normativa, reservada |


Toolchains de referência: `toolchains.toml`.

