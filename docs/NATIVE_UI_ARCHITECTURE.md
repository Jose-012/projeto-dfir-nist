<!-- doc-type: architecture | status: current -->
# Interface nativa - arquitetura (substitui FRONTEND_ARCHITECTURE.md, ADR-0017)

## Objetivo

Fornecer uma interface para o analista como parte do mesmo binário Rust, sem navegador, sem servidor HTTP (nem em loopback) e sem toolchain de frontend.

## Camadas

```text
src/gui/*.rs (eframe/egui/egui_extras)
  ↓ chamada de função direta, in-process
src/service/*.rs (async, sem HTTP)
  ↓
src/persistence/* + src/application/* (inalterados)
```

## Por que uma thread de backend

`egui` roda na thread principal (obrigatório em macOS) e é síncrono: `eframe::App::ui()` é chamado uma vez por quadro e não pode ser `.await`ado. A camada de serviço (`src/service/`) é `async` (SQLx, subprocesso do engine). `src/gui/backend.rs` resolve isso com uma thread dedicada que hospeda seu próprio runtime Tokio:

```text
thread da UI (egui)                    thread de backend (Tokio)
  DfirApp::ui()                      AppServices { db, paths, engine }
    ├─ backend.drain_events() ◄──────────┤ event_tx.send(Event::…)
    │    (try_recv, não bloqueia)        │      ▲
    ├─ desenha o quadro                  │      │ tokio::spawn por Command
    └─ backend.send(Command::…) ─────────┼──────┘
                                          │ command_rx.recv().await
```

Cada `Command` (ex.: `Command::Analyze`) vira uma task Tokio independente que chama a função correspondente em `src/service/` e devolve um `Event` (ex.: `Event::AnalyzeCompleted`) pelo canal de eventos. A UI nunca bloqueia esperando o Tokio: cada quadro drena os eventos pendentes com `try_recv()` e aplica o resultado ao estado da tela correspondente.

## Módulos de `src/gui/`

| Arquivo | Papel |
|---|---|
| `mod.rs` | `DfirApp` (impl `eframe::App`), navegação (Evidência / Incidentes), `Loadable<T>` (estado idle/loading/ready/failed reutilizado por toda tela de lista), `enum_combo` e as listas de valores válidos de cada enum (mesmas aceitas por `src/persistence/`). |
| `backend.rs` | `Command`/`Event`, `Backend::spawn` (thread + runtime Tokio), `handle_command` (um braço por `Command`, chama `src/service/`). |
| `file_dialog.rs` | Seletor nativo de arquivo/pasta **fora da thread da UI** (GUI-01): `FilePicker` cria a thread, pede `rfd::FileDialog` e devolve o resultado por canal; `poll()` é drenado a cada quadro. O `rfd` não distingue cancelamento de falha do seletor, então a UI trata os dois como "nada escolhido" e oferece os caminhos alternativos. |
| `evidence.rs` | Lista de evidências (`egui_extras::TableBuilder`), importação (seletor de `file_dialog.rs`, campo de caminho digitado e arrastar-e-soltar, todos por `request_import`), destruição, disparo de análise (EVTX, YARA e **Detectar**) e resultados YARA. O painel recolhível "Detecções por regras" delega para `detection.rs`. |
| `detection.rs` | Revisão dos candidatos de detecção (F1-28, ADR-0025): `DetectionState` por evidência (lista carregada uma vez, filtro de situação, no máximo 100 candidatos desenhados), cada candidato com regra, ATT&CK, eventos de origem, IOCs e o que a regra não prova; **Confirmar** (exige incidente) e **Rejeitar** (exige motivo) enviam `Command::ConfirmCandidate`/`RejectCandidate`. Candidato decidido não tem botões. Nada aqui cria achado sem o clique. |
| `timeline.rs` | Timeline paginada (ADR-0021; filtro estruturado `campo:valor`, ADR-0025: a sintaxe é validada **na interface antes de enviar** e o erro aparece ao lado da caixa, sem derrubar a timeline carregada): `TimelinePager` (cache de até 48 páginas de 200 linhas, sem dependência de egui) e `TimelineView` (tabela virtualizada de uma linha por evento, filtro aplicado no SQLite, avisos do parser em destaque). Fala com o backend por `Command::TimelinePage` / `Event::TimelinePageLoaded`; respostas de filtro ou job antigos são descartadas. Testes sem janela com `egui_kittest` (ADR-0022). |
| `incidents.rs` | Lista/criação de incidentes e a casca de abas que delega para os painéis F1-03/04/05 abaixo. |
| `deliverables.rs` | Aba "Entregáveis" (F1-07/F1-12): pré-visualização dos cinco documentos, escolha da pasta de saída (também por `file_dialog.rs`, fora da thread da UI), geração, máscara de dados pessoais e lista de arquivos com SHA-256/tamanho. |
| `plan.rs` | Itens do plano de fortalecimento (F1-29): formulário, oito domínios, revisões e cobertura por domínio. |
| `gaps.rs` | Lacunas de fonte (F1-31): declaração, revisão e situação por período, sem confundir "não medido" com "sem lacuna". |
| `nist.rs`, `attack.rs`, `critical_path.rs`, `impact.rs`, `iocs.rs`, `recommendations.rs`, `export.rs` | Um painel por área funcional de incidente (F1-03/04/05); cada um segue o mesmo padrão: `ensure_loaded` (carrega ao trocar de incidente), `on_*` (aplica `Event`), `ui` (formulário + lista). |

## Camada de serviço (`src/service/`)

Porta 1:1 dos antigos handlers Axum, sem HTTP: `mod.rs` (evidência, análise, timeline, incidentes, helper `current_results_for_incident` compartilhado), `nist.rs` (profile + gap analysis), `findings.rs` (ATT&CK, caminho crítico, impacto, IOCs, recomendações, exportação). Ver ADR-0017 para o que mudou de comportamento (só a importação de evidência, que agora lê um arquivo local escolhido pelo operador em vez de multipart).

## Segurança

A interface não abre porta de rede alguma — nem em loopback. Não há CSP nem escaping HTML porque não há HTML: toda a árvore de widgets é construída em Rust tipado. Evidências e paths físicos continuam não expostos além do necessário (a UI mostra `storage_reference`/nome original, não o caminho absoluto no disco). O diálogo de arquivo (`rfd`, encapsulado em `file_dialog.rs`) é o único ponto onde o processo lê um caminho escolhido interativamente pelo operador — mesmo modelo de confiança de qualquer aplicativo desktop nativo; o caminho digitado ou arrastado passa pela mesma validação (`request_import`) e continua sendo entrada não confiável. No Linux, o diálogo depende de `xdg-desktop-portal` ou `zenity`; a ausência deles **não** congela a janela nem impede a importação pelos caminhos alternativos (ver `docs/OPERATION.md` §3D).

## Limitações conhecidas do MVP

- Sem tela de histórico/diff entre versões de profile NIST (o dado existe em `ProfileHistoryView`; a UI mostra a versão vigente).
- Resultado de análise só é visível na sessão em que foi disparada (mesma limitação da UI web anterior).
- Listas menores (IOCs, mapeamentos ATT&CK, recomendações, estágios do caminho crítico) usam `ui.group`/rótulos em vez de `TableBuilder` — ver ADR-0017, "Consequências".

## Testes e observabilidade (ADR-0022)

- **Testes de tela sem janela:** `egui_kittest` (dev-dependency) renderiza a tela e opera pela árvore de acessibilidade (clicar, digitar, consultar rótulos). Padrão em `gui/timeline.rs` (`ui_tests`): `Harness::new_ui_state` com `(TimelineView, Backend)`, `Backend::for_test()` para capturar os `Command` enviados e `TimelinePage::for_test()` para alimentar a tela. Estados com spinner repintam sem parar: usar `step()` em vez de `run()`. Digitar exige focar o campo antes (`click()`) e um quadro depois para o texto chegar ao estado.
- **Logs:** `infrastructure/logging.rs` grava JSON em `<data_root>/logs/dfir.log` (rotação 5 × 5 MiB, 0600) além do stderr, e registra panics. A GUI não tem console, então o arquivo é o único rastro de campo. Nunca registrar nome de arquivo de evidência nem conteúdo de eventos.
- **GPU:** `gui/mod.rs::log_renderer` registra, na inicialização, o adaptador escolhido pelo `wgpu` (`adapter`, `backend`, `device_type`, `driver` e o número de candidatos) — GFX-01. O log de `wgpu_hal` para backends que falharam (por exemplo EGL em GPU antiga) continua visível: é diagnóstico real, não ruído a esconder; a linha do adaptador diz qual backend ficou em uso e se a renderização caiu para CPU (`llvmpipe`).
- **Diagnóstico:** `dfir-app diagnostics` (sem nomes de arquivo nem mensagens de erro).
