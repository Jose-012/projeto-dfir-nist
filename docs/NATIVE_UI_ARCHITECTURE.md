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
| `mod.rs` | `DfirApp` (impl `eframe::App`), navegação responsiva (Evidência / Incidentes), **janela de preferências** com pré-visualização imediata de tema, **breadcrumbs** derivados da tela e das seleções/aba vigentes, `Loadable<T>` (estado idle/loading/ready/failed reutilizado por toda tela de lista), `enum_combo` e as listas de valores válidos de cada enum (mesmas aceitas por `src/persistence/`). `format_timestamp` é só apresentação: RFC3339 UTC (`Z` ou `+00:00`, com ou sem frações) vira `AAAA-MM-DD HH:MM:SS UTC`; valores inválidos ou com offset não UTC permanecem inalterados e a persistência/serialização não muda. |
| `backend.rs` | `Command`/`Event`, `Backend::spawn` (thread + runtime Tokio), `handle_command` (um braço por `Command`, chama `src/service/`). |
| `file_dialog.rs` | Seletor nativo de arquivo/pasta **fora da thread da UI** (GUI-01): `FilePicker` cria a thread, escolhe o backend (`auto|zenity|rfd` via `DFIR_FILE_DIALOG_BACKEND`, sonda `zenity` no Linux e cai para `rfd`), pede `rfd::FileDialog`/`zenity` e devolve o resultado por canal; `poll()` é drenado a cada quadro. O resultado é um `enum Picked` explícito (`Path`, `Cancelled`, `Unavailable`, `Failed`). `rfd` legado não distingue cancelamento de falha do seletor; o Zenity distingue os quatro casos. A UI trata cancelamento como "nada escolhido" e trata `Unavailable`/`Failed` preservando diagnóstico específico e oferecendo os caminhos alternativos (digitar caminho, arrastar arquivo). |
| `evidence.rs` | Lista de evidências (`egui_extras::TableBuilder`), importação (seletor de `file_dialog.rs`, campo de caminho digitado e arrastar-e-soltar, todos por `request_import`), destruição, disparo/cancelamento de análise (EVTX, YARA e **Detectar**) e resultados YARA. Importação, custódia e ações importantes usam o timestamp UTC legível e descrições contextuais; o descarte explica que marca `DESTROYED` e preserva a cadeia de custódia. O catálogo vazio orienta a próxima ação; diretórios e importações concorrentes são recusados sem expor componentes do caminho nos avisos. `Command::CancelAnalysis` não bloqueia a UI; `CANCELLED` não recarrega timeline nem resultados analíticos. O painel recolhível "Detecções por regras" delega para `detection.rs`; cada resultado YARA pode abrir sua linhagem por `lineage.rs`. |
| `lineage.rs` | Estado e apresentação somente leitura da linhagem analítica: `LineageState` controla `Idle/Loading/Ready/Failed`, envia `Command::LineageForResult` e correlaciona a resposta por resultado e `LineageScope`. Mostra resultado selecionado, histórico do mais antigo ao vigente e relações de saída, sem despejo JSON cru, com criação em UTC legível. Uma falha permite retry no mesmo resultado; trocar o contexto limpa a vista anterior. É usada tanto por resultados da evidência quanto por itens do plano do incidente. |
| `detection.rs` | Revisão dos candidatos de detecção (F1-28, ADR-0025): `DetectionState` por evidência (lista carregada uma vez, filtro de situação, no máximo 100 candidatos desenhados), cada candidato com regra, ATT&CK, eventos de origem, IOCs e o que a regra não prova; **Confirmar** (exige incidente) e **Rejeitar** (exige motivo) enviam `Command::ConfirmCandidate`/`RejectCandidate`. Candidato decidido não tem botões. Nada aqui cria achado sem o clique. A lista usa a mesma viewport vertical da tela de evidência e revela controles ao ganhar foco. |
| `timeline.rs` | Timeline paginada (ADR-0021; filtro estruturado `campo:valor`, ADR-0025: a sintaxe é validada **na interface antes de enviar** e o erro aparece ao lado da caixa, sem derrubar a timeline carregada): `TimelinePager` (cache de até 48 páginas de 200 linhas, sem dependência de egui) e `TimelineView` (tabela virtualizada de uma linha por evento, filtro aplicado no SQLite, avisos do parser em destaque). O timestamp visível e seu hover usam `format_timestamp`. Fala com o backend por `Command::TimelinePage` / `Event::TimelinePageLoaded`; respostas de filtro ou job antigos são descartadas. Testes sem janela com `egui_kittest` (ADR-0022). |
| `id_regression_tests.rs` | Testes incluídos por `#[path]` no módulo de testes de `mod.rs`: colisão de IDs em seletores e arestas de caminho crítico, com fontes estritas e sem tolerar o glifo de diagnóstico do `egui`. Não contém código de produção. |
| `incidents.rs` | Lista/criação de incidentes e a casca de abas que delega para os painéis F1-03/04/05 abaixo. A lista começa em `Idle`, com contagem neutra e ação **Carregar lista**; só `Ready` mostra a contagem real. Carregar/tentar novamente muda imediatamente para `Loading`, e `Failed` preserva o diagnóstico com retry. As 13 abas, as ações de lista/criação e a auditoria têm descrições contextuais; timestamps de auditoria usam o formato UTC legível. |
| `deliverables.rs` | Aba "Entregáveis" (F1-07/F1-12): pré-visualização dos cinco documentos, escolha da pasta de saída (também por `file_dialog.rs`, fora da thread da UI), geração, máscara de dados pessoais e lista de arquivos com SHA-256/tamanho. |
| `plan.rs` | Itens do plano de fortalecimento (F1-29): formulário, oito domínios, revisões e cobertura por domínio. Novos itens exigem pré/pós-condição, verificação, `source` e `confidence`; o formulário começa em `NOT_VERIFIED`/`LOW`. Registros legados sem os dois últimos campos mostram "não informado (registro legado)". Cada item permite inspecionar linhagem no escopo do incidente. |
| `gaps.rs` | Lacunas de fonte (F1-31): declaração, revisão e situação por período, sem confundir "não medido" com "sem lacuna". |
| `effects.rs` | Efeitos visuais limitados da interface nativa: transições de ação com duração normalizada a no máximo 180 ms, progresso monotónico em `0.0..=1.0` e resolução imediata quando o operador pede movimento reduzido; **gradiente curto de seleção apenas em bordas (`selection_gradient`/`masked_edge`, recortado pelo mesmo `CornerRadius` do controle para não reintroduzir cantos quadrados)**, sem animação ambiente. Declarado como `mod effects;` em `mod.rs`. |
| `guidance.rs` | Painel compartilhado de primeiro uso para estados vazios: título, explicação, passos reais do fluxo e uma ação opcional, usando os tokens e ícones vetoriais existentes. A única revelação é de opacidade, limitada por `style.animation_time` e imediata com duração zero/Reduced Motion; não desliza nem mantém animação ambiente. O componente apenas devolve o clique: a tela continua dona do seletor e dos comandos, e Incidentes não cria dados automaticamente. Declarado como `mod guidance;` em `mod.rs`. |
| `icons.rs` | Catálogo local de ícones vetoriais determinísticos para a interface nativa (geometria sem `Context` nem fontes externas; `MAX_SHAPES_PER_ICON` limita primitivas). Usado em `mod.rs` para a navegação (**`AppIcon::Evidence`**, **`AppIcon::Incidents`** + `paint_icon`); declaração `mod icons;` com `dead_code` permitido para variantes ainda não referenciadas pela UI. |
| `preferences.rs` | Persistência local versionada das preferências da UI (`ui-preferences.json`): tamanho limitado, UTF-8, JSON estrito, versão de esquema e validação de domínio; ficheiro tratado como entrada não confiável. Declarado como `pub mod preferences;` em `mod.rs`; consome os tipos de `super::theme`. |
| `runtime.rs` | `UiWake`: wakeup partilhado entre a thread de backend e a thread egui. Produtores assíncronos notificam depois de entregar um resultado no canal; a UI chama `acknowledge` antes de drenar. Coalesce rajadas, preserva eventos anteriores ao contexto egui e não usa timer periódico em idle (substitui o `request_repaint_after` fixo). Declaração `pub mod runtime;` em `mod.rs`; usado por `backend.rs` e `file_dialog.rs`. |
| `theme.rs` | Tema e tokens da interface nativa: `ThemePreset` (Light/Dark/GoldenWind/Custom), `BaseTheme`, `CustomAccent` com validação de domínio (hue 0..360, chroma 0..0.32) e `ThemeTokens`. Serializado em *snake_case* para estabilidade do arquivo de preferências; `preferences.rs` consome estes tipos. `ThemeTransition` interpola tokens entre presets com duração de 160 ms (`MOTION_DURATION` no módulo, definida como `Duration::from_millis(160)`) dirigida pelo relógio do egui (`animate_bool_with_time`), sem repaint residual após o fim; `ThemeTokens::new` apenas define `motion: Duration::ZERO` quando `reduced_motion` está ativo, e `DfirApp::transition_theme` em `mod.rs` aplica o alvo sem interpolação nesse caso. Declarado como `mod theme;` em `mod.rs` (módulo privado do crate gui). |
| `nist.rs`, `attack.rs`, `critical_path.rs`, `impact.rs`, `iocs.rs`, `recommendations.rs`, `export.rs` | Um painel por área funcional de incidente (F1-03/04/05); cada um segue o mesmo padrão: `ensure_loaded` (carrega ao trocar de incidente), `on_*` (aplica `Event`), `ui` (formulário + lista). |

## Camada de serviço (`src/service/`)

Porta 1:1 dos antigos handlers Axum, sem HTTP: `mod.rs` (evidência, análise, timeline, incidentes, helper `current_results_for_incident` compartilhado), `nist.rs` (profile + gap analysis), `findings.rs` (ATT&CK, caminho crítico, impacto, IOCs, recomendações, exportação), `lineage.rs` (histórico e relações estritas por escopo) e `plan.rs` (itens de hardening estruturados, sem executor). Ver ADR-0017 para o que mudou de comportamento (só a importação de evidência, que agora lê um arquivo local escolhido pelo operador em vez de multipart).

## Segurança

A interface não abre porta de rede alguma — nem em loopback. Não há CSP nem escaping HTML porque não há HTML: toda a árvore de widgets é construída em Rust tipado. Evidências e paths físicos continuam não expostos além do necessário (a UI mostra `storage_reference`/nome original, não o caminho absoluto no disco). O diálogo de arquivo (`rfd`, encapsulado em `file_dialog.rs`) é o único ponto onde o processo lê um caminho escolhido interativamente pelo operador — mesmo modelo de confiança de qualquer aplicativo desktop nativo; o caminho digitado ou arrastado passa pela mesma validação (`request_import`) e continua sendo entrada não confiável. No Linux, o diálogo depende de `xdg-desktop-portal` ou `zenity`; a ausência deles **não** congela a janela nem impede a importação pelos caminhos alternativos (ver `docs/OPERATION.md` §3D).

## Reavaliação de framework (ADR-0029)

A stack `eframe/egui` foi mantida após auditoria prática e leitura dos módulos. Os defeitos de rolagem, foco, IDs e jobs foram corrigidos sem nova dependência. Slint, GTK4, Qt, Iced e uma volta ao frontend web exigiriam reescrita e validação sem benefício demonstrado para o escopo atual. A decisão completa e os critérios para reabrir a comparação estão na ADR-0029.

A feature `accesskit` do `eframe` permanece desativada: a árvore interna usada pelo `egui_kittest` não é AT-SPI. Não se declara compatibilidade com leitor de tela externo.

## Limitações conhecidas do MVP

- NIST tem seletores independentes e somente leitura para percorrer o histórico de Current e Target, mas não há diff lado a lado. A seleção mostra metadados e entradas da versão escolhida, sinaliza quando não é a vigente e não recalcula gaps nem altera a versão usada em novas análises.
- Resultado de análise só é visível na sessão em que foi disparada (mesma limitação da UI web anterior).
- Listas menores (IOCs, mapeamentos ATT&CK, recomendações, estágios do caminho crítico) usam `ui.group`/rótulos em vez de `TableBuilder` — ver ADR-0017, "Consequências".

## Testes e observabilidade (ADR-0022)

- **Testes de tela sem janela:** `egui_kittest` (dev-dependency) renderiza a tela e opera pela árvore de acessibilidade (clicar, digitar, consultar rótulos). Padrão em `gui/timeline.rs` (`ui_tests`): `Harness::new_ui_state` com `(TimelineView, Backend)`, `Backend::for_test()` para capturar os `Command` enviados e `TimelinePage::for_test()` para alimentar a tela. Estados com spinner repintam sem parar: usar `step()` em vez de `run()`. Digitar exige focar o campo antes (`click()`) e um quadro depois para o texto chegar ao estado. **Os testes de transição (`theme_transition_interpolates_and_finishes_without_repaint_loop`, `crossfade_progress_is_bounded_and_advances`, `crossfade_progress_never_exceeds_one`) usam `HarnessBuilder::default().with_step_dt(1.0 / 60.0).with_max_steps(32)` porque o `step_dt` padrão do harness avança 250 ms por frame, o que tornaria a duração de 150–180 ms imprecisa para assertivas de progresso.**
- **Primeiro `Id` animado:** no egui 0.37.0, o primeiro `animate_bool_with_time(id, true, ...)` de um `Id` desconhecido retorna `1.0`; `Crossfade::progress` semeia `false` antes de pedir `true` para não saltar a primeira seleção. O teste `crossfade_reuses_id_after_clear` também cobre `clear()` seguido da reutilização do mesmo `Id`, com `with_step_dt(1.0 / 60.0)` e progresso limitado e não decrescente. A revelação de `guidance.rs` usa o mesmo seed, com testes de progresso temporal, opacidade final repetida e duração zero imediata.
- **Orientação de estado vazio:** `guidance_fits_narrow_width_and_returns_the_action_click` verifica conteúdo, ação e geometria em 420 px. Os testes integrados de Evidências preservam uma única ação `Importar evidência…` e expõem `Escolher arquivo…`; o de Incidentes verifica os próximos passos e a ausência de comandos. Essas assertivas não substituem revisão estética humana nem AT-SPI.
- **Logs:** `infrastructure/logging.rs` grava JSON em `<data_root>/logs/dfir.log` (rotação 5 × 5 MiB, 0600) além do stderr, e registra panics. A GUI não tem console, então o arquivo é o único rastro de campo. Mudanças reais de tela registram somente o enum (`Evidence`/`Incidents`); páginas de timeline registram somente id da evidência, offset, existência da página e contagem de eventos. A importação registra `native UI import path edited` apenas com `non_empty`, `native UI import requested` sem campos de dados e `native UI import finished` apenas com `success`; esses eventos permitem verificar o fluxo nativo sem registrar o caminho. Nunca registrar nome de arquivo de evidência nem conteúdo de eventos.
- **GPU:** `gui/mod.rs::log_renderer` registra, na inicialização, o adaptador escolhido pelo `wgpu` (`adapter`, `backend`, `device_type`, `driver` e o número de candidatos) — GFX-01. O log de `wgpu_hal` para backends que falharam (por exemplo EGL em GPU antiga) continua visível: é diagnóstico real, não ruído a esconder; a linha do adaptador diz qual backend ficou em uso e se a renderização caiu para CPU (`llvmpipe`).
- **Diagnóstico:** `dfir-app diagnostics` (sem nomes de arquivo nem mensagens de erro).
