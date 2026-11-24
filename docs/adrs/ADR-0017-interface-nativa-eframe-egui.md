<!-- doc-type: decision | status: accepted -->
# ADR-0017 — Interface nativa (eframe/egui/egui_extras) substitui o par Axum + `frontend/` TypeScript

## Status

ACEITA — 2026-09-22

## Contexto

Desde a Fase 0 (ADR-0005) a interface era um par: `dfir-app` servia HTML/JSON via Axum em `127.0.0.1`, e um frontend TypeScript compilado (`frontend/`, ADR-0009 trocou Bun por Node/npm no toolchain de build) rodava no navegador do próprio operador e falava com esse servidor por `fetch`. Era, na prática, um cliente-servidor local rodando na mesma máquina, para o mesmo único operador — a fronteira de rede existia só porque HTML/JS é o que um navegador executa.

A demanda explícita desta mudança foi: "implemente interface nativa em Rust com eframe/egui com egui_extras, substituindo a interface web." Por `AGENTS.md` (ordem de precedência: requisito explícito da demanda > corpo normativo do spec > decisões/ADRs > docs derivados), este pedido do operador — que é ao mesmo tempo arquiteto e implementador deste projeto — tem prioridade sobre o contrato de frontend descrito em `docs/PROJECT_SPEC_v6_4_5.md` (seção F.3.1) e sobre as ADR-0005/0009, que ficam superseded por esta.

Motivação de fundo (não apenas "porque foi pedido"): um binário nativo elimina a fronteira de rede (mesmo loopback-only), o parsing de multipart, o toolchain de build de frontend inteiro (Node/npm/esbuild/vitest — ADR-0009 inteira fica sem objeto) e a necessidade de content-security-policy/escaping HTML (Askama existia só por isso). Em troca, ganha um único processo, sem porta aberta nem em loopback, mais simples de raciocinar para uma ferramenta forense de operador único.

## Decisão

### Camada de serviço extraída de `web/mod.rs`

A lógica de negócio dos handlers Axum (~2300 linhas: hashing+transação de importação de evidência, verificação de integridade antes de analisar, as transações `BEGIN IMMEDIATE`/auditoria/custódia, NIST profile+gap, ATT&CK mapping, caminho crítico, matriz de impacto, IOCs, recomendações, exportação) foi portada **linha a linha**, não reescrita, para `src/service/` (`mod.rs`, `nist.rs`, `findings.rs`) — funções `async` simples, sem HTTP, chamadas diretamente pela GUI no mesmo processo. `src/persistence/`, `src/application/` (engine/process) e `src/infrastructure/storage.rs` não mudaram.

A única mudança de comportamento real é a importação de evidência: em vez de multipart de um upload de navegador, a GUI abre um diálogo nativo de arquivo (`rfd`) e `import_evidence` lê o caminho local escolhido — o mesmo hashing BLAKE3+SHA-256 em streaming e a mesma transação de importação continuam.

### Interface nativa em `src/gui/`

`eframe 0.33.3` + `egui 0.33.3` + `egui_extras 0.33.3` (tabelas via `TableBuilder`) + `rfd 0.17.2` (diálogos nativos de abrir/salvar arquivo, backend `xdg-portal` no Linux — sem exigir GTK do sistema). Versões escolhidas por serem o release mais recente de cada uma com MSRV declarado ≤ 1.88 — abaixo do `rust-version = "1.97"` fixado no `Cargo.toml` do projeto, mas ainda compatível com ele; a escolha não teria sido diferente com um toolchain 1.97 disponível.

Arquitetura: a UI do egui roda na thread principal (obrigatório em macOS) e é síncrona (`update()` uma vez por quadro); a camada de serviço é `async`. Uma thread dedicada (`gui::backend::spawn`) hospeda um runtime Tokio próprio e processa um enum `Command` recebido por canal, devolvendo um enum `Event` por outro canal que a UI drena sem bloquear (`try_recv`) a cada quadro. Nenhuma chamada de rede ou de banco acontece na thread da UI.

Telas: "Evidência" (lista, importar via diálogo nativo, destruir, disparar EVTX_TIMELINE/YARA_SCAN, timeline filtrável, resultados YARA) e "Incidentes" (lista/criação + abas NIST CSF, ATT&CK, Caminho crítico, Impacto, IOCs, Recomendações, Exportar) — mapeamento 1:1 com as telas que existiam em `frontend/src/app.ts`/`incident-ui.ts`/`timeline.ts`, agora removidos.

### O que foi removido, não apenas desviado

- `src/web/` (handlers Axum), `templates/` (HTML + o template Askama de CLI), `frontend/` (TypeScript/Node/npm) inteiros.
- Dependências: `axum`, `tower`, `tower-http`, `askama`. Askama só existia pelo escaping automático em contexto HTML; sem HTML, a única saída de texto restante (`engine-smoke`) virou um `format!` direto — manter uma dependência de compilação de templates para uma linha de texto deixou de ter a necessidade demonstrada que `CONVENTIONS.md` exige.
- `AppConfig::bind`, `DFIR_BIND`, `DFIR_BIND_ALLOW_NON_LOOPBACK`: não existe mais porta nenhuma para vincular.
- O toolchain Node/npm inteiro (`toolchains.toml`) fica sem nenhum consumidor no projeto.

### O que fica pendente, deliberadamente fora do escopo desta ADR

- Uma tela de histórico/diff entre versões de profile NIST (o service layer já devolve o histórico completo em `ProfileHistoryView`; a UI atual mostra só a versão vigente).
- Navegar resultados de jobs de análise antigos fora da sessão atual (mesma limitação que a UI web tinha: resultado só aparece logo após disparar a análise).
- `cargo deny`/`cargo audit`/`cargo vet` sobre a nova árvore de dependências (accesskit, winit, ashpd/zbus via `rfd`): não executados nesta sessão — ver "Evidência de execução".

## Consequências

Positivas: um processo, sem porta de rede nem em loopback; sem toolchain de frontend; leitura de arquivo local direta em vez de multipart; menos dependências (rede: -axum/-tower/-tower-http/-askama; +eframe/+egui/+egui_extras/+rfd).

Negativas: a interface nativa não é acessível remotamente (a web, mesmo loopback-only, permitia портforwarding manual; isso nunca foi um requisito, mas era possível). Telas construídas com `TableBuilder` para evidência/timeline; as listas menores (IOCs, mapeamentos ATT&CK, recomendações, estágios do caminho crítico) usam `ui.group`/rótulos simples em vez de tabela — suficiente para o volume esperado, mas menos denso visualmente que uma tabela completa.

## Evidência de execução

Ambiente de verificação: container de desenvolvimento sem o toolchain fixado em `rust-toolchain.toml` (1.97.1); `rustc`/`cargo` 1.91.1 instalados via `apt` para esta sessão, usados com `--ignore-rust-version` (a MSRV declarada de `sqlx 0.9` e de várias dependências transitivas já excedia 1.91 antes desta mudança — pré-existente, não introduzido aqui). O operador deve re-executar os gates abaixo com o toolchain 1.97.1 real antes de considerar isto fechado.

- `cargo check -p dfir-app`: limpo, 0 erros, 0 warnings.
- `cargo test -p dfir-app --bin dfir-app`: **93/93 testes verdes** (inclui os testes novos de `error.rs`/`cli.rs` e todos os testes pré-existentes da camada de persistência, inalterados).
- `cargo fmt --all -- --check`: limpo.
- `cargo check --workspace --all-targets` (inclui `dfir-engine`/`dfir-protocol`, não tocados por esta mudança): ver `docs/FASE1_STATUS.md` para o resultado registrado após esta sessão.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: ver `docs/FASE1_STATUS.md`.
- Não executados nesta sessão: `cargo nextest`, `cargo audit`, `cargo deny` (exigem instalar essas ferramentas via `cargo install`, não presentes no ambiente de verificação) — mesmo estado PENDING-HUMAN já registrado em sessões anteriores deste projeto para essas mesmas ferramentas.
- `scripts/validate_fase0.py` e `scripts/doc_gate.py` foram atualizados nesta sessão para não depender mais de `frontend/`/Askama (ver commits desta sessão); re-executados e verdes — ver `docs/FASE1_STATUS.md`.

## Adendo — 2026-10-02 (ADR-0018)

1. **Classificação corrigida.** O texto acima chama o pedido do operador de "requisito explícito da demanda" e invoca a precedência de `AGENTS.md` §1. Isso estava incorreto: a demanda original (SAGA SENAI 12412, `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md`) não menciona tecnologia de interface. A troca para eframe/egui é uma **decisão do operador** (arquitetura, nível 3), válida e mantida, registrada por esta ADR e pela nota (5) da especificação v6.5.0.
2. **Versões.** A versão `0.33.3` citada acima foi a decidida em 2026-09-22. O `Cargo.toml` passou a exigir `0.36.2` para `eframe`/`egui`/`egui_extras` e o `Cargo.lock` foi regenerado em 2026-10-02 (ver `CHANGELOG.md` 6.5.0). A API `eframe::App` correspondente usa `fn ui`, não `update`.

## Adendo — 2026-11-24 (ADR-0029)

A [**ADR-0029**](ADR-0029-manter-eframe-egui.md) reavaliou a interface nativa e decidiu manter `eframe/egui/egui_extras` **0.37** (versão vigente em 2026-11-18). As versões anteriores `0.33.3` (decisão original, 2026-09-22) e `0.36.2` (atualização via ADR-0018, 2026-10-02) são preservadas como histórico neste documento. Nenhuma migração de framework foi iniciada.
