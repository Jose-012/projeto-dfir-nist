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
- **Confirmação:** pendente.

### 17. Envelope MCP incompleto repetido sem nova informação

- **Classe:** erro de execução e de aderência ao critério de parada.
- **Fato:** em 2026-11-20, chamadas diretas ao dispatcher `mcp` foram emitidas sem o objeto `arguments` e repetiram erros de propriedade obrigatória em `order` e `route`. A mudança para `codemode`, com `arguments:{action,args}` ou `arguments:{task}` explícito, executou os mesmos fluxos read-only com sucesso.
- **Risco:** confundir erro do envelope com indisponibilidade da ferramenta e consumir o orçamento repetindo uma chamada inválida.
- **Prevenção:** `AGENTS.md` §15 documenta o envelope completo, e `docs/SESSION_IMPROVEMENT.md` §2 exige validar o objeto aninhado antes da chamada e mudar de método quando o erro não trouxer informação nova. O `doc_gate` preserva a referência a `tools.mcp`.
- **Confirmação:** pendente.

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

## Próxima revisão

Na próxima sessão que execute uma das atividades acima, registre ao lado de `Confirmação` o comando/ação que aplicou a prevenção. Se a falha se repetir, marque `recorrente` e proponha o controle executável mínimo; não acrescente outra lição idêntica.
