<!-- doc-type: log | status: current -->
# Lições reutilizáveis entre sessões

Registro operacional sanitizado governado por `docs/SESSION_IMPROVEMENT.md` e `AGENTS.md` §16. Não é matriz de estado, evidência de release ou narrativa completa de conversas. As entradas abaixo nasceram da rodada de verificação residual concluída nos commits `b45c922`, `d30ef0c` e `adad4c2`.

## 2026-11-19 - verificação residual DFIR

Todas as confirmações começam como **pendentes** para a próxima sessão aplicável. A correção feita na própria rodada não conta como confirmação futura do loop.

### 1. Referências antecipadas a evidências

- **Classe:** erro de execução.
- **Fato:** documentos chegaram a mencionar `EVD-051` e `EVD-052` antes dos respectivos arquivos e resultados finais existirem. A reconciliação posterior corrigiu isso antes do commit.
- **Risco:** uma referência vigente poderia parecer `PASS` sem execução correspondente.
- **Prevenção:** IDs futuros ficam em scratch; arquivo e registro precedem qualquer referência vigente, seguidos de novo `doc-gate` (`docs/SESSION_IMPROVEMENT.md`, início e execução).
- **Confirmação:** confirmada em 2026-11-19: `EVD-054` teve arquivo criado antes do registro e das demais referências; `just doc-gate` final passou.

### 2. Investigação gráfica sem orçamento eficaz

- **Classe:** erro de execução.
- **Fato:** a lane gráfica consumiu cerca de 50 minutos com múltiplas técnicas de injeção/observação antes de receber ordem de consolidação. O resultado útil final foi `EVD-050 BLOCKED`, não um fluxo validado.
- **Risco:** pesquisa aberta, consumo de contexto e atraso sem nova informação proporcional.
- **Prevenção:** orçamento explícito no plano; duas tentativas sem informação nova mudam o método; limites padrão de 15/30 minutos e consolidação obrigatória.
- **Confirmação:** confirmada em 2026-11-19: a lane do swarm-prompt teve orçamento de 20 minutos, recebeu pedido explícito de consolidação, foi inspecionada após timeout e parada por ID completo; o root só assumiu a escrita após confirmar a ausência de artefato.

### 3. Exit de helper tratado como possível sucesso funcional

- **Classe:** erro de método evitado no resultado final.
- **Fato:** cliques X11 tiveram exit 0 enquanto a tela permaneceu `Screen::Evidence`. O smoke final de resize também teve exit 0 e emitiu `Xlib.error.BadMatch`.
- **Risco:** transformar envio de evento em falsa prova de navegação ou resize limpo.
- **Prevenção:** toda ação exige pós-condição independente e inspeção de diagnósticos; `EVD-050` registra janela/resize apenas como resultado parcial e bloqueado.
- **Confirmação:** confirmada em 2026-11-19: `gh run watch` foi seguido de confirmação MCP do run e das 27 etapas; o diagnóstico do doc-gate foi lido e uma divergência de ticket foi corrigida antes do PASS.

### 4. Evidência vigente sensível a advisories e premissas antigas

- **Classe:** risco herdado revelado pela sessão.
- **Fato:** a nova execução encontrou oito vulnerabilidades de `wasmtime 48.0.8`, invalidando o supply-chain vigente de `EVD-041`. Dry-runs também mostraram que a premissa de `WVR-004` estava superada por SQLx 0.9.1.
- **Risco:** carregar um `PASS` histórico como atual sem repetir o gate ou reavaliar a exceção.
- **Prevenção:** revisar gatilhos de `STALE` no início e repetir supply-chain quando lock/advisories mudarem; exceções exigem dry-run e justificativa atual.
- **Confirmação:** confirmada em 2026-11-19: a execução 44567609855 recarregou 1550 advisories e repetiu audit/deny; o resultado foi registrado separadamente em `EVD-054`.

### 5. Binário executado sem vínculo com o HEAD modificado

- **Classe:** limitação identificada e delimitada.
- **Fato:** o primeiro e2e com CPython 3.14.6 usou binários pré-existentes identificados por SHA-256 e não provou a árvore posteriormente modificada. `EVD-049` declara esse limite; a reconstrução e o novo e2e foram registrados separadamente em `EVD-052`.
- **Risco:** atribuir ao HEAD um resultado obtido com outro binário.
- **Prevenção:** toda evidência deve ligar commit/tree, hashes dos binários e comando de build, ou dizer explicitamente que o binário era pré-existente.
- **Confirmação:** confirmada em 2026-11-19: `EVD-054` liga commit `7acf795`, tree `122de0f` e build no checkout limpo do runner; a execução 44576017693 repetiu o fluxo no commit documental `172cd70`.

### 6. Busca vazia incompatível com o conteúdo conhecido

- **Classe:** erro de ferramenta/método.
- **Fato:** buscas por IDs conhecidos retornaram zero em caminhos que continham esses IDs. O resultado não foi usado para provar ausência; a revisão seguiu por leitura e `git diff`.
- **Risco:** concluir que referências obsoletas não existem por falha de caminho, índice ou ferramenta.
- **Prevenção:** controle positivo conhecido ou método alternativo obrigatório para qualquer zero inesperado usado em decisão.
- **Confirmação:** pendente.

### 7. IDs de coordenação usados incorretamente

- **Classe:** erro de execução.
- **Fato:** um `bg wait` usou um ID de await incompleto e retornou `Task not found`. Uma tentativa de parar workers com parâmetro inadequado também falhou; o worker CPython já não existia quando outra parada foi tentada.
- **Risco:** esperar ou encerrar o alvo errado, ou acreditar que a coordenação terminou quando a chamada falhou.
- **Prevenção:** copiar IDs completos, usar `todo.waiting_on`, consultar `swarm list`/`bg list` e verificar o resultado efetivo de cada parada. Após a recorrência, `AGENTS.md` §16, `docs/SESSION_IMPROVEMENT.md`, `dfir-improve` e `swarm-prompt` passaram a vedar explicitamente IDs de `await_members` em `bg wait`.
- **Confirmação:** recorrente em 2026-11-19: um ID completo de `await_members` foi usado em `bg wait` e retornou `Task not found`. O estado foi verificado por `swarm list`, e a distinção de canais foi promovida nos checklists.

### 8. Encurtamento acidental de linhas históricas da RTM

- **Classe:** erro de execução detectado antes do commit.
- **Fato:** a primeira edição para acrescentar `EVD-050..052` também removeu detalhes históricos de algumas linhas. A revisão do diff identificou a perda e restaurou o texto, mantendo os acréscimos.
- **Risco:** apagar rastreabilidade enquanto se faz uma atualização aparentemente pequena.
- **Prevenção:** edições aditivas e revisão semântica do diff, não apenas `git diff --check`.
- **Confirmação:** confirmada em 2026-11-19: `git diff --numstat` e a revisão das linhas editadas preservaram o histórico da RTM; somente F1-12/F1-14/F1-15 mudaram, com `doc-gate` final verde.

### 9. Silêncio de build confundível com travamento

- **Classe:** risco de interpretação, sem cancelamento indevido.
- **Fato:** watchdogs emitiram stall durante compilação de `nextest`, embora a tarefa continuasse ativa e depois terminasse com 536 passed/1 skipped.
- **Risco:** cancelar trabalho saudável ou duplicar build pesado em um host com memória limitada.
- **Prevenção:** silêncio dispara inspeção de progresso/processo, não cancelamento automático; builds pesados permanecem sequenciais.
- **Confirmação:** pendente.

### 10. Pin de action sem a ferramenta no manifesto da tag

- **Classe:** erro de execução.
- **Fato:** a primeira execução do CI falhou porque a `install-action` pinada não oferecia `cargo-nextest 0.9.151` para Linux x86_64; os gates posteriores foram skipped. A inspeção do manifesto em uma tag atual confirmou versão/hash e a execução seguinte passou (`EVD-054`).
- **Risco:** confundir SHA completo e versão exata com disponibilidade garantida do binário, ou esconder a falha usando fallback não controlado.
- **Prevenção:** `docs/TOOLCHAINS.md` exige conferir versão e hash no manifesto da tag exata antes da atualização; o workflow mantém checksum e `fallback: none`, e a evidência preserva a falha inicial.
- **Confirmação:** pendente.

### 11. Cache de build e E2E disputam o orçamento de disco

- **Classe:** erro de execução e guarda fail-closed.
- **Fato:** um run falhou com ENOSPC ao restaurar aproximadamente 1,88 GiB de cache contendo `target/`; o seguinte passou nos demais gates, mas o E2E rejeitou corretamente 965 MiB livres diante da guarda de 1 GiB. Após restringir o cache e executar `cargo clean`, o run final removeu 7,8 GiB, ficou com 14 G livres e concluiu 110/110 etapas (`EVD-055`).
- **Risco:** quebrar o runner por cache de artefatos derivados ou enfraquecer a guarda de disco para transformar uma falha legítima em PASS.
- **Prevenção:** o workflow não cacheia `target/`, mede espaço antes/depois de `cargo clean` e mantém `DFIR_MIN_FREE_DISK_BYTES` e `DFIR_ENGINE_SANDBOX=required`; `docs/TOOLCHAINS.md` registra a regra.
- **Confirmação:** pendente.

### 12. Worker automático combinado com spawn explícito

- **Classe:** erro de execução.
- **Fato:** em 2026-11-19, uma lane com `lane_owner = "worker"` iniciou um worker automático e um `swarm spawn` explícito iniciou outro para a mesma entrega. Os dois chegaram a progresso 1/2 sobre o mesmo alvo antes da duplicidade ser detectada. O worker automático concluiu; o duplicado foi parado por ID completo e a entrega foi revisada.
- **Risco:** escritas concorrentes, resultado ambíguo e consumo duplicado de contexto/recursos.
- **Prevenção:** `AGENTS.md` §17 exige escolher um único mecanismo de início por lane, com dono exclusivo de escrita e confirmação de progresso/parada; `CONVENTIONS.md` e `CLAUDE.md` ligam a regra, e o `doc_gate` verifica sua presença.
- **Confirmação:** confirmada em 2026-11-19: uma única lane usou spawn explícito com início automático desativado; `swarm list` confirmou um único worker e depois sua parada por ID completo, sem duplicação.

### 13. Transição de todo sem efeito repetida

- **Classe:** erro de execução.
- **Fato:** em 2026-11-19, várias chamadas de `todo update` mantiveram os itens em `in_progress`, embora a intenção fosse concluí-los. A divergência ficou visível nos estados retornados, mas a mesma forma de chamada foi repetida antes da mudança de método.
- **Risco:** churn de coordenação, falsa expectativa de fechamento e perda de tempo sem nova evidência.
- **Prevenção:** `.jcode/skills/dfir-improve/SKILL.md` exige ler o estado retornado após mutações de todo/swarm/background e aplicar o limite de duas tentativas sem informação nova antes de mudar de método.
- **Confirmação:** confirmada em 2026-11-20: transições de timeline, NIST e RTK para `completed` e das novas lanes para `in_progress` foram lidas no estado retornado, sem repetição da chamada ineficaz histórica.
- **Confirmação adicional em 2026-11-22:** após a perda repetida do campo `waiting_on` em chamadas diretas, `codemode` construiu o patch com ID e razão explícitos e o estado retornado confirmou a pausa correta, sem novas tentativas idênticas. Na mesma rodada, antes de um fallback de modelo, o root repetiu a suíte atual com `JCODE_SCRATCH_DIR` explícito; os dois testes, incluindo positivo e oito mutações, passaram e o fallback foi revogado antes de criar trabalho duplicado.

### 14. Escrita atrasada após parada do worker

- **Classe:** erro de coordenação revelado pela revisão do gate.
- **Fato:** em 2026-11-19, uma versão do prompt escrita pelo root foi substituída posteriormente por uma entrega atrasada do worker que já havia sido parado. Um needle de `await_members` falhou no gate e o controle positivo direto revelou a mudança de conteúdo antes do commit.
- **Risco:** perder uma entrega válida, sobrescrever correções ou commitar uma versão cuja autoria e conteúdo não foram revisados.
- **Prevenção:** `AGENTS.md` §17, `docs/SESSION_IMPROVEMENT.md`, `dfir-improve` e `swarm-prompt` exigem confirmar estado terminal e reler o artefato antes de assumir sua escrita; o `doc_gate` preserva essa salvaguarda e a distinção de IDs de await.
- **Confirmação:** confirmada em 2026-11-20: após a parada terminal do worker timeline, o root releu `evidence.rs` imediatamente antes de assumir a escrita, revisou a consolidação e integrou a expansão sem sobrescrever o diff anterior.

### 15. Spawn visível abriu fora do checkout informado

- **Classe:** erro de coordenação detectado antes de qualquer escrita dos workers.
- **Fato:** em 2026-11-20, três lanes visíveis receberam `working_dir`, mas terminaram inicialmente sem relatório/diff; uma delas informou estar em `/home/jose`. Não houve alteração de arquivos. O coordenador corrigiu as mesmas sessões por DM com o checkout absoluto, sem respawn.
- **Risco:** trabalhar em cópia errada, procurar artefatos descartáveis ou duplicar workers enquanto a lane real está ociosa.
- **Prevenção:** `AGENTS.md` §17 e `.jcode/swarm-prompt.md` passaram a exigir checkout absoluto no prompt e confirmação de `git rev-parse --show-toplevel`/HEAD antes da primeira edição. Correção de diretório ocorre na sessão existente.
- **Confirmação:** confirmada em 2026-11-20 nos dois spawns seguintes (`breadcrumbs-native` e `svg-deterministico`): ambos usaram início explícito sem modo automático, retomaram as mesmas sessões por DM após o retorno inicial sem relatório e confirmaram `pwd`, `git rev-parse --show-toplevel` e HEAD no checkout absoluto antes da primeira edição. Não houve respawn nem duplicação.

### 16. Round-trip de leitura anotada e saída truncada

- **Classe:** erro de execução detectado antes do commit.
- **Fato:** em 2026-11-20, uma transferência `read` → `write` gravou os prefixos de numeração apresentados pela leitura no inventário de licenças. A recuperação por Base64 também encontrou o limite de 30.000 caracteres da saída do bash. Leitura direta e `cmp` detectaram o problema; três blocos completos restauraram o texto e o destino ficou byte a byte igual ao scratch validado.
- **Risco:** um exit 0 de escrita poderia transformar representação anotada ou truncada em documento normativo aparentemente regenerado.
- **Prevenção:** `docs/SESSION_IMPROVEMENT.md` §2 exige fonte bruta, atenção ao limite do canal e `cmp`/hash entre origem e destino antes do gate; conteúdo parcial não pode ser gravado como completo.
- **Confirmação:** confirmada em 2026-11-22: uma montagem `read` → `codemode` encontrou zero headings porque a leitura continha prefixos de linha. O controle positivo CP-00/CP-02 falhou antes da escrita; após remover somente a apresentação, os 29 blocos foram extraídos e o canônico foi relido e validado pelo linter. Nenhum prefixo de numeração foi gravado no documento.

### 17. Envelope MCP incompleto repetido sem nova informação

- **Classe:** erro de execução e de aderência ao critério de parada.
- **Fato:** em 2026-11-20, chamadas diretas ao dispatcher `mcp` foram emitidas sem o objeto `arguments` e repetiram erros de propriedade obrigatória em `order` e `route`. A mudança para `codemode`, com `arguments:{action,args}` ou `arguments:{task}` explícito, executou os mesmos fluxos read-only com sucesso.
- **Risco:** confundir erro do envelope com indisponibilidade da ferramenta e consumir o orçamento repetindo uma chamada inválida.
- **Prevenção:** `AGENTS.md` §15 documenta o envelope completo, e `docs/SESSION_IMPROVEMENT.md` §2 exige validar o objeto aninhado antes da chamada e mudar de método quando o erro não trouxer informação nova. Após a recorrência, `dfir-improve` passou a exigir `action` explícita em `skill_manage`, e o `doc_gate` verifica essa salvaguarda além da referência a `tools.mcp`.
- **Confirmação:** recorrente em 2026-11-20: chamadas `skill_manage` pretendidas como reload/list omitiram `action`, caindo no default de `load` ou no erro de `name` ausente. A mudança para `codemode` tornou `action` explícita, mas o `reload` retornou `not found` e não foi contado como sucesso. A leitura dos arquivos confirmou sua integridade. A prevenção foi generalizada para actions obrigatórias e promovida a um needle do gate. A fixture do novo needle passou no baseline com exit 0 e, após a remoção da frase, falhou com exit 1 e o diagnóstico específico ``dfir-improve safeguard missing: Em `skill_manage`, declare `action` explicitamente``.
- **Escopo confirmado:** a investigação posterior na fonte `10114429c` e no runtime mostrou que skills do projeto formam um overlay relido do disco em `list`/`read`/`load`, enquanto `reload` e `reload_all` renovam somente globais. O `reload` da global `optimization` passou, e o da local `dfir-improve` retornou `not found` sem alterar seu conteúdo. Portanto, esse `not found` era escopo esperado, não arquivo ausente nem indisponibilidade da ferramenta. `AGENTS.md`, `CONVENTIONS.md`, `dfir-improve` e o procedimento agora orientam `read`/`load` após editar locais, e o gate preserva a regra do overlay. A fixture do novo needle passou com exit 0 e, após a remoção da frase, falhou com exit 1 e `dfir-improve safeguard missing: overlay local é relido do disco`. A instrução global foi criada em `~/.jcode/preferred-tools.md`, camada documentada para novas sessões.

### 18. Guia/help divergente do comportamento da versão instalada

- **Classe:** risco de instrução desatualizada revelado por controles reais.
- **Fato:** o guia do jcodemunch citava `get_importers`, rejeitado pelo servidor 1.108.469; o `menu` atual expõe `find_importers`, e `get_call_hierarchy` com `depth=1` produziu o controle positivo de 34 callees diretos. No RTK 0.49.0, `rewrite` imprimiu equivalências com `rc=3`, embora o help indicasse `0` para comandos suportados. `/usr/bin/git-delta --version` retornou 129 e usage de `git diff`, confirmando que não era o pager `delta`.
- **Risco:** institucionalizar nomes, exits ou substitutos de ferramenta que não funcionam no host, mesmo quando a documentação de outra versão os descreve.
- **Prevenção:** `AGENTS.md` §15 agora registra versões, ações confirmadas no `menu`, recuperação por `rtk recall`, divergência de exit do `rewrite` e a identidade do homônimo `git-delta`. `docs/SESSION_IMPROVEMENT.md` exige versão, stdout, exit e pós-condição para esse tipo de divergência.
- **Confirmação:** pendente.

### 19. Default global confundido com modelo efetivo por role

- **Classe:** limitação de auditoria resolvida por configuração, sidecar e fonte da versão instalada.
- **Fato:** em 2026-11-20, o harness 1.0.1 (`10114429c8133994861d6c915a2483d397c0f7c2`) apresentou coordenador `moonshotai/kimi-k3.1-pro`; swarm efetivo `nvidia-nim:mistralai/mistral-large-4-1t-a52b`; review efetivo `openrouter:particle-ai/particle-1:free`, embora seu default global fosse `nvidia-nim:moonshotai/kimi-k3.1-pro`; judge efetivo/global `nvidia-nim:nvidia/nemotron-3.5-super-vl-120b-a12b`; e ambient global `nvidia-nim:nvidia/nemotron-3.5-lightning-30b-a3b`, sem override de sessão. Memory não tinha pin global nem de sessão e usa auto-seleção de backend, não um identificador fixo presumido. Os defaults globais de autoreview, autojudge e ambient estavam `enabled=false`. Swarm tinha `swarm_model_selection=false` e allowlist vazia. As variáveis de ambiente correspondentes aos roles estavam ausentes.
- **Risco:** documentar o default global como se fosse o worker efetivo, aplicar o modelo de swarm aos demais roles ou tentar passar `model` quando o schema o remove por política.
- **Prevenção:** a fonte da versão instalada confirmou precedência sessão > global e fallback próprio por role. `AGENTS.md` §17 e `.jcode/swarm-prompt.md` exigem fotografia por `/agents`, `/subagent-model` e `swarm list_models`, com leitura restrita ao sidecar `.agent-models.meta` quando o runtime estiver indisponível. O `doc_gate` verifica as novas salvaguardas.
- **Confirmação:** pendente.

### 20. Gate alterado validado somente no caminho positivo

- **Classe:** lacuna de verificação identificada na revisão das skills.
- **Fato:** os needles de `AGENTS.md` e `swarm-prompt.md` introduzidos no commit `233823d` foram inicialmente verificados somente com o gate positivo. Nesta rodada, uma cópia criada por `git archive` em scratch permitiu violar cada condição isoladamente: remover `get_call_hierarchy` produziu exit 1 e `missing session-improvement instruction: AGENTS.md: get_call_hierarchy`; remover `swarm_model_selection` produziu exit 1 e `missing swarm-prompt requirement: swarm_model_selection`. O checkout real não foi usado para induzir as falhas.
- **Risco:** um needle ausente, incorreto ou desconectado poderia deixar o gate verde sem demonstrar que a regressão pretendida é detectada.
- **Prevenção:** `.jcode/skills/dfir-verify/SKILL.md` item 7 exige controle negativo em cópia sanitizada para mudanças de gate/lint/teste de presença e restringe `edit` à fixture em scratch. O diagnóstico e o exit esperado são pós-condições obrigatórias.
- **Confirmação:** pendente.

### 21. Múltiplos processos Cargo concorrentes sem dono único

- **Classe:** erro de execução.
- **Fato:** 33 processos `cargo` foram disparados pela mesma lane sem aguardar os comandos anteriores movidos para background. A contenção exigiu `TERM` após confirmação do CWD; todos os resultados dessa concorrência foram descartados como `STALE`.
- **Risco:** pressão de memória/swap, diagnósticos misturados, espera em locks e evidência que não corresponde a uma árvore estável.
- **Prevenção:** `dfir-improve` §2 exige dono explícito único para trabalho Rust pesado e `bg wait` no mesmo ID antes de qualquer outro comando Rust. A contenção continua condicionada a progresso, consumo e CWD; relatório de worker não libera outra build.
- **Confirmação:** confirmada em 2026-11-21: Nautilus aguardou cada teste focado antes do seguinte; Calf executou um único `just verify` sequencial (662/662), e o root confirmou ausência de Cargo/Rustc ao final.

### 22. Logs relativos/ausentes e relatórios sem artefato

- **Classe:** erro de método.
- **Fato:** caminhos relativos criaram artefatos em `validation/` dentro do checkout, e relatórios posteriores afirmaram a existência de logs que `read` não encontrou. O root rejeitou esses relatórios; texto de worker não foi convertido em evidência.
- **Risco:** declarar `PASS`/`VERIFIED` sem saída correspondente, commitar artefatos temporários ou perder a ligação entre comando, tree e diagnóstico.
- **Prevenção:** `dfir-improve` §2 e `dfir-verify` §8 exigem caminhos absolutos fora do checkout e leitura posterior de arquivo, exit, contagens e warnings pelo integrador. Relatório sem artefato localizável não é evidência.
- **Confirmação:** confirmada em 2026-11-21: o root rejeitou logs inexistentes, moveu artefatos relativos para fora do checkout e leu os logs absolutos de navegação, efeitos, ícones e `just verify`, incluindo warnings e contagens.

### 23. Teste enfraquecido para passar (run→step, plural→singular, API sem conferir assinatura)

- **Classe:** erro de método.
- **Fato:** um teste de navegação trocou `harness.run()` por `step()` e rótulos plurais por singulares para contornar a falha; a animação também chamou a API do egui como função livre e converteu o resultado para `0/1`, sem confirmar o método/tipo real na fonte da versão locked. O root rejeitou a redução de rigor e exigiu `Context::animate_bool_with_time` com progresso `f32` em segundos.
- **Risco:** teste verde sem exercitar o caminho real e implementação compilável que elimina o comportamento pretendido.
- **Prevenção:** `dfir-change` proíbe reduzir rigor para acomodar implementação e exige assinatura, unidade e retorno na fonte da versão do `Cargo.lock`; `dfir-verify` §8 exige leitura dos logs e invalida concorrência. O integrador revisa o diff semântico dos testes junto com o código.
- **Confirmação:** confirmada em 2026-11-21: a assinatura `Context::animate_bool_with_time(..., bool, f32) -> f32` foi lida no egui 0.37.0; `harness.run()` e rótulos plurais foram restaurados, os testes focados passaram e `just verify` concluiu 662/662 com clippy `-D warnings`.

### 24. Crossfade e navegação: API do egui 0.37.0 e ambiguidade de labels

- **Classe:** erro de método e de execução.
- **Fato:**
  - `ctx.memory(|m| m.is_animating(id))` não existe no egui 0.37.0; causaria erro de compilação se não fosse revertido antes do `cargo test`.
  - `Id::new(...)` está deprecated no egui 0.37.0 (sugere `Id::unique` ou `IdSalt::new`), gerando 5 warnings de `deprecated` nos testes de crossfade.
  - `let mut harness = Harness::new_ui(...)` com `mut` desnecessário gerou warning `unused_mut`.
  - `get_by_label("Evidências")` encontrou dois nós (botão de navegação + label do breadcrumb com o mesmo label), gerando panic "Found two or more nodes".
  - O `nav_label()` de Evidence retorna `"Evidências"` (plural) e de Incidents retorna `"Incidentes"` (plural), mas o teste de breadcrumbs usava singular `"Evidência"`/`"Incidente"`.
  - `std::thread::sleep` nos testes de crossfade não é determinístico para tempo egui; substitui por `Harness::run()`/`step()`.
  - Imports não usados: `UiPreferences` nos testes de navegação e `kittest::Queryable` nos testes de crossfade.
- **Risco:** código com warnings (não limpo para `clippy -D warnings`), testes flaky por tempo real, panic por ambiguidade de labels, testes com rigor enfraquecido (singular em vez do plural real da UI).
- **Prevenção:** consultar fontes do egui 0.37.0 (`grep is_animating`, `grep deprecated`) antes de usar APIs; desambiguar queries por `Role` (`get_by_role_and_label`) ou `query_all_by_label(...).next()` quando houver colisão de labels; usar `nav_label()` (plural) para rótulos visíveis de navegação; evitar `sleep` em testes de UI egui — usar `Harness` para avançar frames; revisar warnings do `cargo test` antes de considerar o gate concluído.
- **Confirmação:** confirmada em 2026-11-21: `navigation_tests4.log` preserva oito warnings anteriores às correções; `effects_tests2.log` e `icons_tests.log` passaram sem warnings nesses dois logs. A execução integrada posterior `final-calf-verify.txt` confirma 662/662, doctest e clippy `-D warnings` sem warnings. O registro anterior não foi reescrito como execução limpa.

### 25. Procedimento substituído por relatório de lane sem falhar o doc-gate

- **Classe:** erro de execução e lacuna de gate.
- **Fato:** `docs/SESSION_IMPROVEMENT.md` foi substituído por um relatório de lane, removendo metadados, loop e procedimento obrigatório; `just doc-gate` ainda terminou com `PASS`. O root restaurou o conteúdo da autoridade e rejeitou o relatório como substituto do procedimento.
- **Risco:** perda silenciosa das instruções de fechamento com aparência de documentação válida.
- **Prevenção:** `scripts/doc_gate.py` exige o `doc-type`, título, loop, três seções do procedimento, formato de lição e antipadrões. Uma cópia sanitizada sem um marcador deve falhar com diagnóstico específico antes do PASS no checkout.
- **Confirmação:** confirmada em 2026-11-21: a cópia válida passou (`doc-gate-preservation-positive.txt`); removendo somente `## Formato mínimo de uma lição`, a cópia falhou com exit 1 e `missing session-improvement instruction` para esse marcador (`doc-gate-preservation-negative.txt`).

### 26. Uso de MCPs e tools: nomes de ação, batched e seleção por tipagem

- **Classe:** lição de execução e seleção de ferramenta.
- **Fato:** em 2026-11-21, um benchmark de 11 casos usou os 3 MCP servers (jcodemunch 6 tools, lean-ctx 12 tools, context7 2 tools) juntos e separados, com tipagem estática/dinâmica e estruturas complexa-mista/simples/híbrida. Duas chamadas `order` falharam usando `order` como nome de action (o front-door recusa a própria action); uma chamada `ctx_callgraph` falhou porque `callgraph` não é ação válida (válidas: `callers`, `callees`, `trace`, `risk`). Chamadas batched reduziram round-trips de N para 1. `ctx_read mode=full` em arquivo >2000 linhas gerou budget warning (37829/20000 tok). `agentgrep paths_only=true` foi a requisição mais barata (apenas caminhos, sem conteúdo).
- **Risco:** usar nomes incorretos de action, consumir o orçamento com arquivos grandes em modo full, confiar em saída truncada como completa, ou repetir a mesma chamada inválida sem mudança de método.
- **Prevenção:** antes de `mcp action=call`, conferir o schema da action via `menu`/`route`; não usar `order` nem `callgraph` como nomes de ação; priorizar chamadas batched para poupar requisições; para arquivos grandes, evitar `mode=full` e usar `mode=symbol/anchored` com ranges ou `get_ranked_context` com `compress=true` e `token_budget`; para descoberta sem alvo conhecido, `ctx_search semantic` com `top_k` baixo; requerer ação explícita (`action` em `skill_manage`, `arguments.{action,args}` no dispatcher MCP) e saída completa antes de considerar o caso concluído.
- **Confirmação:** pendente (benchmark executado em 2026-11-21; os erros de ação foram corrigidos na mesma rodada e os casos A1/A2/B1/B2/C1/C2/D1/D2/E1/E2 concluíram com saída válida).

### 27. Codemode (JS sandbox): tipagem, store/load, paralelo vs sequencial e try/catch

- **Classe:** lição de execução e seleção de ferramenta.
- **Fato:** em 2026-11-21, um benchmark de 12 casos executou codemode (JavaScript sandbox) com tipagem estática/dinâmica, estruturas simples/complexas/aninhadas, store/load, paralelo vs sequencial e try/catch, integrando MCPs (jcodemunch, lean-ctx, context7) e tools nativas (agentgrep). Resultados: C1 static simples 1 chamada/12 ms; C2 static complexo 2 chamadas/281 ms (acesso a propriedades aninhadas); D1 dynamic simples 1 chamada/12 ms; D2 dynamic complexo 2 chamadas/2648 ms (JSON.stringify para busca semântica); M1 complex misturado 3 chamadas/1914 ms (filter/map sobre array tipagem mista); M2 complex aninhado 3 chamadas/1653 ms (report.incident.profile.type); S1 simple individual 1 chamada/13 ms; S2 simples mínimo 1 chamada/1531 ms; H1 store/load 2 chamadas/124 ms (funcionou); H2 paralelo 2626 ms vs sequencial 3876 ms com 4 MCPs (paralelo ~32% mais rápido); H3 try/catch capturou TypeError de JS e erro retornado por tool, ambos recuperáveis. Total: 24 chamadas.
- **Risco:** confiar em codemode para chamadas independentes (não traz vantagem sobre batch), não usar store/load quando precisar de estado entre chamadas, não tratar erro retornado por tool (não é throw) nem erro real de JS, e presumir que o sandbox do codemode conecta servidores MCP locais por stdio (não conecta `lean-ctx` com `ctx_read`/`ctx_search`; o erro é de conexão do sandbox, não de parâmetros nem de servidor remoto).
- **Prevenção:** usar batch para chamadas MCP independentes (fewest requests, menor tempo); usar codemode apenas quando houver store/load intermediário, encadeamento onde o resultado de um passo alimenta o próximo, ou necessidade de try/catch com lógica condicional de JS; para chamadas MCP locais (ex.: `lean-ctx`), usar `batch` ou chamadas diretas fora do `codemode`; acessar propriedades aninhadas com cuidado (tipos podem ser undefined/null); para erros de tool, verificar objeto de retorno (não apenas catch); validar schema da action via menu/route antes de callTool.
- **Confirmação:** confirmada em 2026-11-21: benchmark de casos A–D executado; `codemode` falhou ao conectar `lean_ctx` no sandbox (erro de conexão, corrigido por `batch` com chamadas diretas); `jcodemunch` respondeu via `order`; todos os casos concluíram com saída válida.

### 28. Erros recorrentes de `todo`: fase ausente, nomes duplicados, nomes desconhecidos

- **Classe:** erro de execução recorrente.
- **Fato:** em 2026-11-21, chamadas `todo add` falharam com `added todo needs phase, or an after todo with membership` (fase não informada); `duplicate todo name` (nome reutilizado sem `rename_to` ou `update`); `unknown todo name` (tentativa de atualizar item não registrado no plano). Os três erros ocorreram na mesma sessão de benchmark.
- **Risco:** churn de coordenação, perda de rastreamento de tarefas e falsa impressão de progresso sem itens efetivamente registrados no plano.
- **Prevenção:** antes de `todo add`, sempre informar `phase` ou `after`; antes de `todo update`/`remove`, confirmar que o nome existe no plano atual (ler estado do `todo` primeiro); evitar reutilizar nomes — usar `rename_to` para preservar identidade e dependências. Essa regra é derivada do schema do `todo` (fase ou `after` obrigatórios em `add`; nomes únicos dentro do plano).
- **Confirmação:** recorrente em 2026-11-23: uma atualização usou nome derivado do resumo e retornou `unknown todo name`. O controle operacional foi executar `todo` sem `update`/`add` para ler o plano atual, copiar o nome real `Atualizar evidência e commits` e só então atualizá-lo; nenhum item duplicado foi criado.

### 29. Primeiro `Id` animado e passo temporal padrão do harness

- **Classe:** lição de implementação e de método de teste.
- **Fato:** no egui 0.37.0, o primeiro `animate_bool_with_time(id, true, ...)` de um `Id` desconhecido retorna `1.0`, podendo saltar a primeira transição se não houver seed de `false`. Além disso, o `step_dt` padrão do `egui_kittest` avança 250 ms por frame, intervalo maior que as transições de 150–180 ms; um teste com o padrão pode terminar em um passo sem medir o progresso intermediário.
- **Risco:** perder a animação na primeira seleção ou após `clear()`/reutilização do `Id`, e obter testes verdes que não distinguem interpolação de conclusão imediata.
- **Prevenção:** semear `false` antes do primeiro `true`, cobrir explicitamente `clear()` seguido de reutilização do mesmo `Id` e usar `HarnessBuilder::default().with_step_dt(1.0 / 60.0).with_max_steps(32)` com assertivas de progresso limitado e não decrescente. Não substituir o passo temporal por `sleep` nem enfraquecer a assertiva para acomodar o harness.
- **Confirmação:** confirmada em 2026-11-21: `crossfade_reuses_id_after_clear` cobre o seed após `clear()` e reuso do mesmo `Id`; efeitos passaram 22/22, tema 8/8 e o `just verify` sequencial terminou com 669/669 testes, 1 skipped, doctests e Clippy `-D warnings` sem warnings. Na amostra central de três quadros da GUI isolada, Reduced Motion apresentou somente as cores inicial e final, sem tons intermediários; essa observação nativa não substitui os testes temporais nem comprova estética por RGB.

## Próxima revisão

### 30. Headings do coordination playbook eram contados dentro de fences

- **Classe:** defeito estrutural herdado, corrigido nesta sessão.
- **Fato:** antes da correção, `CP_HEADING` e `ROLE_HEADING` rodavam no texto bruto e contavam headings CP/papel dentro de blocos ``` e ~~~. Isso contradiz o tratamento de conteúdo literal de fences CommonMark e podia criar `duplicate_cp_id` ou satisfazer `missing_role` indevidamente.
- **Prevenção:** `_mask_fenced_blocks` mascara backtick/til, comprimento de abertura/fechamento e EOF preservando offsets e CRLF. A máscara é aplicada a headings e duplicação de prosa, não ao scanner de placeholders, que permanece global para não esconder campos não resolvidos em exemplos.
- **Confirmação:** confirmada nesta sessão por `test_adversarial.py`, incluindo til, fence longo, EOF, papel real removido e offsets CRLF, e pela suíte combinada de 69 testes em `integration-tests-fixed.log`.

### 31. Identidade de módulo em `sys.modules` não é estável entre suítes no mesmo processo

- **Classe:** erro de método evitado.
- **Fato:** `test_adversarial.py` e `test_exhaustive.py` importam o linter com `spec_from_file_location` e registram o mesmo nome `coordination_playbook_gate` em `sys.modules`. No mesmo processo (suíte combinada), o segundo import sobrescreve a entrada, então `assertIs(sys.modules[name], GATE)` pode falhar mesmo com ambas as suítes corretas.
- **Risco:** declarar falha de regressão por identidade de objeto quando o contrato real é apenas "módulo registrado com `validate`".
- **Prevenção:** testes de registro de módulo devem assertar presença em `sys.modules` e existência de `validate()`, não identidade de objeto entre suítes.
- **Confirmação:** confirmada nesta sessão: `test_module_registered_in_sys_modules` foi reescrito para não exigir `assertIs`; a suíte combinada (40 testes) passou com exit 0.

### 32. `FormatChecker` não verifica `date-time` sem o backend opcional

- **Classe:** limitação de verificação revelada por controle negativo.
- **Fato:** a primeira execução com `jsonschema 4.26.0` e `FormatChecker` detectou somente dois de três controles por relatório: `generated_at: not-a-date` passou porque `rfc3339-validator` não estava instalado. Depois da instalação isolada de `rfc3339-validator 0.1.4`, um timestamp válido foi aceito, `not-a-date` foi rejeitado e os 12 controles finais foram detectados.
- **Prevenção:** para cada formato usado, executar um caso válido e um inválido antes de declarar validação completa; registrar implementação, backend, versão e ambiente isolado. `FormatChecker` ativo sem backend não é prova de cobertura do formato.
- **Confirmação:** confirmada nesta sessão em `schema-validation-final-with-formats.log`, com quatro contratos `PASS` e 12/12 controles negativos detectados.

### 33. Ausência de relatório foi inferida no diretório errado

- **Classe:** erro de execução de inventário, corrigido sem reescrever o histórico.
- **Fato:** uma revisão anterior registrou `ox-delivery.json` e `rat-delivery.json` como ausentes, mas ambos já existiam no diretório `drafts/` desde antes da integração. Timestamps e hashes confirmaram que não eram escritas atrasadas.
- **Prevenção:** antes de declarar ausência, enumerar o diretório canônico da entrega, fazer controle positivo em um arquivo conhecido e registrar stat/hash dos artefatos encontrados. Separar original, cópia normalizada e cópia root-repair, sem transformar o erro de lookup em falha do worker.
- **Confirmação:** confirmada nesta sessão em `contract-inventory.log` e `contract-resolution-audit.log`; os quatro originais foram preservados e suas cópias revisadas passaram JSON Schema completo.

### 34. Leitura estática foi marcada como `reproduced` sem controle executado

- **Classe:** erro de método de revisão, rejeitado antes de qualquer patch de produto ou gate.
- **Fato:** uma lane classificou como reproduzida uma alegação de falha mascarada em `just supply-chain`, mas o controle root com etapas 23/0, 0/17 e 23/17 retornou 23/17/17, preservando a última falha. Na mesma entrega, versões antigas do histórico foram descritas como estado vigente, embora a tabela atual já tivesse os valores resolvidos, e um manifesto de scratch foi tratado como arquivo da raiz do checkout. Um candidato root sobre `fs::rename` também foi rejeitado após conferir a documentação da toolchain 1.100, que substitui o destino.
- **Prevenção:** CP-14 e `dfir-verify` exigem comando, exit e diagnóstico para `reproduced`, reservam `static_candidate` à leitura e testam shell com códigos distintos. Também exigem separar vigente/histórico, confirmar inventário rastreado e registrar hipóteses rejeitadas. `dfir-change` limita catálogos Rust a hipóteses e exige comportamento da API na toolchain efetiva.
- **Confirmação:** pendente para a próxima sessão que revisar findings ou aplicar regras externas de Rust. Nesta rodada, os controles citados impediram alterações indevidas em `justfile`, `docs/SUPPLY_CHAIN.md` e `src/gui/preferences.rs`.

### 35. Interação nova de diálogo nativo calibrada fora do repositório

- **Classe:** melhoria de método de verificação nativa.
- **Fato:** a seleção dentro do Zenity não podia ser inferida do smoke anterior, que cobria abrir/cancelar e importação por caminho manual. Uma driver externa e descartável confirmou `Ctrl+L`, outcome `path` e uma linha independente no SQLite antes do patch do harness.
- **Risco:** confundir injeção de teclas ou digitação no campo manual com seleção real pelo diálogo, ou alterar o harness sem antes conhecer a interação do backend.
- **Prevenção:** calibrar a ação e sua pós-condição fora do repositório antes do patch. `scripts/native_x11_smoke.py --import-method zenity` exige outcome `path`, fechamento do diálogo, importação no SQLite e foco devolvido antes das análises; o modo `manual` continua padrão e deve ser repetido como regressão.
- **Confirmação:** confirmada em 2026-11-23 por `EVD-060`: calibração, fluxo Zenity completo e regressão manual completos, com EVTX 12, detecção 5 e página 1/índice 12. O PASS permanece limitado ao backend Zenity deste host.

## Próxima revisão

Na próxima sessão que execute uma das atividades acima, registre ao lado de `Confirmação` o comando/ação que aplicou a prevenção. Se a falha se repetir, marque `recorrente` e proponha o controle executável mínimo; não acrescente outra lição idêntica.
