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
- **Confirmação:** pendente.

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
- **Prevenção:** copiar IDs completos, usar `todo.waiting_on`, consultar `swarm list`/`bg list` e verificar o resultado efetivo de cada parada.
- **Confirmação:** confirmada em 2026-11-19: os IDs completos foram usados em `bg wait`; `bg list` confirmou todas as tarefas próprias em estado terminal e os resultados foram lidos.

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

## Próxima revisão

Na próxima sessão que execute uma das atividades acima, registre ao lado de `Confirmação` o comando/ação que aplicou a prevenção. Se a falha se repetir, marque `recorrente` e proponha o controle executável mínimo; não acrescente outra lição idêntica.
