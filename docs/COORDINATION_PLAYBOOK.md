<!-- doc-type: procedure | status: current -->
# Playbook de coordenação Cricket, Fox e especialistas

## CP-00 Controle do documento, finalidade e uso

### Natureza

Este é um procedimento derivado para coordenar trabalho multiagente no repositório DFIR/NIST. Ele operacionaliza `AGENTS.md` §15-17, `CONVENTIONS.md`, `docs/DOCUMENT_AUTHORITY.md`, `docs/SESSION_IMPROVEMENT.md` e as skills locais. Não é requisito de produto, ADR, RTM, evidência de release ou autorização para ações externas.

Em conflito, prevalece a hierarquia de `docs/DOCUMENT_AUTHORITY.md`. Uma sugestão deste playbook só se torna regra do projeto quando a autoridade correspondente aprovar a mudança.

### Finalidade

1. Reservar ao Cricket as decisões de maior impacto e reduzir seu trabalho mecânico e suas toolcalls seriadas.
2. Permitir que o Fox distribua, acompanhe e integre trabalho sem aprovar o próprio resultado.
3. Dar a cada especialista um contrato compatível com sua capacidade observada e com as falhas já registradas.
4. Transformar erros em prevenções executáveis, não em narrativa ou culpa.
5. Tornar toda conclusão verificável por artefato, comando, diagnóstico e pós-condição.

### Público e uso por tarefa

- **Cricket:** ler CP-01 a CP-11 e CP-23 antes de coordenar; consultar os demais blocos conforme o risco.
- **Fox:** ler CP-01 a CP-16, CP-22, CP-23, CP-25 e CP-27 antes de distribuir ou integrar lanes.
- **Lobster:** usar CP-08, CP-09, CP-12, CP-17, CP-19, CP-20 e CP-25.
- **Ox:** usar CP-05, CP-06, CP-08, CP-09, CP-13, CP-18, CP-20, CP-21 e CP-25.
- **Rat:** usar CP-08, CP-09, CP-14, CP-20, CP-24, CP-26 e CP-25.

Uma tarefa pequena não exige ler o documento inteiro nem criar lanes. O checklist usado deve ser proporcional ao risco, mas os limites de autoridade e evidência não são opcionais.

### Qualidade esperada, sem promessa impossível

"Alta qualidade" significa que a entrega é compreensível, mínima para o objetivo, reproduzível, validada no nível afirmado e livre de dados indevidos. Não significa ausência garantida de defeitos. O playbook procura reduzir falhas sistemáticas e impedir que incerteza seja apresentada como fato.

Uma entrega incompleta pode ser útil quando seu estado é `PENDING`, `BLOCKED` ou `PENDING-HUMAN` e a pendência é explícita. Uma entrega aparentemente completa, mas sem evidência, não passa no aceite.

### Invariantes que nenhuma otimização pode remover

- autoridade e precedência documental;
- proteção de evidências, secrets e dados de caso;
- interface nativa e engine separado com JSONL;
- mudança mínima e preservação de trabalho alheio;
- reprodução Before/After para correção de comportamento;
- pós-condição independente, diagnósticos e logs reais;
- builds Rust pesados com dono único e espera pelo mesmo ID;
- controles negativos para gates novos ou alterados;
- revisão final e commits pelo root;
- nenhuma ação externa implícita.

---

## CP-01 — Autoridade, precedência, escopo e não objetivos

### Autoridade (ordem de precedência)
1. **Requisitos explícitos da demanda** — `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md`, rastreabilidade em `docs/DEMAND_TRACE.md`
2. **Corpo normativo da especificação** — `docs/PROJECT_SPEC_v6_5_1.md`
3. **Decisões/ADRs/RTM/status e contratos** — `docs/adrs/`, `docs/rtm.md`, `docs/FASE1_BACKLOG.md`
4. **Índices/XML/prompt derivados** — `docs/API.md`, `docs/NATIVE_UI_ARCHITECTURE.md`, `.jcode/swarm-prompt.md`
5. **Hipóteses, exemplos, sugestões** — nunca viram requisito sem processo de autoridade

### Princípios derivados (AGENTS.md §1)
- Não transforme hipótese, exemplo, inferência ou sugestão em requisito.
- Pedidos do operador que mudam tecnologia/escopo (ADR-0017 a ADR-0030) são **decisões de arquitetura/escopo** (nível 3), registradas por ADR com atualização da especificação. Não invocam precedência nº 1.
- `docs/DOCUMENT_AUTHORITY.md` é a autoridade por domínio.

### Escopo do playbook
- Papéis, roteamento de ferramentas, orçamento, lanes, integração, handoff, checklists, templates.
- Referencia autoridades em vez de copiá-las (preservation-map).
- Não incorpora o template swarm inteiro.

### Não objetivos
- Não criar nova autoridade, requisito, ADR, RTM, estado de release.
- Não autorizar ações externas por si só.
- Não copiar listas de tooling/segurança de `CONVENTIONS.md` — referenciar gatilhos.
- Não criar retrospectiva duplicada de `SESSION_IMPROVEMENT.md` — inserir pontos de confirmação no ciclo.
- Não elevar playbook a requisito/ADR/RTM.

---

## CP-02 Papéis, direitos de decisão e RACI

### Cricket

Planejador, coordenador de autoridade, integrador e avaliador final. Prioriza autoridade, risco, conflitos, segurança, arquitetura e aceite.

### Fox

Editor e subcoordenador de lanes. Detalha contratos, distribui trabalho, acompanha dependências, integra drafts e eleva exceções. Não aprova o próprio trabalho.

### Lobster

Especialista visual e documental. Produz grounding visual/documental, schemas visuais e capítulos relacionados. Trabalha com dados sintéticos e uma imagem por vez quando imagens forem autorizadas.

### Ox

Especialista de implementação e tooling. Produz edição estreita, testes determinísticos, ferramentas de QA e fixtures. Mantém baseline, hash e logs por iteração.

### Rat

Especialista de normalização e artefatos. Produz tabelas, formatos, schemas, checklists e resumos vinculados a fontes. Não apresenta normalização como execução independente.

### RACI por atividade

`R` executa, `A` responde pela decisão final, `C` é consultado e `I` é informado. Somente uma função `A` por atividade.

| Atividade | Cricket | Fox | Lobster | Ox | Rat |
|---|---|---|---|---|---|
| Interpretar missão e precedência | A/R | C | I | I | I |
| Definir escopo e risco | A/R | C | I | C | I |
| Decompor em lanes | A | R | C | C | C |
| Redigir núcleo de coordenação | A | R | I | C | C |
| Produzir capítulo visual/documental | A | C | R | I | C |
| Produzir tooling e testes de QA | A | C | I | R | C |
| Produzir schemas e templates | A | C | C | C | R |
| Decidir introduzir dependência/serviço/API | A/R | C | I | C | I |
| Autorizar ação externa | A/R | I | I | I | I |
| Executar gate de lane contratado | C | I | C | R | I |
| Integrar drafts | A | R | C | C | C |
| Revisão semântica final | A/R | C | I | I | I |
| Gates finais e evidência de tree | A/R | I | I | C | C |
| Commit e fechamento | A/R | I | I | I | I |

### Limites de delegação

O Fox pode reordenar etapas, pedir correções e consolidar drafts dentro da missão. Ele não pode ampliar escopo, criar autorização, alterar requisitos, aceitar comportamento não reproduzido, declarar gate final ou fazer commit/push.

O especialista pode escolher detalhes locais de implementação dentro do arquivo/fronteira recebido. Ele deve parar e escalar quando a entrega exigir outro arquivo, outra API, nova dependência, comportamento diferente, dado sensível ou ação externa.

### Escalada obrigatória ao Cricket

- conflito entre autoridade e missão;
- `unsafe`, concorrência, lifetimes complexos, macros ou API pública;
- mudança comportamental ou forense fora da hipótese aprovada;
- necessidade de dependência, serviço, transporte ou configuração global;
- risco a dados, secrets ou evidências;
- divergência entre relatório e artefato que o Fox não consegue resolver;
- duas tentativas sem informação nova;
- ausência de evidência necessária para o estado pretendido;
- rejeição de conteúdo visual pelo provedor;
- necessidade de exceder orçamento ou horário limite.

### Uma única autoridade de aceite

O Fox pode marcar um draft como editorialmente integrado. O especialista pode marcar sua implementação como pronta para revisão. Somente o Cricket pode marcá-la como integrada/verificada no repositório, e apenas depois de conferir a evidência correspondente. `APROVADO`, `LIMPO` ou `PASS` escrito por um worker é texto de relatório, não mudança de estado.

---

## CP-03 — Estados de trabalho e níveis de evidência; relatório não é gate

### Estados de trabalho (AGENTS.md §10)
```
PLANNED → IMPLEMENTED → INTEGRATED → VERIFIED → ACCEPTED
```
- **PLANNED**: no todo, com dono, escopo, pós-condições, orçamento.
- **IMPLEMENTED**: artefato escrito, testes atualizados, documentação afetada atualizada.
- **INTEGRATED**: revisão semântica do diff, `git diff --check`, gates aplicáveis executados, evidências STALE invalidadas.
- **VERIFIED**: evidência correspondente ao nível afirmado (não "parece correto").
- **ACCEPTED**: root confirma com evidência vinculada à tree/commit.

### Níveis de evidência (gates)
```
PASS / WAIVED / PENDING / BLOCKED / STALE
```
- **PASS**: gate executado, pós-condição confirmada, log absoluto, hash/tree vinculado.
- **WAIVED**: exceção de governança (ex.: WVR-001 Windows, WVR-002 aarch64), **não** prova de verificação.
- **PENDING**: dependência/autorização ausente, gate não executado por ferramenta ausente.
- **BLOCKED**: risco a dados, conflito de autoridade, autorização crítica ausente.
- **STALE**: evidência válida para tree anterior, tree mudou — requer reexecução.

### Relatório ≠ Gate
- Relatório é **índice** para a evidência, nunca a evidência em si.
- Antes de integrar: artefato final lido, diff semântico revisado, comandos/exits/diagnósticos confirmados, pós-condição independente verificada.
- `exit 0` não substitui pós-condição. Busca vazia inesperada exige controle positivo ou método alternativo.
- Duas tentativas sem informação nova → muda método ou consolida pendência.
- `PENDING-HUMAN` para aceites estéticos não demonstrados (rejeição de provedor visual), nunca inferência negativa.

---

## CP-04 — Algoritmo de seleção e ranking de tarefas, critérios de não iniciar

### Seleção de tarefas (ordem de prioridade)
1. **Bloqueadores críticos** — risco a dados, segredo vazado, autorização crítica ausente, conflito de autoridade.
2. **Dependências de integração** — tarefas que desbloqueiam múltiplas lanes (ex.: schema comum do Rat antes de Fox integrar).
3. **Gates obrigatórios** — verificação que impede progresso sem evidência (linter, doc-gate, supply-chain).
4. **Entregas de valor direto** — artefatos solicitados pela missão (CP-XX, drafts, schemas, linter).
5. **Melhorias e limpeza** — deduplicação, formatação, refatoração de texto.

### Ranking dentro da mesma prioridade
- Menor orçamento estimado primeiro (reduz latência total).
- Menor superfície de risco (arquivos isolados vs. transversais).
- Disponibilidade de dono (worker livre ou self).

### Critérios de NÃO iniciar (não entram no todo)
- [ ] Sem dono explícito (self ou worker com lane_owner).
- [ ] Sem pós-condições observáveis (não apenas comandos).
- [ ] Sem orçamento definido (minutos por método/lane).
- [ ] Escrita em arquivos fora do escopo da lane/missão.
- [ ] Dependência de ação externa não autorizada.
- [ ] Duplicação de lane existente (mesmo artefato/fronteira).
- [ ] Tarefa apenas de leitura/revisão sem artefato de entrega.
- [ ] Placeholder `TODO`/`TBD`/`<PREENCHER: CAMPO>` no contrato — deve vir preenchido.

### Consolidação precoce
- Se o orçamento restante não comportar uma nova hipótese com verificação, consolidar em vez de abrir nova lane.
- Quando várias lanes dependerem do mesmo insumo, concluir ou substituir essa dependência antes de iniciar trabalho que a reutilize.
- Cricket não abre workers enquanto apenas espera — consolida resultado parcial útil e informa pendências.

---

## CP-05 — Orçamento de contexto, latência e toolcalls do Cricket; pontos de delegação

### Gatilho de entrada

O Cricket precisa delegar trabalho ou coordenar mais de uma lane com artefatos independentes. O gatilho não autoriza criar worker: primeiro devem existir missão, dono, escopo exclusivo, dependências, pós-condições observáveis, orçamento e autorização externa.

### Insumos

- Missão e prioridade corrente, com autoridade e escopo explícitos.
- Contrato de lane e matriz de dependências.
- Registro de toolcalls e tempo por fase da rodada anterior, quando disponível.
- Limites de `AGENTS.md` §§15–17 e da lição de “sequência mecânica sem hipótese”.

### Orçamento numérico do Cricket

Os limites abaixo são **heurística inicial de planejamento por fase**, não meta de preencher, limiar de qualidade ou regra universal. Ajuste-os à complexidade e às métricas observadas; nunca reduza a verificação final para caber no teto. O Cricket registra o consumo real para retrospectiva, mas não cria lane de métrica para uma única tarefa trivial.

| Fase do Cricket | Tempo-alvo | Teto | Toolcalls root-alvo | Teto | Postcondição para sair da fase |
|---|---:|---:|---:|---:|---|
| Setup | 5 min | 8 min | 6 | 8 | Checkout/HEAD/status confirmados; missão e escopo decididos; lanes independentes identificadas |
| Delegação | 7 min | 12 min | 8 | 12 | Cada lane tem dono, artefato, fronteira, dependência, orçamento e mecanismo único de início |
| Execução | 20 min | 30 min | 12 | 18 | root fez apenas coordenação, risco, conflito e decisão; trabalho mecânico foi delegado |
| Integração | 8 min | 15 min | 10 | 14 | Artefatos finais lidos; diffs, gates e pendências revisados; root decide aceite ou devolve |
| **Total** | **40 min** | **65 min** | **36** | **52** | Relatório contém comandos/exits/paths/counts/warnings/postcondições e pendências reais |

Regras de contenção:

1. O orçamento conta **toolcalls do root**, não todas as chamadas dos workers. Toolcalls de gate e verificação são contados na fase em que forem executados.
2. Callbacks síncronos e respostas de workers não são contados como toolcalls do root. Respostas humanas e contenções de incidente também não.
3. Se uma fase ultrapassar 80% do alvo, o Cricket reduz escopo, consolida ou escala. Não abre nova lane para “compensar” consumo.
4. Em setup, um `batch` com até seis leituras independentes custa uma chamada de root, não seis. Em integração, até dez verificações independentes podem formar um `batch`.
5. Cada tentativa mecânica precisa de hipótese, método de confirmação e pós-condição. Duas tentativas sem informação nova mudam o método ou encerram a investigação.
6. O Cricket não copia path resumido de relatório. Localiza o artefato, usa caminho absoluto e lê o log real.

### Mapa de custo e dependência das chamadas

| Chamada | Consumo relativo | Quando usar | Dependências | Não usar para |
|---|---|---|---|---|
| `read` | 1 requisição, contexto exato | Ler um path conhecido e trecho necessário | Path absoluto/relativo conhecido | Buscar símbolo desconhecido ou ler tudo |
| `agentgrep` | 1 requisição, saída filtrada | `grep`/`find`/`outline`/`trace` no código | Termo ou relação conhecido | Simular `grep -r`, `find` amplo ou ler arquivo inteiro |
| `batch` | N chamadas internas, 1 round-trip root | Chamadas independentes, inclusive MCP | Schema e payload de cada tool validados | Encadear resultado A no tool B |
| `codemode` | 1 requisição, estado JS na sessão | Encadeamento, filtro, estado intermediário, fan-out com agregação | Schema e lógica de agregação | Simples leitura, batch sem estado ou comando real |
| `bash` | 1 requisição, execução real | Git, Python, testes, hashes, diffs e comandos do projeto | Comando e diretório explícitos | Exploração do workspace, leitura editorial ou catálogo MCP |
| MCP de leitura | 1 requisição por tool, estado externo atual | Confirmar estado externo/local atualizado | Action/schema atual confirmado | Substituir fonte local sem necessidade |
| `swarm` | 1 requisição + ciclo de vida | Trabalho paralelo com artefatos concretos | Lanes independentes e dono único | Tarefa trivial, apenas revisão ou arquivos sobrepostos |
| `bg` | 1 requisição + espera | Comando longo já iniciado, com log e ID | ID completo e dono do trabalho | Polling sem hipótese ou IDs de `await_members` |
| `schedule` | 1 requisição | Trabalho futuro com wake time e critério | Data futura e entregável | Simular concorrência agora ou substituir lane |

Regras de dependência:

1. `batch` não exige a saída de uma chamada para configurar outra. `codemode` só é escolhido quando há encadeamento, filtro, estado ou fan-out com agregação reais.
2. `bash` executa comandos reais e não substitui as ferramentas nativas de busca/leitura nem os catálogos MCP.
3. Toda ação de MCP é validada contra o schema atual antes da chamada. Uma rejeição de validação não prova que o serviço está indisponível.
4. Uma busca vazia inesperada exige controle positivo ou método alternativo de leitura/diff. O resultado não é tratado como ausência.
5. `bg wait` usa o mesmo ID completo do trabalho em segundo plano que iniciou o comando. Um identificador de `await_members` nunca pertence a `bg wait`.
6. `swarm` não é usado para trabalho trivial, investigação sem artefato ou lanes que escrevem os mesmos arquivos.

### Matriz de delegação obrigatória

| Trabalho | Cricket | Fox | Lobster | Ox | Rat | Motivo/evidência de aceite |
|---|---|---|---|---|---|---|
| Autoridade, precedência, conflito, risco, aceite final, commit | **decide** | consulta | informa | informa | informa | O root preserva a decisão e a responsabilidade final |
| Missão, seleção e distribuição de lanes | decide | **prepara/distribui** | consulta | consulta | consulta | Fox coordena sem se autoaprovar |
| Formatação normalizada e schema de artefato | decide formato mínimo | revisa | informa | informa | **executa** | O especialista entrega o schema; o root decide e integra |
| OCR ou grounding visual controlado | decide escopo | integra | **executa** | revisa artefato final | normaliza resumo | Imagem não entra no contexto automático; uma imagem por vez |
| Parsing serial de artefatos/logs | decide campos | revisa | visual | linter/determinístico | **normaliza** | Não apresentar normalização como gate independente |
| Cliques mecânicos e tentativas seriais | formula hipótese e limite | coordena | executa como especialista | não | não | Duas tentativas sem informação nova mudam o método |
| Linter estrutural e controles negativos | decide regra/aceite | integra | não | **executa** | valida schema | Gate prova estrutura, não qualidade semântica |
| Gates Rust | concede vez e registra | acompanha | não | **prepara ordem/artefatos** | não | Único dono, execução sequencial |
| Relatório final, logs, paths, contagens e warnings | **decide texto e evidência** | consolida drafts | entrega como especialista | entrega como especialista | normaliza | O relatório é índice; a evidência real fica no log |

**Limite inviolável:** Cricket pode ler o mínimo para decidir e delegar. Ele não executa formatação especializada, OCR, parsing serial ou cliques mecânicos quando um especialista pode entregar o artefato com evidência. Ele mantém a autoridade, a resolução de conflito, a classificação de risco, o julgamento de staleness e o aceite.

### Saída

- Plano por fase com alvo/teto de tempo e toolcalls.
- Tabela de delegação com dono, fronteira e critério de aceite.
- Registro do consumo real para a próxima rodada.

### Pós-condição

- Setup, delegação, execução e integração têm números, não apenas recomendações qualitativas.
- Cada toolcall relevante tem custo, dependência e antiuso explicitado.
- Nenhum toolcall é prescrito por ideologia; a escolha decorre de custo, dependência e pós-condição.
- O Cricket conserva autoridade, conflito, risco, staleness e aceite.

### Critério de parada e escalada

Parar a fase ao atingir o teto, perder dependência, violar escopo, não conseguir localizar evidência ou exceder duas tentativas sem informação nova. Escalar conflito de autoridade, risco a dados, autorização externa ausente ou divergência entre relatório e artefato ao Cricket. Fox eleva decisões de escopo; o root decide.

---

## CP-06 — Árvore de decisão de ferramentas: read/agentgrep/batch/codemode/bash/MCP/swarm/bg/schedule

### Gatilho de entrada

É necessário obter informação, executar uma ação ou coordenar trabalho. Antes da chamada, o agente deve nomear o **resultado observável**, o **custo relativo** e a **dependência** que impede outra ferramenta.

### Árvore de decisão

```text
Preciso de um resultado observável
├─ Arquivo/trecho já conhecido?
│  └─ read
├─ Procurar símbolo, arquivo, outline ou relação no código?
│  └─ agentgrep (grep | find | outline | trace)
├─ Várias chamadas independentes, inclusive MCP?
│  └─ batch
├─ Resultado A alimenta B, há filtro/estado/fan-out com agregação?
│  └─ codemode
├─ Preciso executar git, Python, teste, hash ou diff real?
│  └─ bash
├─ Preciso confirmar estado externo/local que só o MCP conhece?
│  └─ MCP de leitura, após validar action/schema
├─ Trabalho paralelo, artefato concreto e fronteiras independentes?
│  └─ swarm
├─ Comando já iniciado, de longa duração, com ID completo do trabalho?
│  └─ bg wait no mesmo ID
└─ Trabalho genuinamente futuro, com wake time?
   └─ schedule
```

### Regras por ferramenta

#### `read`

- Entrada: path conhecido e trecho suficiente.
- Consumo: uma chamada, conteúdo delimitado.
- Saída: bytes ou texto com a limitação da ferramenta explicitamente considerada. Conteúdo anotado ou truncado não é fonte bruta completa.
- Antiuso: não usar como busca global nem ler arquivo grande sem recorte.

#### `agentgrep`

- Entrada: termo/relação e path/glob; `paths_only=true` quando só os paths bastam.
- Consumo: uma chamada filtrada.
- Saída: regiões estruturadas e paths.
- Antiuso: não substituir por `grep -r`, `rg` amplo ou `find` recursivo via `bash`.

#### `batch`

- Entrada: chamadas independentes, com `action`, `server`, `tool` e `arguments` completos.
- Consumo: um round-trip do root para N operações independentes.
- Saída: resultados separados por chamada.
- Antiuso: não colocar dependência entre A e B no mesmo batch. Não omitir `arguments` ao usar o dispatcher MCP.

#### `codemode`

- Entrada: lógica de encadeamento, filtro, estado intermediário ou fan-out com agregação.
- Consumo: uma chamada, mas maior superfície de lógica e dependência de schema.
- Saída: apenas valores agregados/filtrados que são necessários.
- Antiuso: não usar para uma leitura, um grep, um batch sem dependência ou um comando real. Não presumir filesystem/rede/timers no sandbox.
- Entrada anotada: `read` pode incluir numeração de linhas e aviso de truncamento. Remova apenas a apresentação, valide um heading conhecido como controle positivo e falhe fechado antes de montar documentos ou relatórios.

#### `bash`

- Entrada: comando real, diretório explícito e resultado esperado.
- Consumo: uma chamada, execução real e saída potencial.
- Saída: exit, stdout/stderr e artefato quando aplicável.
- Antiuso: não explorar o workspace com shell quando existe `agentgrep`; não ler com `cat`/`head`/`tail` quando existe `read`; não editar com `sed`/`perl`/script Python.

#### MCP de leitura

- Entrada: objetivo sem dados sensíveis, `action`/`schema` atual e `arguments` explícitos.
- Consumo: uma chamada por tool, latência externa/local específica.
- Saída: estado atual com pós-condição verificável.
- Antiuso: não indexar ou reindexar como efeito colateral; não enviar dados de caso, paths sensíveis ou conteúdo de eventos; não tratar a saída de MCP como gate local.

#### `swarm`

- Entrada: lanes independentes, entrega concreta, dono e escopo exclusivo.
- Consumo: requisições de coordenação e ciclo de vida até a integração.
- Saída: artefato worker revisável, não veredito de aceite.
- Antiuso: não usar para tarefa trivial, leitura/revisão sem artefato ou fronteiras sobrepostas; não combinar auto-spawn e `swarm spawn`.

#### `bg`

- Entrada: ID completo do trabalho já iniciado e comando que pode continuar em segundo plano.
- Saída: exit, log absoluto e saída inspecionada.
- Antiuso: não iniciar um segundo comando Rust antes de `bg wait` no mesmo ID; não usar ID de `await_members`; não cancelar o trabalho silencioso sem inspecionar progresso/consumo.

#### `schedule`

- Entrada: wake time futuro, entregável, prioridade e critério de sucesso.
- Consumo: uma chamada de criação e retomada futura.
- Saída: execução/relatório futuro vinculado à mesma sessão ou a uma sessão filha autorizada.
- Antiuso: não usar para mascarar trabalho que precisa ser concluído agora; não presumir autorização externa.

### Exemplos negativos

1. **Não** executar `rg --hidden` por Bash para descobrir um símbolo quando `agentgrep` já oferece grep/outline/trace e paths filtrados.
2. **Não** usar `batch` para uma chamada cuja segunda operação depende da primeira; nesse caso, `codemode` ou chamadas sequenciais explícitas são adequadas.
3. **Não** usar `codemode` para encapsular um único `read` e alegar eficiência sem agregação.
4. **Não** usar `bash` para emitir `cat`, `head`, `tail`, `sed` ou `perl` como substituto de leitura/edição nativa.
5. **Não** usar `swarm` quando existe uma única edição local determinística.
6. **Não** usar `bg` apenas para adicionar outra camada a um comando curto que já retornou.
7. **Não** usar `schedule` para uma tarefa que precisa bloquear a integração atual.
8. **Não** enviar dados de caso, nomes de evidências, conteúdo de eventos, secrets ou paths sensíveis para MCP, workers ou logs.
9. **Não** chamar uma ação de MCP sem validar o envelope atual. Envelope rejeitado não prova indisponibilidade.
10. **Não** concluir ausência a partir de busca vazia sem controle positivo ou método alternativo.

### Saída

- Escolha da ferramenta com justificativa operacional.
- Resultado, pós-condição e qualquer escalação necessária.

### Pós-condição

- A decisão foi guiada por custo e dependência, não por preferência de ferramenta.
- Cada ramo tem custo, uso, pós-condição e antiuso.
- Busca vazia e schema inválido têm recuperação explícita.
- Nenhum MCP ou bash foi usado como substituto ideológico de uma ferramenta nativa.

### Critério de parada e escalada

Parar após obter a pós-condição ou após duas tentativas sem informação nova. Escalar schema MCP recorrente, tool indisponível, dependência ausente, divergência de fonte, risco a dados ou necessidade de mutação externa.

---

## CP-07 — Seleção de skills, leitura mínima e overlays locais

### Regra geral
Carregue **somente** skills cuja descrição corresponda à tarefa. As instruções do repositório e a hierarquia de autoridade prevalecem sobre qualquer skill.

### Skills do projeto (overlay local em `.jcode/skills/`, `.agents/skills/`, `.claude/skills/`)
| Skill | Quando carregar | Ação obrigatória |
|---|---|---|
| `.jcode/skills/dfir-improve/SKILL.md` | Toda sessão que altera arquivos **ou** executa gates | Loop abertura/feedback/integração/fechamento (§16 AGENTS.md) |
| `.jcode/skills/dfir-change/SKILL.md` | Ao alterar código, contratos ou documentação | Menor mudança suficiente, rigor de teste, autoridade |
| `.jcode/skills/dfir-verify/SKILL.md` | Antes de declarar integração ou verificação | Gates aplicáveis, controles negativos, evidência de tree |
| `.jcode/skills/dfir-detect/SKILL.md` | Incluir/alterar/remover regras `EVTX_DETECTION` | Com `dfir-change` e `dfir-verify`, campos normalizados |

### Overlay local — leitura e validação
- Skills do projeto formam overlay local **relido do disco** em `list`, `read`, `load`.
- Após editar skill local: chame `read` ou `load` para confirmar. **Não use `reload`** (renova só skills globais).
- `reload` de skill local → "not found" indica escopo, não ausência.
- Ação desconhecida em `skill_manage` pode retornar texto sem exceção — valide conteúdo, não só chamada.

### Skills globais
- Opcionais, exigem utilidade clara no caso concreto.
- Não instalar skills/pacotes/catálogos implicitamente — instalação = mudança de tooling/supply chain.

### Skill carregada ≠ evidência
- Gate real e pós-condição continuam obrigatórios.
- IDs futuros de evidência ficam em scratch até execução, arquivo e registro existirem.

### Leitura mínima por papel
| Papel | Skills obrigatórias | Leitura adicional |
|---|---|---|
| Cricket | `.jcode/skills/dfir-improve/SKILL.md`, `.jcode/skills/dfir-verify/SKILL.md` | `AGENTS.md`, `CONVENTIONS.md`, `docs/DOCUMENT_AUTHORITY.md`, `docs/SESSION_IMPROVEMENT.md`, `.jcode/swarm-prompt.md` |
| Fox | `.jcode/skills/dfir-improve/SKILL.md`, `.jcode/skills/dfir-change/SKILL.md`, `.jcode/skills/dfir-verify/SKILL.md` | Documentos da missão, matriz de falhas e mapa de preservação aplicáveis |
| Lobster | `.jcode/skills/dfir-verify/SKILL.md` | Fontes textuais e schemas visuais da tarefa |
| Ox | `.jcode/skills/dfir-change/SKILL.md`, `.jcode/skills/dfir-verify/SKILL.md` | Fontes de implementação, contrato do linter e logs da tarefa |
| Rat | `.jcode/skills/dfir-verify/SKILL.md` | Artefatos, schemas e fontes de normalização da tarefa |

---

## CP-08 — Contrato de missão: autoridade, escopo, pós-condições, risco, orçamento e autorizações

### Estrutura obrigatória (preenche `.jcode/swarm-prompt.md`)

```markdown
## Missão
<PREENCHER: MISSAO_E_RESULTADO_ESPERADO>  # uma frase, resultado observável

## Requisitos, ADRs, contratos e tickets aplicáveis
<PREENCHER: REFERENCIAS_DE_AUTORIDADE>    # lista curta com IDs normativos (RF-*, D*, ADR-*, EVD-*)

## Escopo permitido
<PREENCHER: ESCOPO>                       # arquivos, fronteiras, ações permitidas

## Fora de escopo
<PREENCHER: FORA_DE_ESCOPO>               # explícito: o que NÃO fazer

## Arquivos reservados à integração pelo coordenador
<PREENCHER: ARQUIVOS_DE_INTEGRACAO>       # ex.: docs/COORDINATION_PLAYBOOK.md, AGENTS.md

## Orçamento total e horário limite
<PREENCHER: ORCAMENTO_TOTAL>              # ex.: "90 min Fox, 45 min cada especialista, 20:00 UTC"

## Pós-condições globais
<PREENCHER: POS_CONDICOES_GLOBAIS>        # observáveis, não comandos

## Gates e verificações aplicáveis
<PREENCHER: GATES_E_VERIFICACOES>         # ex.: just verify, just doc-gate, linter, schema validation

## Modelos efetivos por role e escopo da fotografia
<PREENCHER: COORDENADOR_SWARM_REVIEW_JUDGE_MEMORY_AMBIENT>  # registro operacional, não requisito

## Política de commits desta tarefa
<PREENCHER: QUEM_COMMITA_E_QUANDO>        # padrão: Cricket commita após revisão; worker não faz push

## Autorizações externas explícitas
<PREENCHER: AUTORIZACOES_EXTERNAS_OU_NENHUMA>  # ação, alvo, escopo; NENHUMA se não houver
```

### Validação da missão
- Todos os campos preenchidos no contrato real, sem `{{...}}`; os rótulos `<PREENCHER: ...>` existem somente nos templates e precisam ser substituídos ao instanciá-los.
- `ESCOPO` e `FORA_DE_ESCOPO` não se sobrepõem.
- `ARQUIVOS_DE_INTEGRACAO` não inclui arquivos de workers.
- `AUTORIZACOES_EXTERNAS` vazio ou lista com ação/alvo/escopo.
- `POS_CONDICOES_GLOBAIS` contém critérios observáveis (arquivo existe, hash X, contagem Y, exit 0, diagnóstico Z).

### Exemplo fictício preenchido
```markdown
## Missão
Produzir docs/COORDINATION_PLAYBOOK.md com CP-00..CP-28, linter e templates.

## Requisitos, ADRs, contratos e tickets aplicáveis
AGENTS.md §15-17, CONVENTIONS.md, docs/DOCUMENT_AUTHORITY.md e docs/SESSION_IMPROVEMENT.md

## Escopo permitido
Drafts sanitizados da rodada e scripts/coordination_playbook_gate.py; somente o root escreve o canônico

## Fora de escopo
Mudanças forenses, novas dependências, imagens não autorizadas e ações externas

## Arquivos reservados à integração pelo coordenador
docs/COORDINATION_PLAYBOOK.md, AGENTS.md, docs/DOCUMENT_AUTHORITY.md e justfile

## Orçamento total e horário limite
90 min de coordenação, 45 min por especialista e deadline <DEADLINE_ISO8601>

## Pós-condições globais
- CP-00..CP-28 presentes uma vez e com conteúdo
- Cinco papéis com fluxo, checklist e limites
- Decisões pendentes registradas com dono e razão
- Relatório contém arquivos, comandos/exits, diagnósticos e pendências reais

## Gates e verificações aplicáveis
Linter Python stdlib, positivo e mutações negativas, just doc-gate, git diff --check e revisão semântica root

## Modelos efetivos
Registrar modelos por role e escopo da fotografia, sem tratá-los como requisito

## Política de commits
Cricket commita após revisão; workers não fazem push

## Autorizações externas
NENHUMA
```

---

## CP-09 — Contrato de lane: artefato, propriedade, dependências, início único e aceite

### Estrutura obrigatória (tabela no `.jcode/swarm-prompt.md`)

| Lane e dono | Artefato e entrega | Escopo exclusivo de escrita | Dependências | Orçamento | Aceite e pós-condições | Início |
|---|---|---|---|---|---|---|
| <PREENCHER: LANE_1_E_DONO> | <PREENCHER: ARTEFATO_1> | <PREENCHER: ARQUIVOS_OU_FRONTEIRAS_1> | <PREENCHER: DEPENDENCIAS_1_OU_NENHUMA> | <PREENCHER: ORCAMENTO_1> | <PREENCHER: ACEITE_1> | <PREENCHER: AUTOMATICO_OU_SPAWN_1> |

### Regras de início único — escolha **um** mecanismo
1. **`lane_owner = "worker"`** (ou `owner = "worker"`) → worker auto-spawn quando lane pronta. **Não combine** com `swarm spawn` para a mesma lane.
2. **`swarm spawn` explícito** → coordenador lança. **Não configure** também `lane_owner = "worker"`.

### Propriedade de escrita
- Cada worker escreve **exclusivamente** nos arquivos/fronteiras da sua linha.
- Não edita arquivos do coordenador, de outra lane, ou fora da tarefa.
- Se entrega exigir ultrapassar limite → solicitar redivisão ao coordenador **antes** de escrever.

### Dependências
- Citam lanes anteriores por nome (ex.: `ox` depende de `rat` para schema).
- Aceite contém pós-condições **observáveis**, não apenas comandos.

### Orçamento
- Respeita menor entre: total, lane, método.
- 15 min método improdutivo → consolida ou muda método.
- 30 min lane exploratória sem resultado útil → consolida.

### Aceite (exemplos por papel)
| Papel | Artefato | Pós-condições observáveis |
|---|---|---|
| Lobster | `visual-documentation.md`, `lobster-delivery.json` | Referências arquivo:linha, sem placeholders, nenhuma imagem quando proibida e uma imagem por vez quando autorizada |
| Ox | `ox-tooling.md`, `ox-delivery.json`, `scripts/coordination_playbook_gate.py`, `tests/fixtures/coordination_playbook/**` | Baseline hash estável, diffs incrementais, linter passa positivo + falha negativas, comandos/exits/paths absolutos |
| Rat | `validation-artifacts.md`, `lane-delivery-schema.json`, `rat-delivery.json` | Schema válido, formatos copiáveis, links para logs, sem execução de gates |
| Fox | `core-coordination.md`, `playbook-integrated.md`, `fox-delivery.json`, `fox-decisions.md` | Todos CP-00..CP-28 presentes, 5 papéis cobertos, decisões listadas, delivery JSON válido |

---

## CP-10 — Workflow Cricket: decisões difíceis, contratos, integração e aceite

### Gatilho
Nova demanda, mudança de escopo/tecnologia, ou coordenação complexa que requer autoridade root.

### Insumos
- Demanda original ou ADR propondo mudança.
- `docs/DEMAND_TRACE.md` para rastreabilidade.
- Playbook existente (se houver) para regras de coordenação.

### Passos
1. **Intake & classificação** — identificar nível de precedência (demanda vs. decisão vs. hipótese).
2. **Definir missão** — preencher contrato CP-08 com escopo, orçamento, pós-condições, autorizações.
3. **Selecionar lanes** — usar algoritmo CP-04; não criar lanes triviais/dependentes sem dono.
4. **Preencher template swarm** — validar os campos CP-08 e as regras de `.jcode/swarm-prompt.md`.
5. **Spawn workers** — um mecanismo por lane (`lane_owner=worker` OU `swarm spawn`).
6. **Acompanhar** — copiar IDs completos, `swarm list`/`bg list`, não cancelar build silencioso sem inspecionar.
7. **Integrar** — aguardar workers necessários, revisar diff semântico, `git diff --check`, gates aplicáveis.
8. **Resolver conflitos** — elevados via `fox-decisions.md` ou DM direto; Cricket decide.
9. **Commits temáticos** — pequenos, após gates, só arquivos da tarefa, sem dados operacionais.
10. **Fechamento** — `swarm list`/`bg list` zerados, revisão semântica final, `SESSION_LEARNINGS.md` se lição nova.

### Saída
- Missão preenchida, lanes definidas, workers spawnados, `fox-decisions.md` iniciado.

### Pós-condições
- Template swarm com campos obrigatórios preenchidos e pós-condições verificáveis.
- Orçamento total alocado ≤ limite da sessão.
- Autorizações externas explicitadas (ou NENHUMA).

### Parada/Falha/Escalada
- **Parada**: orçamento esgotado, dependência/autorização ausente, conflito de autoridade, risco a dados, falta de progresso (critérios CP-04).
- **Falha**: worker não entrega artefato, relatório diverge do arquivo, gate não passa.
- **Escalada**: Fox → Cricket para decisões críticas; registra em `fox-decisions.md`.

---

## CP-11 — Workflow Fox: distribuir, acompanhar, integrar drafts e elevar conflitos

### Gatilho
Missão recebida do Cricket com contrato CP-08 preenchido.

### Insumos
- Contrato de missão (CP-08).
- Contratos de missão e de lane CP-08/CP-09.
- Matriz de falhas, mapa de preservação e outras fontes sanitizadas da tarefa.
- Contratos de lane (CP-09) para cada especialista.

### Passos
1. **Validar missão** — confirmar checkout/HEAD, escopo, orçamento, autorizações.
2. **Enviar contratos de lane** — DM para cada worker com CP-09 preenchido (escopo exclusivo, artefato, aceite, orçamento).
3. **Acompanhar execução** —
   - Copiar IDs completos dos workers.
   - `swarm status` / `bg list` para progresso.
   - Não iniciar trabalho Rust pesado sem vez do coordenador.
   - Se worker silencioso > 15 min: inspecionar progresso/consumo antes de cancelar.
4. **Receber entregas** — ler artefato final **antes** de integrar; confirmar diff semântico, comandos/exits, pós-condições.
5. **Integrar drafts** —
   - Combinar `core-coordination.md` + `visual-documentation.md` + `ox-tooling.md` + `validation-artifacts.md` → `playbook-integrated.md`.
   - Preservar todos CP-00..CP-28, evitar duplicação, manter ordem canônica.
   - Referenciar autoridades (não copiar).
   - Substituir placeholders por conteúdo real.
6. **Elevação de conflitos** —
   - Conflito entre workers (escopo, dependência, prioridade) → registrar em `fox-decisions.md` com opções, aguardar Cricket.
   - Omissão no design brief → registrar, propor preenchimento, aguardar Cricket.
   - Relatório divergente do arquivo → não integrar, devolver ao worker com diagnóstico.
7. **Produzir entregas Fox** —
   - `playbook-integrated.md` (documento completo).
   - `fox-delivery.json` (schema do Rat).
   - `fox-decisions.md` (conflitos/omissões pendentes root).
8. **Executar prechecks** — linter, schema quando aplicável e controle de diff nos drafts. O `just doc-gate` final, a revisão semântica e os demais gates de integração pertencem ao Cricket.

### Saída
- `playbook-integrated.md` pronto para revisão root.
- `fox-delivery.json` e `fox-decisions.md`.

### Pós-condições
- Todos CP-00..CP-28 presentes, sem seções vazias/placeholders.
- 5 papéis com fluxo, checklist, limites.
- `fox-decisions.md` lista conflitos/omissões para root.
- `fox-delivery.json` com arquivos, comandos/exits, diagnósticos e pendências reais; schema é validado somente quando essa verificação for aplicável e executada.

### Parada/Falha/Escalada
- **Parada**: orçamento Fox definido na missão esgotado, dependência worker não entregue ou autorização root ausente.
- **Falha**: linter/schema falha, duplicação excessiva, CP ausente.
- **Escalada**: para Cricket via `fox-decisions.md` e DM.

---

## CP-12. Workflow Lobster: grounding visual/documental e limites

### Gatilho de entrada

A lane é acionada por uma missão visual/documental com escopo, fontes e restrições explícitas. O contrato define os artefatos permitidos e se imagens podem ser abertas. Sem autorização para imagens, Lobster trabalha apenas com código, relatórios textuais e schemas sanitizados já disponíveis.

### Insumos e propriedade das fontes

As fontes devem ser versionadas ou identificadas por caminho absoluto no índice sanitizado da rodada. Relatórios antigos são insumos históricos, nunca substitutos do código e do diff atuais.

| Insumo | Papel | Uso nesta seção |
|---|---|---|
| `docs/SESSION_LEARNINGS.md` | Registro vivo de falhas e controles | Origina prevenções sem transformar narrativa em requisito. |
| `docs/DOCUMENT_AUTHORITY.md` | Autoridade por domínio | Define a propriedade documental e a deduplicação por referência. |
| `src/gui/theme.rs` | Fonte de implementação dos tokens, contraste e transição de tema | Sustenta as decisões documentais sobre tema, movimento reduzido e duração. |
| `src/gui/effects.rs` | Fonte de implementação dos efeitos de transição e seleção | Sustenta as decisões sobre crossfade, movimento reduzido, gradiente e ausência de animação periódica. |
| `src/gui/mod.rs` | Fonte de integração da UI e das preferências | Sustenta as decisões sobre navegação, janela de preferências, cancelamento e integração dos efeitos. |
| `super-vl-doc-sync.md` | Relatório textual da sincronização anterior | É índice de uma revisão anterior, não substitui a fonte nem o diff final. |
| `super-vl-visual-review.json` e `super-vl-golden-visual-review.json` | Relatórios textuais de inspeções controladas anteriores | Sustentam o formato da revisão e suas limitações, sem reclassificar as imagens nesta missão. |
| `root-integration-review.md` | Revisão textual do root | Registra o pipe indevido removido e a regra de aceitar arquivo e diff, não anúncio. |
| `validate_structured_report.py` | Validador local do schema estrito | Define o formato aceito, o allowlist de nomes e os enums de `overall` e `confidence`. |

### Passos operacionais

1. Confirmar `git rev-parse --show-toplevel` e `git rev-parse HEAD` antes de qualquer escrita; divergência interrompe a lane.
2. Ler somente as fontes textuais necessárias. Para cada afirmação visual/documental, registrar arquivo e linha ou o bloco textual exato da fonte.
3. Separar três níveis de afirmação:
   - **implementação**: comportamento definido na fonte atual;
   - **observação anterior**: relato textual de uma inspeção controlada anterior;
   - **limitação**: aquilo que as fontes e os quadros estáticos não demonstram.
4. Abrir imagens somente quando a missão autorizar. Sem essa autorização, reaproveitar apenas relatórios que registram nome lógico, `observed`, notas, confiança, `overall` e limitações.
5. Validar o texto final por diff e controles textuais de whitespace, pipes de tabela, placeholders, base64 e referências. O relatório só pode declarar o que foi executado.

### Saída

Uma seção de playbook com workflow acionável, checklist, esquema do relatório visual, limites de RGB, estados de preferência, escalada e critérios de parada. A decisão de layout e integração canônica permanece com o root.

### Pós-condições

- Toda regra técnica local aponta para uma fonte textual com arquivo e linha.
- O documento não afirma animação, estética ou contraste a partir de um quadro estático.
- Relatórios visuais antigos são citados como evidência histórica, com o HEAD e as limitações preservados.
- Nenhuma imagem é aberta, anexada, reenviada ou convertida em base64.
- A revisão final do root decide a integração e o aceite. Um relatório de Lobster não é gate.

### Critério de parada e escalada

O orçamento da lane é definido pela missão. Duas tentativas sem informação nova exigem mudar de método ou encerrar a investigação. Rejeição do provedor encerra a lane visual sem retry, troca de modelo ou bypass: o aceite estético fica `PENDING-HUMAN`, enquanto a validação textual e independente continua. Se a ferramenta visual estiver indisponível, o resultado visual é `uncertain`, nunca uma inferência negativa. A política de anexos e escalada está em `AGENTS.md:243`.

### Grounding técnico para o playbook

- **Tema e preferência visual.** `ThemeTokens::new` resolve a base, a personalização de matiz/croma e a duração de movimento (`src/gui/theme.rs:150-205`). O ajuste de acento deve ser descrito a partir dessa função, sem generalizar para cores semânticas, que permanecem estáveis (`src/gui/theme.rs:1-4`).
- **Movimento reduzido.** A duração zero é criada nos tokens quando `reduced_motion` está ativo (`src/gui/theme.rs:200-204`). A UI também resolve imediatamente a troca de tema quando o alvo ou os tokens ativos têm movimento zero (`src/gui/mod.rs:402-407`). O playbook deve preservar essa distinção, pois o relatório anterior registrou uma confusão entre os dois pontos (`super-vl-doc-sync.md:10-13`).
- **Duração e progressão de tema.** `MOTION_DURATION` é 160 ms (`src/gui/theme.rs:11`). `ThemeTransition::apply` interpola progressivamente e só agenda repaint enquanto a transição está ativa (`src/gui/theme.rs:354-368`). A descrição documental deve usar a fonte atual, não o relatório antigo que apontou HEAD distinto (`super-vl-doc-sync.md:3-5`).
- **Efeitos de ação.** A duração recomendada é 150 ms e o teto é 180 ms (`src/gui/effects.rs:25-35`). Movimento reduzido ou duração zero resulta em progresso final imediato (`src/gui/effects.rs:44-62` e `src/gui/effects.rs:86-94`).
- **Crossfade.** O crossfade usa o temporizador interno do egui, resolve imediatamente em movimento reduzido e não mantém scheduler em repouso (`src/gui/effects.rs:238-290`). Ao deselecionar, o estado é removido e o scheduler é zerado (`src/gui/effects.rs:293-306`).
- **Gradiente de seleção.** O gradiente é horizontal, limitado à área da seleção e recortado pelo mesmo raio do controle (`src/gui/effects.rs:314-408`). A borda mascarada usa gradiente alfa, sem multiplicação gama/linear (`src/gui/effects.rs:410-432`).
- **Navegação e preferências.** A navegação usa item selecionado com preenchimento e gradiente de borda (`src/gui/mod.rs:439-501`). A janela de preferências permite tema/base, matiz, croma e movimento reduzido; cancelamento restaura o tema salvo (`src/gui/mod.rs:537-635`).
- **Ausência de animação periódica.** O módulo de efeitos declara que cada efeito parte de evento explícito e termina, sem animação ambiente ou temporização periódica (`src/gui/effects.rs:1-11`). A UI registra que backend, seletores e animações pedem repaint na origem, sem timer periódico em idle (`src/gui/mod.rs:1011-1015`).

---

## CP-13 — Workflow Ox: edição estreita, baseline/hash e loop determinístico

### Gatilho de entrada

Uma missão exige editar um ou poucos arquivos com uma mudança local que pode ser revisada por diff. O fluxo não se aplica a uma reescrita integral que exige contrato e revisão próprios.

### Insumos

- Checkout absoluto e HEAD esperado.
- Escopo de escrita exclusivo e arquivos reservados ao coordenador.
- Arquivo alvo, mudança comportamental mínima e critério de convergência.
- Fontes de autoridade e testes aplicáveis.

### Passos

1. **Confirmar checkout, branch e HEAD.** Executar `git rev-parse --show-toplevel` e `git rev-parse HEAD` no checkout absoluto. Se o path ou HEAD divergir, não editar e pedir correção ao coordenador.
2. **Registrar estado e baseline.** Executar `git status --short` e `sha256sum <arquivo-alvo>`. Guardar comando, exit, path absoluto, hash, contagem de warnings e qualquer diff preexistente. Não presumir que um hash em relatório é o hash atual.
3. **Ler o trecho corrente.** Usar `read`/`agentgrep` antes de editar. Se o arquivo mudou desde o baseline, recalcular o baseline e explicar a mudança.
4. **Aplicar a menor edição.** Usar `edit`/`apply_patch`/`write` apenas no escopo. Não reescrever o arquivo inteiro para uma alteração local. Preservar correções concorrentes.
5. **Comparar hash e diff.** Após cada iteração, executar hash absoluto e diff semântico do arquivo. O diff incremental deve mostrar apenas a mudança prevista. Se o hash/diff não muda, registrar `N/A` e não repetir a revisão mecânica.
6. **Repetir até convergência.** Cada nova iteração parte do diff anterior, declara hipótese nova e compara hash/diff. Terminar quando a pós-condição passa e a revisão semântica confirma apenas a alteração pedida.
7. **Revisar o arquivo final.** Ler o resultado final, não confiar no snippet do relatório nem em uma visão parcial da ferramenta.
8. **Executar gates aplicáveis.** Registrar comando, exit, path absoluto do log, contagens, warnings e pós-condição independente. Relatório não é evidência.
9. **Fechar.** Executar `git diff --check`, revisar `git status --short`, confirmar que nenhum arquivo fora do escopo mudou e reportar hashes/diffs finais.

### Saída

- Arquivo editado com diff mínimo.
- Baseline e hashes por iteração.
- Diff incremental e log de gate localizáveis.
- Relatório com todos os metadados de execução.

### Pós-condição

- Baseline inicial e hash final são localizáveis e reproduzíveis.
- O diff final contém somente a mudança solicitada.
- Cada iteração compara hash e diff; ausência de diff é `N/A`.
- O relatório inclui comando, exit, path absoluto, contagens, warnings e pós-condição.
- O arquivo final foi relido e não existe divergência entre relatório e artefato.

### Critério de parada e escalada

Parar quando o diff convergir, o gate aplicável falhar, o arquivo mudar fora da hipótese, o checkout divergir, o escopo for violado ou duas tentativas não trouxerem informação nova. Escalar conflito de fonte, escrita concorrente, necessidade de reescrita integral ou pedido de aceite sem gate.

---

## CP-14 — Workflow Rat: normalização, schemas, formatação e vínculo com logs

### Gatilho

Normalizar uma entrega recebida de lane ou preparar um contrato que o root possa validar de forma determinística.

### Insumos

1. Artefato ou relatório original da lane.
2. Baseline da rodada, indexado pelo log de validação da rodada (fora do checkout).
3. Logs e diffs que sustentam as afirmações da entrega.
4. Autoridade, escopo e pós-condições da missão.

### Passos

1. **Preservar a fonte.** Copiar o texto factual para a estrutura canônica. Não corrigir números, nomes, comandos, exits ou conclusões sem fonte.
2. **Classificar cada afirmação.** Artefato final, observação de relatório, comando, log, diff, decisão ou pendência.
3. **Resolver caminho.** Artefato versionado usa path relativo ao repositório, por exemplo `docs/COORDINATION_PLAYBOOK.md`. Artefato ou log fora do checkout usa path absoluto real, registrado no índice de entrega da rodada. Path relativo nunca prova destino.
4. **Vincular evidência.** Cada artefato e cada claim relevante recebe path, `kind`, `claim`, `sha256` quando conhecido e `verification_status`.
5. **Separar execução de indexação.** O normalizador não executa gates. Gate não executado usa `executed: false` e estado `PENDING`, `BLOCKED` ou `STALE` conforme o fato.
6. **Revisar semanticamente.** Comparar a entrega normalizada com o artefato final e com a fonte. Divergência de hash, contagem, arquivo ou comando impede integração.

### Saída

- Schema comum de entrega, distribuído como artefato da rodada e referenciado pelo playbook.
- Uma instância normalizada da categoria entregue, com `schema_version: 1.0.0` e `delivery_kind` correspondente.

### Pós-condição

- JSON sintaticamente válido.
- Blocos específicos exigidos apenas quando `delivery_kind` correspondente é emitido.
- Toda claim com ligação rastreável e estado de verificação pelo root.
- Nenhum gate independente executado ou declarado `PASS` pela normalização.

### Critério de parada e falha/escalada

- Parar e consolidar se o artefato final não puder ser localizado, se a fonte contradizer o relatório ou se a recontagem pelo root exigir informação ausente.
- Escalar ao coordenador como `BLOCKED` em conflito de autoridade, dependência não entregue ou risco a dados. Divergência factual não corrigida permanece `BLOCKED`, nunca `PASS`.

### Formato de normalização

| Campo | Regra |
|---|---|
| `schema_version` | String `1.0.0` |
| `delivery_kind` | `mission`, `lane`, `report`, `decision`, `task_card` ou `handoff` |
| `generated_at` | Timestamp ISO 8601 do emissor |
| `producer` | Papel, session ID, raiz absoluta do checkout e HEAD de 40 hex |
| Bloco `report` | Before/After, artefatos, comandos/exits, diagnósticos, pós-condições, gates e pendências |
| Bloco `mission` | Autoridade, escopo, pós-condições, risco, orçamento, autorizações e gates aplicáveis |
| Bloco `lane` | Artefato, dono, escopo exclusivo, dependências, mecanismo único de início, orçamento e aceite observável |
| Bloco `decision` | Conflito, opções, decisão, justificativa, responsável, prazo, estado e autoridade |
| Bloco `task_card` | ID, descrição, dono, estado, prioridade, dependências, Before/After, validação e follow-ups |
| Bloco `handoff` | Origem, destino, insumos, saída, estado, aceite, próximo dono e próxima ação |

A validação estrutural do root deve, no mínimo:

1. rejeitar path relativo de artefato de rodada ou log;
2. rejeitar `PASS` em gate com `executed: false`;
3. rejeitar relatório sem `before`, `after`, `commands`, `diagnostics`, `postconditions`, `gates` e `pending`;
4. rejeitar link de evidência sem path, claim ou `verification_status`;
5. rejeitar hash fora do formato SHA-256 de 64 hex;
6. permitir ao root trocar `pending_root_read` por `root_read` ou `root_rejected` e registrar `root_note`.
7. validar o próprio schema com `Draft202012Validator.check_schema` e cada instância com o dialecto declarado, registrando implementação e versão.
8. ativar `FormatChecker` somente com os backends exigidos pelos formatos usados e provar um caso válido e um inválido por formato. Para `date-time`, por exemplo, o backend `rfc3339-validator` deve estar presente no ambiente isolado; backend ausente deixa a verificação do formato `PENDING`, nunca a promove a `PASS`.
9. preservar originais e registrar toda normalização/reparação root em cópia, com ledger de mudanças, hashes e tamanhos recalculados. Schema válido não prova veracidade da claim nem aceite de gate.

---

## CP-15 — Máquina de estados e handoffs entre intake, lane, integração, gate e fechamento

### Estados globais da coordenação

```
INTAKE → LANE_EXECUTION → INTEGRATION → GATE_VERIFICATION → CLOSURE
```

Cada estado tem critérios de entrada, ações obrigatórias e saída para o próximo.

### 1. INTAKE (Cricket)
**Gatilho**: Nova demanda, ADR propondo mudança, ou coordenação complexa.
**Entrada**: Contrato de missão (CP-08) preenchido, template swarm validado.
**Ações**:
- Classificar precedência (demanda vs decisão vs hipótese).
- Definir lanes via algoritmo CP-04.
- Preencher contratos de lane (CP-09) para cada especialista.
- Spawn workers (um mecanismo por lane).
- Iniciar `fox-decisions.md`.
**Saída**: Missão ativa, workers executando, Fox acompanhando.
**Handoff para LANE_EXECUTION**: IDs dos workers copiados, orçamentos alocados, dependências declaradas.

### 2. LANE_EXECUTION (Fox acompanha, Workers executam)
**Gatilho**: Workers spawnados e confirmados.
**Entrada**: Contratos de lane válidos, escopo exclusivo, orçamentos.
**Ações Fox**:
- `swarm status` / `bg list` a cada ciclo.
- Copiar IDs completos, confirmar progresso.
- Não iniciar trabalho Rust pesado sem vez.
- Worker silencioso > 15 min → inspecionar antes de cancelar.
**Ações Workers**:
- Entregar artefato + relatório JSON no escopo exclusivo.
- Atualizar todo da própria lane.
- Não editar fora da fronteira.
**Saída**: Artefatos prontos nos drafts, relatórios JSON recebidos.
**Handoff para INTEGRATION**: Fox confirma entregas lidas, diff semântico revisado, pós-condições observadas.

### 3. INTEGRATION (Fox)
**Gatilho**: Todas as lanes necessárias entregaram (ou orçamento esgotado com resultado parcial útil).
**Entrada**: `core-coordination.md`, `visual-documentation.md`, `ox-tooling.md`, `validation-artifacts.md`, relatórios JSON.
**Ações**:
- Ler cada artefato final **antes** de integrar.
- Combinar em `playbook-integrated.md` preservando CP-00..CP-28, ordem canônica, sem duplicação.
- Referenciar autoridades (não copiar).
- Substituir placeholders por conteúdo real.
- Registrar conflitos/omissões em `fox-decisions.md`.
- Produzir `fox-delivery.json` (schema Rat).
**Saída**: `playbook-integrated.md`, `fox-delivery.json`, `fox-decisions.md`.
**Handoff para GATE_VERIFICATION**: Artefatos prontos para linter, schema, doc-gate, diff-check.

### 4. GATE_VERIFICATION (Fox prepara, Cricket executa e valida)
**Gatilho**: Integração editorial concluída e arquivos reservados entregues ao root.
**Entrada**: Draft integrado, relatórios disponíveis, linter, schema quando aplicável e diff real do checkout.
**Ações Fox**:
- Entregar prechecks e diagnósticos, sem promover estados finais.
- Resolver divergências editoriais antes do handoff.
**Ações Cricket**:
- Repetir linter Python stdlib com positivo e mutações negativas.
- Validar schema quando essa verificação for aplicável e houver validador disponível; caso contrário registrar PENDING.
- Executar `just doc-gate` após ligar o linter ao `justfile`.
- Executar `git diff --check` e revisão semântica do artefato final.
**Saída**: Gates PASS/WAIVED/PENDING/BLOCKED/STALE registrados pelo root com evidência.
**Handoff para CLOSURE**: Gates aplicáveis PASS ou pendências explicitamente registradas; WAIVED somente por governança autorizada.

### 5. CLOSURE (Cricket)
**Gatilho**: Gates verificados.
**Entrada**: `playbook-integrated.md` validado, decisões pendentes resolvidas ou registradas.
**Ações**:
- `swarm list` / `bg list` → zerados (nenhum worker/processo próprio ativo).
- Revisão semântica final do diff.
- Commits temáticos pequenos, só arquivos da tarefa.
- `SESSION_LEARNINGS.md` se lição nova recorrente/confirmada.
- Relatório final ao Cricket: arquivos, commits, comandos/exits, gates, pendências, ações externas.
**Saída**: Commits locais, estado limpo, documentação atualizada.
**Fim**: Coordenação encerrada.

### Tabela de handoffs

| De → Para | Artefato de handoff | Critério de aceite do receptor |
|---|---|---|
| Cricket → Fox | Contrato missão (CP-08) + lanes (CP-09) | Missão válida, orçamento alocado, autorizações explícitas |
| Workers → Fox | Artefato + JSON delivery | Arquivo lido, diff semântico OK, pós-condições observadas |
| Fox → Cricket (integration) | `playbook-integrated.md` + `fox-decisions.md` | CP-00..CP-28 presentes, sem placeholders, 5 papéis cobertos |
| Fox → Cricket (gates) | Resultados linter/schema/doc-gate/diff | PASS ou WAIVED justificado; PENDING/BLOCKED documentados |
| Cricket → Fim | Commits + relatório | `git status` limpo, `swarm list`/`bg list` vazios, lições registradas |

---

## CP-16 — Classes de risco e escalas: unsafe, concorrência, lifetimes, macros, API pública, comportamento, dados e ações externas

### Matriz de classes de risco (ordem de severidade)

| Classe | Descrição | Exemplos no projeto | Escala de impacto | Ação requerida |
|---|---|---|---|---|
| **R1 — Unsafe/Rust unsafe** | Bloco `unsafe`, FFI, manipulação de ponteiros brutos | Novo bloco ou mudança de fronteira de memória | **Crítico** — corrupção de memória, UB | Parar e escalar ao root; aplicar somente os controles exigidos pela autoridade e pelo risco concreto, sem presumir ADR ou crate novo. |
| **R2 — Concorrência/data races** | `Arc<Mutex>`, `tokio::spawn`, shared state, channels | Backend thread, engine subprocess, `Crossfade` HashMap | **Alto** — race conditions, deadlocks, leaks | Dono único para builds pesados; `bg wait` no mesmo ID; `saturating_add` + `min` em progresso; `HashMap` só em thread única (UI). |
| **R3 — Lifetimes/borrow** | Referências cruzadas, `&mut` aliased, `Pin` | `ThemeTransition` + `Context`, `Crossfade` + `Context` | **Alto** — use-after-free, borrow checker | `animate_bool_with_time` gerencia lifetime; não armazenar `&Context` além do frame. |
| **R4 — Macros/proc-macro** | Expansão de código e dependências de compile-time | Macro nova ou mudança de comportamento expandido | **Médio** — expansão oculta, supply chain | Revisar a expansão quando relevante, consultar lock/gates aplicáveis e escalar nova macro crate ao root. |
| **R5 — API pública/contrato** | JSONL stdin/stdout engine, SQLite schema, CLI args | `dfir-protocol`, migrations, `AppConfig` | **Médio** — breaking changes, compatibilidade | Contratos versionados; migrations históricas não editadas; ADR para mudança. |
| **R6 — Comportamento/UX** | Animações, reduced motion, repaint, focus, acessibilidade | `ThemeTransition` 160ms, `Crossfade` step_dt, `selection_gradient` | **Médio** — regressão visual, acessibilidade | Testes com `step_dt=1/60`, assertivas monotonicidade, harness estrito, VL controlado. |
| **R7 — Dados/evidência** | Arquivos EVTX, hashes, custódia, PII, secrets | Dados de caso, exports, manifests e logs | **Alto** — vazamento, adulteração, perda | Validar e canonicalizar paths; não logar nomes/conteúdo sensíveis; usar sandbox e diagnósticos sem dados. |
| **R8 — Ações externas** | Push, PR, dispatch workflow, envio email, pagamento | `git push`, `gh`, MCP web, browser | **Crítico** — irreversível, externo | **Nunca** por worker; só Cricket com autorização explícita (ação, alvo, escopo); `gh auth git-credential`; sem tokens no repo. |

### Escalas de decisão por classe

| Classe | Quem detecta | Quem decide | Prazo | Evidência mínima |
|---|---|---|---|---|
| R1, R8 | Qualquer (worker relata) | **Cricket** (root) | Imediato — para a coordenação | Classificação de risco, autoridade aplicável ou autorização explícita registrada |
| R2, R3, R7 | Ox (tooling), Fox (integração), Rat (artefatos) | **Fox** eleva para Cricket se crítico | ≤ 15 min após detecção | Log absoluto, hash, diagnóstico, baseline |
| R4, R5 | Ox (linter, supply chain) | **Fox** (integração) / Cricket (ADR) | Próximo gate | `cargo deny`/`audit`, migration, contrato JSONL |
| R6 | Lobster (VL), Fox (testes) | **Fox** (testes) / Cricket (aceite estético) | Orçamento lane | Schema VL, teste harness, relatório textual |

### Regra de ouro
**Nenhum worker decide sobre R1 ou R8.** `unsafe` novo é escalado ao root e segue a autoridade aplicável. Ação externa exige autorização explícita do operador com ação, alvo e escopo registrados pelo Cricket. O playbook não concede autorização nem cria requisito adicional.

---

## CP-17. Workflow visual: uma imagem por vez, schema, rejeição, RGB e limites de aceite

### Gatilho de entrada

O workflow é necessário quando uma revisão estética for explicitamente autorizada e houver imagem sintética isolada. Se a missão proibir imagens, esta lane produz somente grounding textual e registra que nenhuma nova observação visual ocorreu.

### Insumos e schema estrito

O schema aceito pelo validador local é estrito:

- raiz: `schema_version`, `reviewer`, `images`, `overall`, `limitations`, sem chaves extras (`validate_structured_report.py:28-42`);
- `schema_version`: `1`; `reviewer`: deve corresponder ao papel esperado (`validate_structured_report.py:35-36`);
- `overall`: `consistent`, `issues` ou `uncertain` (`validate_structured_report.py:7-9`);
- `images`: uma ou duas entradas, cada uma com `file`, `observed`, `notes` e `confidence` (`validate_structured_report.py:43-54`);
- `confidence`: `high`, `medium` ou `low` (`validate_structured_report.py:7-8` e `validate_structured_report.py:59-61`);
- `file`: somente `reopen-main.png` ou `reopen-preferences.png`, sem path e sem duplicidade (`validate_structured_report.py:9` e `validate_structured_report.py:55-57`);
- para `nano_omni`, também são obrigatórios `labels` e `control_states` (`validate_structured_report.py:49-53` e `validate_structured_report.py:62-68`).

O fixture positivo confirma uma entrada e nenhum campo proibido (`<SCRATCH_DIR>`). O fixture negativo inclui confiança inválida e `base64`, que deve ser rejeitado (`<SCRATCH_DIR>`).

### Passos operacionais

1. Validar o schema antes de abrir qualquer imagem. Um payload inválido não entra na revisão.
2. Confirmar que a imagem é sintética, isolada e está na allowlist. Imagem real, dado de caso, nome físico de evidência ou captura não autorizada não entra no fluxo.
3. Restringir a revisão a uma imagem por vez e um worker individual isolado. O relatório textual é o único retorno ao coordenador (`AGENTS.md:243`).
4. Registrar, para cada imagem, `observed`, notas factuais e `confidence`. Não converter ausência de observação em defeito visual.
5. Usar RGB apenas para confirmar amostras, transições ou valores esperados de uma amostra definida. RGB não mede estética, hierarquia, legibilidade ou qualidade de animação; essa limitação também consta em `docs/SESSION_LEARNINGS.md`.
6. Emitir `overall` somente a partir das observações. `uncertain` é o estado honesto para indisponibilidade, rejeição ou ausência de inspeção válida.
7. Não anexar imagem, base64, bytes ou transcrição visual. O relatório referencia somente o nome lógico permitido.

### Saída

Uma seção de relatório com `overall`, uma entrada por imagem, observações, confiança e limitações, validada contra o schema estrito. Sem inspeção autorizada, a saída é apenas documental e não declara nova entrada visual.

### Pós-condições

- Schema validado antes de inspeção.
- Uma imagem por vez, com nome na allowlist.
- `overall`, `observed` e `confidence` coerentes entre si.
- Nenhum base64, byte, anexo ou caminho de captura no relatório.
- Estados de preferências permanecem separados. A revisão anterior registrou `Customizado` com matiz 160, croma 0.300 e movimento reduzido ativo (`super-vl-visual-review.json:11-16`), enquanto a revisão golden registrou Vento Áureo com movimento reduzido desmarcado (`super-vl-golden-visual-review.json:11-16`). Esses quadros não podem ser combinados como um único estado.

### Limites de aceite

Uma imagem pode sustentar apenas o que foi observado naquele quadro e naquele estado. Quadros estáticos não provam animação (`super-vl-visual-review.json:19-23` e `super-vl-golden-visual-review.json:19-23`). Uma diferença de preferências entre rodadas é estado diferente, não regressão ou melhora. A revisão anterior não usou amostras RGB, e o contraste observado foi subjetivo (`super-vl-golden-visual-review.json:19-23`). A implementação da interface, por outro lado, mantém testes explícitos de contraste mínimo 4.5:1 (`src/gui/theme.rs:531-563`), o que é uma garantia de implementação e não substitui inspeção estética.

### Falha, indisponibilidade e escalada

- Rejeição pelo provedor: encerrar a lane visual sem repetir a entrada, trocar de modelo ou contornar o filtro. O aceite estético permanece `PENDING-HUMAN` (`AGENTS.md:243`).
- Imagem ausente, ilegível ou fora da allowlist: `observed=false`, confiança baixa ou média conforme o que foi possível verificar, e `overall=uncertain` se a observação necessária não ocorreu.
- Schema inválido: corrigir o payload textual antes de qualquer inspeção. Repetir a mesma estrutura inválida não é nova evidência.
- Duas tentativas sem informação nova: mudar de método ou consolidar como `BLOCKED`/`PENDING-HUMAN`, sem retry infinito.

### Checklist acionável

- [ ] Schema validado antes da imagem.
- [ ] Entrada é sintética, isolada e autorizada.
- [ ] Nome está na allowlist e não se repete.
- [ ] Uma imagem por vez.
- [ ] `observed` é booleano e `confidence` pertence ao enum.
- [ ] `overall` pertence ao enum.
- [ ] Estados de preferência não foram misturados.
- [ ] RGB, quando presente, está ligado a uma amostra definida e não foi descrito como estética.
- [ ] Relatório não contém base64, bytes, caminho físico ou captura embutida.
- [ ] Limitação de quadro estático está explícita.

---

## CP-18 — Workflow Rust: ordem de gates, dono de build, tempo determinístico e warnings

### Gatilho de entrada

Uma mudança Rust afeta compilação, testes, lint, API, comportamento ou dependências. A autoridade operacional permanece `docs/RELEASE_GATE.md`, `docs/EVIDENCE_REGISTER.md` e o `justfile`.

### Insumos

- Checkout/HEAD e diff da mudança.
- Tipo de mudança e gates aplicáveis.
- Dono único de build e ordem de vez.
- Logs da execução anterior, incluindo warnings antes da correção.

### Passos

1. **Identificar o dono único.** O Cricket concede a vez; um único agente executa todo trabalho Rust pesado naquele checkout. Fox e workers aguardam.
2. **Congelar o plano de gates.** Registrar, antes de executar, a sequência, o ID do trabalho em segundo plano, o path absoluto de cada log e o critério de staleness.
3. **Executar sequencialmente.** Usar `just verify` como gate oficial, cuja ordem atual é `just validate-fase0`, `just fmt`, `just check`, `just nextest`, `just doctest` e `just clippy`. Se algum estágio for executado individualmente, preservar a mesma ordem. Usar `just supply-chain` quando lock, dependências ou advisories mudarem. A autoridade da ordem é o `justfile`, não este resumo.
4. **Usar um único trabalho em segundo plano por vez.** Se um comando for movido para segundo plano, o dono espera `bg wait` no **mesmo ID completo** antes de iniciar o próximo comando Rust. Não usar ID de `await_members` em `bg wait`.
5. **Preservar warnings pré-correção.** Se o primeiro gate produzir warnings, guardar o log original antes da correção. O relatório final distingue “warnings no baseline” do “execução final limpa”, sem reescrever o primeiro resultado.
6. **Tratar concorrência como staleness.** Testes executados concorrentemente no mesmo checkout são `STALE` até uma repetição sequencial da sequência aplicável. Resultados concorrentes não entram como PASS.
7. **Confirmar pós-condição.** Para cada gate, ler exit, stdout/stderr e log absoluto, contar testes/warnings e verificar a árvore esperada. Exit 0 não é suficiente.
8. **Fechar.** Confirmar que não há processo Cargo/Rustc/nextest próprio, revisar status/diff e vincular a evidência a commit/tree.

### Ordem canônica e gates adicionais

| Ordem | Gate | Postcondição |
|---:|---|---|
| 1 | `just validate-fase0` | Validação estrutural inicial concluída |
| 2 | `just fmt` | Diff de formatação ausente |
| 3 | `just check` | Todos os targets compilam com a árvore locked |
| 4 | `just nextest` | Contagem de testes e skips lida |
| 5 | `just doctest` | Doc-tests executados e contados |
| 6 | `just clippy` | Clippy sem warnings bloqueados |
| Adicional | `just supply-chain` | Lock/advisories validados quando aplicável |

### Saída

- Sequência executada com dono, ordem, IDs e logs absolutos.
- Contagens, warnings pré-correção e warnings finais.
- Estado de staleness e vínculo a commit/tree.

### Pós-condição

- A ordem é explícita e sequencial.
- Existe um único dono de build.
- Nenhum segundo comando Rust começou antes do `bg wait` no mesmo ID do trabalho em segundo plano.
- Warnings pré-correção foram preservados.
- Resultados concorrentes foram marcados `STALE` até repetição sequencial.

### Critério de parada e escalada

Parar e preservar o diagnóstico ao primeiro gate Rust que falhar. Não iniciar o próximo stage depois de uma falha de fmt/check/clippy. Não cancelar compilação legítima por silêncio sem inspecionar progresso, processo e consumo. Escalar OOM, ENOSPC, lock/contenção, mudança de API não autorizada ou falha persistente.

---

## CP-19. Workflow documental e deduplicação por propriedade/autoridade

### Gatilho de entrada

O workflow é acionado quando um documento precisa ser sincronizado com o código, ou quando uma nova fonte pode repetir regras já propriedade de outra autoridade. `docs/DOCUMENT_AUTHORITY.md` determina que o playbook seja guia derivado, não nova autoridade nem compilação das mesmas regras.

### Autoridade proprietária por seção

| Seção do playbook | Autoridade proprietária | Forma correta no playbook |
|---|---|---|
| Ferramentas, economia de contexto | `AGENTS.md` §15 (`AGENTS.md:195-214`) | Referenciar custos, dependências e casos de uso, sem copiar o catálogo inteiro. |
| Autoaprimoramento e ciclo de sessão | `AGENTS.md` §16 e `docs/SESSION_IMPROVEMENT.md` (`AGENTS.md:216-228`; `docs/SESSION_IMPROVEMENT.md:6-16`) | Referenciar o procedimento e inserir pontos de confirmação por papel. |
| Swarm, workers e anexos visuais | `AGENTS.md` §17 (`AGENTS.md:230-245`) | Referenciar o template e os limites; não reproduzir o template. |
| Skills | `AGENTS.md` §17 (`AGENTS.md:247-253`) | Referenciar cada skill pelo gatilho e pela pós-condição. |
| MCPs e fontes externas | `AGENTS.md` §17 (`AGENTS.md:255-261`) | Referenciar o envelope, a validação e os limites de dados. |
| Convenções de código e segurança | `CONVENTIONS.md:35-110` | Referenciar a regra aplicável por mudança, sem criar lista paralela. |
| Procedimento de verificação | `.jcode/skills/dfir-verify/SKILL.md:17-39` | Referenciar o gate apropriado e exigir logs, pós-condição e controles negativos. |
| Grounding visual e revisão controlada | `src/gui/theme.rs`, `src/gui/effects.rs`, `src/gui/mod.rs` e relatórios textuais sanitizados | Registrar decisões visuais com arquivo e linha e manter o código como autoridade de implementação. |
| Procedimento de melhoria | `.jcode/skills/dfir-improve/SKILL.md:13-58` | Referenciar abertura, execução, integração e fechamento sem duplicar o checklist. |
| Template de swarm | `.jcode/swarm-prompt.md` | Exigir seus campos preenchidos e explicar o uso, sem incorporá-lo. |

O playbook recebe entrada derivada em `docs/DOCUMENT_AUTHORITY.md`, mas não é promovido a requisito, ADR ou RTM.

### Passos operacionais

1. Identificar a autoridade proprietária de cada seção antes de redigir.
2. Referenciar a autoridade em vez de copiar sua lista de regras.
3. Transformar lições reais em controles operacionais ligados ao artefato lógico, sem expor dados sensíveis (`docs/SESSION_LEARNINGS.md`).
4. Verificar duplicação por função. Exit 0 versus pós-condição, template de swarm e skills repetem limites porque protegem pontos diferentes do ciclo (`docs/SESSION_IMPROVEMENT.md`).
5. Proteger `.jcode/swarm-prompt.md`, skills, `AGENTS.md`, `docs/SESSION_IMPROVEMENT.md` e demais procedimentos normativos. Não removê-los nem reescrevê-los por deduplicação (`docs/DOCUMENT_AUTHORITY.md`).
6. Se houver conflito entre playbook derivado e autoridade, prevalece a autoridade e o conflito deve ser elevado ao root.

### Saída

Uma seção de workflow documental com tabela de propriedade, regras de referência, exceções de redundância, checklist de deduplicação e escalada de conflito.

### Pós-condições

- Cada regra nova tem uma referência à autoridade ou ao fato sanitizado que a origina.
- Nenhuma seção normativa é copiada como nova autoridade.
- Redundâncias intencionais continuam presentes nas autoridades originais.
- `.jcode/swarm-prompt.md` e as skills locais permanecem presentes e referenciados.
- O playbook declara-se derivado e não altera estados de release, requisitos, RTM ou ADRs.

### Checklist acionável

- [ ] A autoridade proprietária de cada seção foi identificada antes da redação.
- [ ] A seção referencia a autoridade, sem copiar sua lista operacional.
- [ ] A regra derivada aponta para o fato ou artefato lógico sanitizado.
- [ ] Exit 0 e pós-condição permanecem distinguíveis.
- [ ] O template de swarm não foi incorporado nem removido.
- [ ] As skills não foram clonadas nem substituídas por uma nova skill.
- [ ] `AGENTS.md`, `CONVENTIONS.md` e `docs/SESSION_IMPROVEMENT.md` não foram reescritos.
- [ ] O texto não afirma estado de release, aceite ou autoridade nova.

### Limites e escalada

O root revisa o diff semântico e decide a integração. Divergência entre relatório e arquivo é resolvida pela leitura do arquivo e do diff, não pelo anúncio. O checklist controla pipes de tabela, newline final, whitespace e coerência entre tabela e prosa; exemplos de falhas anteriores estão em `docs/SESSION_LEARNINGS.md`.

---

## CP-20 — Contrato de artefatos: Before/After, logs, relatórios, task cards e paths

### Gatilho

Integrar uma entrega de lane no draft canônico.

### Insumos

1. Artefato final da lane.
2. Estado do repositório antes e depois.
3. Logs absolutos, diffs semânticos e task cards.
4. Schema comum de entrega distribuído na rodada Rat.

### Passos

1. Registrar Before antes da escrita: timestamp, raiz do checkout, HEAD, tree se aplicável e `git status --short`.
2. Congelar baseline por hash para qualquer arquivo crítico: `path`, `sha256`, `captured_at` e, quando aplicável, tamanho.
3. Produzir After com o mesmo conjunto de campos e listar somente os artefatos realmente produzidos ou alterados.
4. Para cada comando, guardar `command`, `cwd`, exit, diagnóstico, pós-condição e link para log.
5. Para cada artefato final, guardar path, hash, bytes, papel e links para claims.
6. Atualizar a task card com ID estável. IDs não são reutilizados nem renomeados de modo a quebrar dependências.
7. Entregar o relatório como índice. O root relê o artefato e recontas logs, warnings, contagens e hashes.

### Saída

- Registro Before/After em `before` e `after`.
- Registro de artefato em `artifacts[]`.
- Registro de comando em `commands[]`.
- Task card rastreável em `task_card`.
- Handoff em `handoff`, quando a entrega avança para outra etapa.

### Pós-condição

- Todo arquivo produzido tem path rastreável e hash.
- Todo log tem path absoluto e pode ser lido pelo root.
- Before/After captura a mudança sem depender de narrativa.
- A task card liga objetivo, estado, dependências, arquivos e validação.
- Relatório divergente do artefato é rejeitado, não integrado.

### Critério de parada e falha/escalada

- Parar se houver path relativo de artefato de rodada, log ausente, hash divergente, exit sem diagnóstico ou relatório incompatível com o arquivo final.
- Escalar divergência factual ao Fox e ao root. Se a escrita estiver atrasada, confirmar o estado terminal do worker e reler o artefato imediatamente antes de assumir a escrita.

### Campos obrigatórios do artefato

| Grupo | Campos obrigatórios |
|---|---|
| Identidade | `path`, `kind`, `role`, `sha256`, `bytes` |
| Baseline | `baseline.path`, `baseline.sha256`, `baseline.captured_at` |
| Comando | `command`, `cwd`, `exit_code`, `diagnostic`, `postcondition`, `evidence[]` |
| Before/After | `captured_at`, `checkout_root`, `head`, `status_short[]`; `tree` quando disponível |
| Task card | `id`, `description`, `owner`, `status`, `dependencies[]`, `before`, `after`, `validation`, `updated_at` |
| Pendência | `pending[]`, `blocked[]`, `pending_human[]`; cada item inclui o motivo |
| Evidência | `path`, `kind`, `claim`, `verification_status`; `sha256` e `root_note` quando aplicáveis |

### Exemplo preenchido de Before/After

```json
{
  "before": {
    "captured_at": "<TIMESTAMP_ISO8601>",
    "checkout_root": "<CHECKOUT>",
    "head": "<HEAD_ATUAL>",
    "status_short": [
      "?? .scratch/arquivo-preexistente.txt"
    ]
  },
  "after": {
    "captured_at": "<TIMESTAMP_ISO8601>",
    "checkout_root": "<CHECKOUT>",
    "head": "<HEAD_ATUAL>",
    "status_short": [
      "?? .scratch/arquivo-preexistente.txt"
    ]
  }
}
```

O exemplo não afirma que a árvore ficou inalterada. A comparação entre os dois snapshots e os paths explícitos em `artifacts[]` permite ao root observar que a lane escreveu fora do checkout.

### Exemplo preenchido de artefato versionado

```json
{
  "artifacts": [
    {
      "path": "docs/SESSION_LEARNINGS.md",
      "kind": "markdown",
      "role": "Documento vivo de lições, exemplo de artefato versionado rastreável",
      "sha256": "<SHA256_CALCULADO>",
      "bytes": 33843,
      "links": [
        {
          "path": "docs/SESSION_LEARNINGS.md",
          "kind": "artifact",
          "claim": "O documento vivo mantém as lições 1 a 29 numeradas",
          "sha256": "<SHA256_CALCULADO>",
          "verification_status": "pending_root_read"
        }
      ]
    }
  ],
  "commands": [
    {
      "command": "sha256sum docs/SESSION_LEARNINGS.md",
      "cwd": "<CHECKOUT>",
      "exit_code": 0,
      "diagnostic": "sha256sum printed one digest line for the file",
      "postcondition": "O digest do arquivo é reprodutível a partir do path versionado",
      "evidence": [
        {
          "path": "docs/SESSION_LEARNINGS.md",
          "kind": "artifact",
          "claim": "O arquivo cujo digest foi calculado existe no repositório",
          "verification_status": "pending_root_read"
        }
      ]
    }
  ]
}
```

O hash e o tamanho acima são reais para `docs/SESSION_LEARNINGS.md` no HEAD de referência da rodada, reproduzidos aqui como exemplo de forma. O emissor substitui path, `sha256` e `bytes` pelos valores calculados do artefato final que está indexando. A task card valida `sha256 != <SHA256_CALCULADO>` e `bytes > 0` para artefatos existentes.

### Exemplo preenchido de task card rastreável

```json
{
  "id": "task-003",
  "title": "Normalizar artefatos e contratos de entrega da rodada",
  "description": "Produzir schema comum, contrato Before/After, métricas e templates copiáveis para as lanes da rodada.",
  "owner": "Rat",
  "status": "active",
  "priority": "high",
  "source": "Requisito explícito do root para a rodada do playbook de coordenação",
  "why_selected": "O playbook depende de um contrato estável para integrar relatórios sem tratá-los como evidência.",
  "verifiability": "O root lê os artefatos finais, valida o JSON e recontas os links absolutos.",
  "risk": "low",
  "dependencies": ["task-001"],
  "before": {
    "problem": "As lanes usavam formatos divergentes e paths relativos, impedindo recontagem uniforme.",
    "evidence": [
      {
        "path": "docs/SESSION_LEARNINGS.md",
        "kind": "source",
        "claim": "A lição 22 registra logs relativos/ausentes e relatórios sem artefato; a lição 25 registra procedimento substituído por relatório de lane",
        "verification_status": "pending_root_read"
      }
    ]
  },
  "after": {
    "change": "A lane Rat produziu schema comum e formatos Before/After, métricas e handoff, e registrou o aprendizado no documento vivo.",
    "files_changed": ["docs/SESSION_LEARNINGS.md"],
    "evidence": [
      {
        "path": "docs/SESSION_LEARNINGS.md",
        "kind": "artifact",
        "claim": "O documento vivo registra a prevenção aplicada e a lição nova, se houver",
        "verification_status": "pending_root_read"
      }
    ]
  },
  "validation": {
    "commands": ["sha256sum docs/SESSION_LEARNINGS.md"],
    "result": "observed",
    "evidence": [
      {
        "path": "docs/SESSION_LEARNINGS.md",
        "kind": "artifact",
        "claim": "Digest do artefato final calculado",
        "verification_status": "pending_root_read"
      }
    ]
  },
  "followups": [
    "Root lê os artefatos finais e substitui verification_status conforme a leitura"
  ],
  "updated_at": "<TIMESTAMP_ISO8601>"
}
```

### Contrato de paths

- Artefato versionado usa path relativo ao repositório, por exemplo `docs/COORDINATION_PLAYBOOK.md`.
- Artefato ou log fora do checkout usa path absoluto real e é indexado no relatório de entrega da rodada, não no documento versionado.
- Um path relativo nunca prova destino. O integrador localiza e lê o arquivo real.
- Não registrar nomes de evidência, conteúdo de eventos ou secrets em logs e diagnósticos.

---

## CP-21 — Gates por mudança, controles negativos, staleness e evidência de tree

### Gatilho de entrada

Uma mudança será integrada, declarada implementada ou apresentada como verificada. O gatilho exige seleção de gates antes da alegação de estado, não depois.

### Insumos

- Natureza e risco da mudança.
- Checkout/HEAD e diff.
- Autoridade de gates: `docs/RELEASE_GATE.md` e `docs/EVIDENCE_REGISTER.md`.
- Logs e scripts de gate aplicáveis.
- Estado anterior das evidências e tree/commit associado.

### Passos

1. **Classificar a mudança.** Documentação/contrato, Python/QA, Rust, dependência, fluxo com engine real ou combinação. Cada tipo tem gate próprio.
2. **Selecionar gates.** Registrar comando, ordem, dono, path absoluto de log e critério de pós-condição antes da execução.
3. **Executar controle positivo.** O caminho válido deve passar. O log é lido; exit 0 não basta.
4. **Criar controle negativo isolado.** Copiar somente o artefato sanitizado em `$JCODE_SCRATCH_DIR` ou em diretório de fixture autorizado, violar **exatamente uma** condição nova e executar o mesmo gate. Nunca induzir a falha no checkout real.
5. **Confirmar a falha esperada.** Verificar exit diferente de zero, regra esperada no diagnóstico e pós-condição da detecção. Ler o log original completo. Um parser que procurou substring incompleta não é nova evidência.
6. **Executar o conjunto aplicável.** O resultado positivo mais controles negativos demonstra detecção estrutural; não mede qualidade textual nem concede aceite semântico.
7. **Verificar staleness.** Se código, lock, toolchain, dataset, banco de advisories ou outro insumo do gate mudou, evidência anterior fica `STALE` e o gate deve ser repetido. Idade não prova staleness; mudança do insumo prova.
8. **Vincular evidência.** Registrar commit, tree, hash dos artefatos/binários relevantes, comando, exit, contagens, warnings e path absoluto. Relatório é índice; log/hash é evidência.
9. **Revisar semanticamente.** Conferir diff, `git diff --check`, status e processos próprios. Corrigir divergência entre artefato e relatório antes de integrar.
10. **Fechar honestamente.** Reportar PASS/PENDING/BLOCKED/STALE apenas quando observados. `WAIVED` é governança, não verificação.

### Matriz de gates por tipo de mudança

| Tipo de mudança | Controle positivo | Controle negativo mínimo | Evidência mínima |
|---|---|---|---|
| Documentação/contrato | `just doc-gate` e linter específico do documento | Remover uma condição em cópia/fixture e exigir exit != 0 + regra | Comando, exit, log absoluto, diff, commit/tree |
| Python/QA tooling | Teste unittest/pytest aplicável e linter próprio | Mutar uma regra isoladamente e exigir detecção específica | Contagem de testes, diagnóstico, path de scratch, hash |
| Rust | `just verify` na ordem oficial | Controle comportamental focado quando um teste novo é introduzido | Logs de fmt/check/clippy/nextest/doc, warnings, tree |
| Dependência | `cargo metadata --locked`, `cargo tree --locked`, `cargo deny`, `cargo audit` | Revisar advisory/duplicate/tree de feature conforme gate oficial | Versões resolvidas, advisories, commit/tree |
| Fluxo com engine real | E2E com fixture sanitizada | Quando a mudança altera o parser, fixture negativa sintética aplicável | Engine/app hashes, fixture, contagens, log absoluto |

### Linter estrutural do Coordination Playbook

O linter `scripts/coordination_playbook_gate.py` usa apenas Python stdlib e implementa estas regras determinísticas:

1. `missing_cp_id`: ausência de qualquer `CP-00` a `CP-28`.
2. `duplicate_cp_id`: ID CP repetido em mais de um heading.
3. `empty_cp_section`: heading CP sem conteúdo textual.
4. `placeholder_field`: presença de `{{...}}`.
5. `placeholder_token`: presença isolada de `TODO` ou `TBD`.
6. `missing_reference`: ausência de referência obrigatória a autoridades, template, skills, gate e registro de evidência.
7. `missing_role`: ausência de heading dedicado para Cricket, Fox, Lobster, Ox ou Rat.
8. `excessive_duplicate`: mesmo parágrafo normalizado repetido três ou mais vezes.

O linter **não** mede qualidade por linhas, não infere sentido, não valida a verdade factual e **não** declara aceite semântico. O diagnóstico tem path absoluto, linha, regra e mensagem. O caminho válido retorna exit 0; cada mutação negativa isolada retorna exit 1 com a regra esperada.

Headings de CP/papel e parágrafos de duplicação são avaliados após máscara de fences CommonMark com backtick ou til, comprimento mínimo de três, fechamento compatível e tratamento de EOF, preservando offsets e CRLF. Código literal não satisfaz heading nem produz duplicata estrutural. A detecção de `placeholder_field` e `placeholder_token` permanece global, inclusive em fences, para que templates não resolvidos não se escondam em exemplos.

A auditoria complementar `scripts/coordination_playbook_repo_audit.py` separa os contextos do playbook, de `docs/SESSION_LEARNINGS.md` e da `justfile`. Ela faz parse de blocos JSON, verifica referências versionadas em backticks com ou sem sufixo de linha, aceita diretórios existentes, valida a sequência contígua das lições e compara somente a receita `verify` aos seis passos canônicos. Saída detalhada usa `--verbose`; a saída padrão informa contagens.

Comandos de verificação e análise:

```bash
python3 scripts/coordination_playbook_gate.py
python3 scripts/coordination_playbook_repo_audit.py
just test-coordination-playbook
just test-coordination-playbook-all
just benchmark-coordination-playbook
```

A suíte completa exige `JCODE_SCRATCH_DIR` dedicado para os controles subprocessados. O benchmark não integra o gate obrigatório e não é meta universal de desempenho.

### Staleness e vínculo de tree

| Mudança desde a evidência | Estado | Ação |
|---|---|---|
| Nenhuma mudança nos insumos do gate | Gate anterior ainda candidato a reuso | Confirmar commit/tree e ler log original |
| Código, docs, lock, toolchain, fixture, dataset ou advisories relevante mudou | `STALE` | Repetir o gate afetado |
| Build/test concorrente no mesmo checkout | `STALE` | Repetir sequencialmente sob um único dono |
| Gate não executado por ferramenta ausente | `PENDING` ou `BLOCKED` | Registrar bloqueio real, não PASS |
| Waiver explícito | `WAIVED` | Registrar escopo e autoridade; não chamar de verificado |

### Saída

- Matriz de gates executados.
- Controle positivo e negativos com regra esperada.
- Evidência localizável e vínculo a commit/tree.
- Pendências reais e estados de staleness.

### Pós-condição

- O linter é Python stdlib, determinístico e limitado a estrutura.
- O positivo passa; cada mutação negativa falha pela regra esperada.
- Nenhum diagnóstico é inferido por contagem de linhas e nenhum aceite semântico é emitido.
- Evidência obsoleta foi marcada `STALE` e reexecutada ou deixada explicitamente pendente.
- Toda PASS observada está vinculada a commit/tree e a logs absolutos.

### Critério de parada e escalada

Parar quando uma regra negativa não for detectada, o log não puder ser localizado, a tree mudar durante o gate, o checkout divergir ou o escopo for violado. Escalar falha de linter, divergência entre fixture e regra, dependência ausente, risco a dados, autorização externa ou qualquer pedido de aceite sem revisão semântica.

---

## CP-22 — Runbooks de recuperação: cwd errado, schema inválido, busca vazia, build silencioso, relatório divergente, escrita atrasada e cancelamento

### RB-01 — CWD errado (worker no diretório incorreto)
**Sintoma**: `git rev-parse --show-toplevel` diverge do checkout absoluto da missão.
**Detecção**: Início de worker (contrato exige confirmação), ou `swarm message` relatando path.
**Recuperação**:
1. Fox envia DM: "Corrija a sessão existente — não crie worker duplicado."
2. Worker executa `cd <checkout_absoluto>` e reconfirma `git rev-parse HEAD`.
3. Se já escreveu fora do escopo ou do checkout, interromper a escrita, preservar o artefato sem integrá-lo e escalar ao root. Corrigir a sessão existente; substituir worker somente depois de estado terminal e decisão explícita, nunca como reação automática.
**Prevenção**: Contrato de lane inclui checkout absoluto; spawn inclui `working_dir` mas worker **deve** confirmar.

### RB-02 — Schema inválido (linter/template/swarm prompt falha validação)
**Sintoma**: Linter do Ox reporta falha: IDs ausentes, placeholders, referências obrigatórias, papéis ausentes, duplicação excessiva.
**Detecção**: Gate `scripts/coordination_playbook_gate.py` falha.
**Recuperação**:
1. Fox lê diagnóstico do linter (arquivo/regra/linha).
2. Identifica CP afetado e autor (Fox ou worker).
3. Se Fox: corrige no draft, reexecuta linter.
4. Se worker: devolve com diagnóstico, worker corrige e reentrega.
5. Não prossegue para integração enquanto linter falhar.
**Prevenção**: Linter roda **antes** de integração; templates preenchidos validados no intake.

### RB-03 — Busca vazia inesperada (agentgrep/read/glob retorna vazio mas esperado conteúdo)
**Sintoma**: Busca por símbolo/arquivo/termo conhecido retorna vazio.
**Detecção**: Fox ou worker relata "busca vazia inesperada" no relatório.
**Recuperação**:
1. **Controle positivo**: buscar termo que DEVE existir (ex.: `fn main` em Rust, `CP-00` no playbook).
2. Se controle positivo falha → ferramenta/index com problema → usar método alternativo (`bash` com `rg`/`fd`, ou leitura direta `read`).
3. Se controle positivo passa → conteúdo realmente ausente → registrar como `PENDING` ou `BLOCKED` (dependência não entregue).
4. **Não repetir** a mesma busca sem informação nova (máx 2 tentativas).
**Prevenção**: AGENTS.md §16 — busca vazia inesperada exige controle positivo ou método alternativo.

### RB-04 — Build silencioso (cargo/nextest sem output por > 15 min)
**Sintoma**: `bg tail` ou `swarm status` mostra worker ativo mas sem progresso visível.
**Detecção**: Fox monitora `bg list` / `swarm list` a cada ciclo.
**Recuperação**:
1. `bg output <job_id>` ou `swarm message` para inspecionar progresso/consumo (CPU, RAM, disco).
2. Se compilação legítima (ex.: crate grande) → aguardar, não cancelar.
3. Se travado (OOM, deadlock, disco cheio) → `bg cancel <job_id>` / `swarm stop`, registrar diagnóstico, reavaliar orçamento.
4. **Nunca cancelar** antes de inspecionar.
**Prevenção**: Builds Rust pesados têm dono único + `bg wait` no mesmo ID; `ENOSPC` guardado por `cargo clean` + `df -h` prévio (EVD-055).

### RB-05 — Relatório divergente do arquivo (worker anuncia X, arquivo contém Y)
**Sintoma**: Fox lê artefato final e diff semântico não bate com `worker-delivery.json`.
**Detecção**: Passo 4 do CP-11 — "ler artefato final **antes** de integrar".
**Recuperação**:
1. **Não integrar**. Devolver ao worker com: "Relatório diverge do arquivo. Diferença: [detalhe]. Corrija e reentregue ambos."
2. Worker corrige artefato **e** relatório, reenvia JSON atualizado.
3. Se worker parado → Fox assume escrita **após** confirmar estado terminal e reler artefato (escrita atrasada = nova entrega concorrente).
**Prevenção**: Contrato exige relatório com comandos/exits/paths absolutos/contagens/diagnósticos; `git status`/index são fonte do estado.

### RB-06 — Escrita atrasada (worker parado, Fox assume escrita)
**Sintoma**: Worker status `stopped`/`failed`, Fox precisa completar artefato.
**Detecção**: `swarm status` ou `bg status` mostram worker terminal.
**Recuperação**:
1. Confirmar estado terminal do worker (`swarm status <id>` → `stopped`/`completed`/`failed`).
2. Reler artefato **imediatamente antes** de editar (escrita atrasada = nova entrega concorrente a revisar).
3. Editar apenas no escopo da lane original.
4. Registrar no `fox-decisions.md`: "Escrita assumida por Fox após worker <id> <status>. Diff: [resumo]."
5. Não sobrescrever correções concorrentes — revisar diff antes.
**Prevenção**: Acompanhar progresso, não deixar worker aberto enquanto só espera.

### RB-07 — Cancelamento de coordenação (orçamento esgotado, dependência ausente, risco)
**Sintoma**: Critérios de parada CP-04/CP-15 atingidos.
**Detecção**: Fox ou Cricket identifica critério.
**Recuperação**:
1. Registrar em `fox-decisions.md`: critério, decisão (consolidar/parar), pendências.
2. Confirmar IDs completos e estado antes de `swarm stop`; registrar IDs de `await_members` em `todo.waiting_on`, nunca usar worker ID como ID de `bg wait`.
3. `bg cancel` jobs próprios ativos.
4. Consolidar resultado parcial útil (artefatos entregues e válidos).
5. Relatório final: o que foi feito, o que não, pendências, ações externas autorizadas.
6. `swarm list` / `bg list` → zerados.
**Prevenção**: Orçamentos definidos no intake; consolidação precoce CP-04; Cricket não abre workers enquanto só espera.

### RB-08 — Await expira ou worker apresenta downtime/qualidade insuficiente
**Sintoma**: Orçamento ou await termina sem entrega, ou o artefato tem defeito material reproduzido.
**Recuperação**:
1. Confirmar status, último artefato e consumo; não interpretar silêncio como falha permanente.
2. Fazer uma devolução objetiva com arquivo, diagnóstico e pós-condição.
3. Reproduzir o problema no estado atual antes de trocar modelo ou editar.
4. Se o fallback for necessário e autorizado, consultar `/agents`, `/subagent-model` e `swarm list_models` quando disponíveis; registrar modelo efetivo por role.
5. Confirmar estado terminal antes de substituir worker e manter o mesmo escopo. Não alterar configuração global por padrão e nunca trocar modelo para contornar rejeição visual.
6. Se o root assumir, escrever somente em arquivos reservados ao root; entrega tardia do worker é nova revisão concorrente.
**Pós-condição**: Uma única lane ativa por artefato, decisão registrada e nenhum trabalho duplicado.

### RB-09 — Auto-poke ou reload interrompe a espera/escrita
**Sintoma**: `/overnight` acorda a sessão com todos os itens incompletos ou reload aborta uma ferramenta.
**Recuperação**:
1. Inspecionar manifesto, notas, cartões, validação, Git e todo uma vez.
2. Se o await está pendente e não há trabalho independente útil, aguardar; não transformar o poke em polling ou nova tarefa.
3. Após reload, verificar existência, hash e conteúdo do arquivo antes de reescrever; ausência de confirmação não prova escrita nem ausência de escrita.
4. Manter cartões e notas atualizados antes de voltar a esperar.
**Pós-condição**: Nenhum arquivo duplicado e nenhuma chamada repetida sem nova informação.

### RB-10 — Automação não encontra headings em texto lido
**Sintoma**: Parser que deveria encontrar CPs retorna todos ausentes.
**Recuperação**:
1. Usar um heading conhecido como controle positivo.
2. Verificar se a ferramenta acrescentou numeração, anotações ou truncamento ao conteúdo.
3. Remover somente o prefixo de apresentação e repetir o controle; não editar o documento para compensar um parser inválido.
4. Após escrever, reler o arquivo e executar o linter real.
**Pós-condição**: Todos os 29 CPs são encontrados uma vez e a escrita é confirmada no arquivo, não no retorno da automação.

---

## CP-23 — Integração, revisão semântica, commits, processos e fechamento

### Integração (Fox → Cricket)

**Pré-condições editoriais do Fox**:
- [ ] Workers necessários entregaram ou resultado parcial útil e pendências foram registrados.
- [ ] Cada artefato final foi lido; diff, comandos/exits e pós-condições foram comparados ao relatório.
- [ ] O draft contém CP-00..CP-28 uma vez, sem placeholders de entrega ou duplicação acidental.
- [ ] Conflitos e omissões estão registrados para o root.
- [ ] Prechecks disponíveis têm logs e diagnósticos, sem promoção a aceite.

**Pré-condições finais do Cricket**:
- [ ] Linter do canônico e controles negativos passaram no estado atual.
- [ ] Schema foi validado quando aplicável e havia validador; indisponibilidade permanece PENDING.
- [ ] `just doc-gate` PASS depois da ligação mínima no `justfile`.
- [ ] `git diff --check` limpo e revisão semântica concluída.

**Processo de integração**:
1. Fox entrega `playbook-integrated.md` + `fox-delivery.json` + `fox-decisions.md` ao Cricket via DM.
2. Cricket revisa:
   - Rejeita texto sem fonte ou fora do escopo.
   - Verifica ligações de autoridade (preservation-map).
   - Confirma status, arquivos, ausência de mudanças alheias/processos.
3. Se revisão semântica falha → Cricket devolve a Fox com diagnóstico; Fox corrige e reentrega.
4. Se revisão passa → Cricket executa commits temáticos.

### Commits (Cricket)

**Regras**:
- Pequenos, temáticos, coesos (uma mudança lógica por commit).
- **Após** gates aplicáveis; schema somente quando exigido pela missão, nunca como substituto de leitura do artefato.
- Só arquivos da tarefa, como `docs/COORDINATION_PLAYBOOK.md`, linter, fixtures e entradas de autoridade/links.
- **Não incluem**: dados de caso, exports, bancos operacionais, artefatos derivados, mudanças alheias.
- Mensagem: `<scope>: <imperativo curto>` (ex.: `playbook: add CP-00..CP-28, linter, schemas`).
- `git diff --check` em cada commit.

**Ordem sugerida**:
1. `docs/COORDINATION_PLAYBOOK.md` (documento canônico)
2. `scripts/coordination_playbook_gate.py` + `tests/fixtures/coordination_playbook/**` (linter)
3. `docs/DOCUMENT_AUTHORITY.md` (entrada derivada curta)
4. `AGENTS.md` (link curto §17 se necessário)
5. Atualização de cartões/review (se aplicável)

### Processos e verificação pós-commit

1. `git status --short` → nenhum arquivo da tarefa fora do commit; mudanças alheias preservadas são listadas explicitamente, não apagadas nem apresentadas como limpeza absoluta.
2. `swarm list` → vazio (nenhum worker ativo).
3. `bg list` → vazio (nenhum job próprio ativo).
4. `SESSION_LEARNINGS.md` atualizado **se e somente se** lição nova recorrente/confirmada observada.
5. Relatório final ao Cricket (ou stakeholder):
   - Artefatos e arquivos
   - Pós-condições confirmadas
   - Comandos, exits, diagnósticos
   - Gates executados e resultado
   - Commits
   - Ações externas autorizadas e realizadas
   - Pendências e não executados
   - Estado final swarm/bg

### Fechamento obrigatório

**Checklist de fechamento** (Cricket executa, Fox apoia):
- [ ] Diff semântico revisado (`git diff HEAD~<n>`).
- [ ] `git diff --check` limpo.
- [ ] `git status --short` revisado; somente mudanças alheias conhecidas e preservadas podem permanecer.
- [ ] `swarm list` vazio.
- [ ] `bg list` vazio.
- [ ] Nenhum processo de GUI, engine, observer sem dono.
- [ ] Lições registradas (ou "sem nova lição de repositório").
- [ ] Relatório final entregue.

**Não fazer no fechamento**:
- Push ou publicação (requer autorização explícita separada).
- Declarar `PASS`/`VERIFIED`/`ACCEPTED` além da evidência.
- Converter limite de tempo em sucesso.
- Criar retrospectiva permanente de rotina (só `SESSION_LEARNINGS.md` se lição nova).

---

## CP-24 — Métricas de eficiência e qualidade, primeira passagem, retrabalho e aprendizado

### Gatilho

Revisão periódica do playbook ou fechamento de uma rodada com dados de toolcalls observáveis.

### Insumos

1. Journal da sessão da lane, localizado pelo session ID registrado no log de eventos da rodada.
2. Task cards da rodada, com ID, dono e dependências.
3. Log de eventos da rodada, com fase e timestamp.
4. Relatórios e artefatos finais com hashes.

### Definições e fórmulas

| Métrica | Definição operacional | Fonte primária |
|---|---|---|
| Toolcalls por tarefa | Total de mensagens `tool_use` do journal da sessão dividido pelo número de task cards atribuídos à sessão | Journal por session ID e task card `owner` |
| Toolcalls serializados | Toolcalls executados um a um quando independentes poderiam ter sido agrupados | Sequência de `tool_use`, `name` e timestamps |
| Tempo por fase | `fim_fase - início_fase` | Timestamps das fases ou mensagens do journal |
| Primeira passagem | `1` se a task foi aceita sem correção material após a primeira entrega; `0` caso contrário | Diff e revisões entre `before` e `after` |
| Retrabalho | Quantidade de edições no mesmo arquivo após a primeira escrita | Journal `tool_use` com `name` em `write`, `edit`, `apply_patch` e path-alvo |
| Reescrita integral | `1` se uma edição substituiu o arquivo inteiro para corrigir uma mudança local, `0` caso contrário | Comparação de tamanho e diff ou evento explícito da revisão |
| Relatório divergente | `1` se o relatório afirma algo que o artefato final contradiz, `0` caso contrário | Leitura do artefato final e do relatório |
| Tentativas sem informação nova | Pares de tentativas do mesmo método sem novo estado observável | Journal e diagnóstico do método |
| Lições registradas | Entradas novas em `docs/SESSION_LEARNINGS.md` ligadas à rodada | Diff e contagem por identificador lógico |
| Lições confirmadas | Lições cuja prevenção foi observada em ação posterior | Confirmação registrada com sessão e comando |

**Unidade de comparação:** a mesma task card, nunca “linhas”. Não use contagem de linhas, bytes ou tamanho do documento como proxy de qualidade. Volume textual é contexto, não medida de qualidade.

### Dados canônicos

Este playbook não fixa contagens de uma sessão. Cada rodada registra seu próprio snapshot, fonte e timestamp fora do documento durável. Números sem fonte ou sem possibilidade de recontagem não entram na métrica.

### Passos

1. Fixar o timestamp do snapshot e arquivar o comando de contagem no diretório da rodada.
2. Contar `tool_use` por session ID e associar cada session a uma task card.
3. Separar `batch`, MCP e polling; não chamar polling de trabalho de produto.
4. Contar writes e edits por path-alvo, excluindo cópias de baseline.
5. Comparar Before/After e versões para primeira passagem e retrabalho.
6. Ler o artefato final e classificar relatório divergente por claim, não por impressão.
7. Listar lições registradas e confirmadas com a ação e a sessão que confirmou a prevenção.
8. Não derivar qualidade de contagem de linhas, bytes ou tamanho do documento.

### Checklist de coleta pelo root

- [ ] Fixar timestamp do snapshot e arquivar o comando de contagem.
- [ ] Contar `tool_use` por session ID e associar cada session a uma task card.
- [ ] Separar `batch`, MCP e polling.
- [ ] Contar writes e edits por path-alvo, excluindo cópias de baseline.
- [ ] Comparar Before/After e versões para primeira passagem e retrabalho.
- [ ] Ler o artefato final e classificar relatório divergente por claim.
- [ ] Listar lições registradas e confirmadas com a ação e a sessão que confirmou a prevenção.
- [ ] Não derivar qualidade de contagem de linhas, bytes ou tamanho do documento.

### Benchmark de robustez e desempenho

`scripts/coordination_playbook_benchmark.py` mede o linter em corpus válidos escaláveis, sem alterar o documento fonte nem usar repetição para inflar casos. O procedimento é:

1. congelar baseline do linter e do corpus por hash;
2. usar tamanhos ordenados e parágrafos únicos, exigindo zero diagnósticos;
3. executar warm-up e rodadas com o mesmo interpretador, host e carga registrados;
4. reportar mediana, desvio, IQR, p95, mínimo/máximo e throughput, não apenas média;
5. medir pico com `tracemalloc` fora do loop e atribuir custo com `cProfile`;
6. comparar Before/After pelo mesmo corpus e registrar razões, sem declarar otimização sem evidência;
7. exportar JSON para análise e manter o benchmark fora do `doc-gate`.

O resultado é descritivo do ambiente e do hash medidos. Não é requisito de negócio, gate de release nem substituto de testes de correção.

### Saída

- Tabela de métricas acima, com timestamp e fontes indexadas no relatório de entrega da rodada.
- Registro de recontagem pelo root em log separado no diretório da rodada.

### Pós-condição

- Cada número tem fonte identificável e timestamp.
- Toolcalls, tempo, primeira passagem, retrabalho e lições são separados.
- Qualidade não é inferida de linhas, bytes ou tamanho do documento.
- Métricas de processo não são apresentadas como gate de release.

### Critério de parada e falha/escalada

- Parar se o journal não tiver IDs ou timestamps suficientes para atribuição. Reportar a métrica como indisponível, sem estimar.
- Escalar ao root quando a coleta exigir dados sensíveis, alterar a fonte ou usar transcript não autorizado.

---

## CP-25 — Checklists completos por papel e fase

### Checklist Cricket (Root) — Por fase

| Fase | Item | Critério |
|---|---|---|
| **Intake** | Missão preenchida | CP-08 completo, sem placeholders |
| | Lanes selecionadas | Algoritmo CP-04 aplicado, dono por lane |
| | Template swarm validado | Campos preenchidos e mecanismo único de início por lane |
| | Orçamento total | ≤ limite da sessão, alocado por lane |
| | Autorizações externas | Explícitas (ação/alvo/escopo) ou NENHUMA |
| **Execução** | Workers spawnados | Um mecanismo por lane, IDs copiados |
| | Acompanhamento | `swarm status`/`bg list` a cada ciclo |
| | Builds Rust | Dono único, `bg wait` mesmo ID |
| | Escaladas | Registradas em `fox-decisions.md` |
| **Integração** | Artefatos lidos | Fox confirmou diff semântico, exits, pós-condições |
| | Playbook integrado | CP-00..CP-28, sem placeholders, 5 papéis |
| | Gates executados | Linter, doc-gate e diff-check PASS; schema quando aplicável; WAIVED somente por governança |
| | Revisão semântica | Cricket aprova ou devolve com diagnóstico |
| **Commits** | Temáticos e pequenos | Uma mudança lógica, após gates |
| | Só arquivos da tarefa | Sem dados operacionais, mudanças alheias |
| | `git diff --check` | Limpo em cada commit |
| **Fechamento** | `swarm list`/`bg list` | Vazios |
| | `git status` | Limpo |
| | Lições | `SESSION_LEARNINGS.md` se nova/recorrente |
| | Relatório final | Arquivos, comandos, gates, commits, pendências |

### Checklist Fox (Subcoordenador) — Por fase

| Fase | Item | Critério |
|---|---|---|
| **Setup** | Missão validada | Checkout/HEAD/escopo/orçamento/autorizações OK |
| | Contratos de lane | DM para cada worker com CP-09 preenchido |
| **Acompanhamento** | IDs copiados | Completos, `swarm status`/`bg list` |
| | Progresso confirmado | Não apenas anúncio — ler artefato |
| | Silêncio > 15 min | Inspecionar antes de cancelar |
| **Integração** | Ler antes de integrar | Cada artefato final, diff semântico, exits |
| | Combinar drafts | Ordem canônica, sem duplicação, autoridades referenciadas |
| | Placeholders | Substituídos por conteúdo real |
| | `fox-decisions.md` | Conflitos/omissões listados para root |
| | `fox-delivery.json` | Schema Rat válido, dados reais |
| **Prechecks** | Linter (Ox) | Positivo + mutações negativas PASS nos insumos do Fox |
| | Schema (Rat) | Resultado registrado quando aplicável; indisponibilidade não vira PASS |
| | Preparação do doc-gate | Draft e links prontos; execução final pelo Cricket |
| | `git diff --check` | Limpo nos arquivos acessíveis ao Fox |
| **Entrega** | `playbook-integrated.md` | Completo para revisão root |
| | `fox-decisions.md` | Conflitos pendentes (ou vazio) |
| | `fox-delivery.json` | Schema Rat, arquivos/comandos/exits/diagnósticos/pendências |

### Checklist Lobster (Visual/Documental) — Por fase

| Fase | Item | Critério |
|---|---|---|
| **Setup** | Checkout/HEAD confirmado | `git rev-parse --show-toplevel` e `HEAD` |
| | Fontes textuais lidas | theme.rs, effects.rs, mod.rs, relatórios |
| **Execução** | Grounding textual | Referências arquivo:linha, sem placeholders |
| | Sem abertura de imagens | Apenas fontes textuais e schemas |
| | Limites aplicados | Uma imagem por vez, schema estrito, sem base64 |
| **Validação** | Schema visual válido | Se aplicável, `super-vl-visual-review.json` |
| | Controle textual | Whitespace/markup verificado |
| **Entrega** | `visual-documentation.md` | CP-12, CP-17, CP-19 completos |
| | `lobster-delivery.json` | Arquivos, comandos/exits, diagnósticos, pendências reais |

### Checklist Ox (Tooling/Determinístico) — Por fase

| Fase | Item | Critério |
|---|---|---|
| **Setup** | Checkout/HEAD confirmado | `git rev-parse --show-toplevel` e `HEAD` |
| | Documentos da rodada lidos | design-brief, session-matrix, preservation-map |
| **Execução** | Edição estreita | `apply_patch`/`edit`, não reescrita integral |
| | Baseline hash | Congelado antes de editar, verificado a cada iteração |
| | Loop determinístico | Diff vazio = N/A, não motivo para revisão |
| | Relatório completo | Comando, exit, path absoluto, contagens, warnings, pós-condição |
| **Linter** | Python stdlib | Determinístico, sem dependências externas |
| | Positivo passa | Playbook válido passa |
| | Negativas falham | Cada mutação falha pela regra esperada |
| | Diagnósticos | Arquivo/regra/linha |
| **Entrega** | `ox-tooling.md` | CP-05, CP-06, CP-13, CP-18, CP-21 |
| | `scripts/coordination_playbook_gate.py` | Linter funcional |
| | `tests/fixtures/coordination_playbook/**` | Controles positivo/negativo |
| | `ox-delivery.json` | Arquivos, comandos/exits, diagnósticos, pendências reais |

### Checklist Rat (Artefatos/Normalização) — Por fase

| Fase | Item | Critério |
|---|---|---|
| **Setup** | Checkout/HEAD confirmado | `git rev-parse --show-toplevel` e `HEAD` |
| | Documentos da rodada lidos | design-brief, session-matrix, preservation-map |
| **Execução** | Schema JSON lane | Mission, lane, report, decision, task card, handoff |
| | Formato artefatos | Before/After, logs absolutos, task cards, paths absolutos |
| | Normalização | Sem alterar conteúdo factual |
| | Vinculação logs | Cada artefato linka log/evidência |
| **Métricas** | Numéricas | Toolcalls, tempo, primeira passagem, retrabalho, lições |
| | Fonte rastreável | Logs da rodada |
| | Sem qualidade por linhas | Não mede por contagem |
| **Templates** | Copiáveis | Campos obrigatórios marcados, validação |
| **Entrega** | `validation-artifacts.md` | CP-14, CP-20, CP-24, CP-26 |
| | `lane-delivery-schema.json` | Schema + templates |
| | `rat-delivery.json` | Arquivos, comandos/exits, diagnósticos, pendências reais |

---

## CP-26 — Templates copiáveis: missão, lane, relatório, decisão, task card e handoff

### Gatilho

Início de uma coordenação com lanes, decisão em conflito ou passagem de ownership.

### Insumos

1. Missão, autoridade, escopo e pós-condições já definidos pelo coordenador.
2. Artefatos, logs e task cards das lanes envolvidos.
3. Formulário swarm derivado em `.jcode/swarm-prompt.md`.
4. Schema comum de entrega distribuído na rodada Rat.

### Uso do template swarm

Use `.jcode/swarm-prompt.md` quando a missão realmente exige swarm. Ele permanece o formulário preenchível de missão e lanes. Antes do spawn:

1. substitua todo campo entre chaves duplas pelo valor real;
2. confirme autoridade, escopo, fora de escopo, arquivos reservados, orçamento, pós-condições, gates e autorizações;
3. escolha exatamente um mecanismo de início por lane;
4. mantenha o checkout absoluto e o HEAD esperado no prompt;
5. confirme que cada lane tem artefato concreto, dono e escopo exclusivo.

### Passos

1. Copiar o bloco da categoria necessária e preencher todos os campos obrigatórios.
2. Resolver cada path para forma rastreável: relativo ao repositório para arquivo versionado, absoluto para artefato ou log fora do checkout.
3. Substituir marcadores de hash e tamanho pelos valores calculados do artefato final.
4. Registrar comando, exit, diagnóstico e log de cada verificação executada.
5. Declarar como `PENDING`, `BLOCKED` ou `PENDING-HUMAN` tudo que não puder ser observado.
6. Entregar o relatório como índice. O root relê o artefato, valida o diff e confere a pós-condição antes de integrar.

Os formulários abaixo estruturam a entrega de cada lane. Copie somente o bloco da categoria necessária. Todos os campos mostrados são obrigatórios. Remova campo opcional apenas quando não se aplicam e registre `N/A` com justificativa.

### Template de missão

```json
{
  "mission_id": "mission-001",
  "mission": "Resultado único e observável que a missão deve produzir.",
  "authority": [
    "AGENTS.md §15-17",
    "AGENTS.md §16",
    "CONVENTIONS.md"
  ],
  "scope": {
    "allowed": ["Arquivo ou fronteira de escrita permitida"],
    "forbidden": ["Ação explicitamente proibida"],
    "reserved_for_coordinator": ["Arquivo que somente o coordenador integra"]
  },
  "postconditions": ["Condição observável que o root pode recontar"],
  "risk": {
    "level": "low",
    "mitigations": ["Controle que reduz o risco"]
  },
  "budget": {
    "total_minutes": 45,
    "deadline": "<TIMESTAMP_ISO8601>",
    "method_limit_minutes": 15,
    "lane_limit_minutes": 45
  },
  "external_authorizations": [],
  "gates": [
    {
      "name": "Nome do gate aplicável",
      "owner": "root",
      "applicable": true
    }
  ]
}
```

### Template de lane

```json
{
  "lane_id": "lane-001",
  "owner": "Rat",
  "artifact": "Entrega concreta e arquivo final",
  "write_scope": ["docs/COORDINATION_PLAYBOOK.md"],
  "dependencies": [],
  "start_mechanism": {
    "kind": "explicit_spawn",
    "automatic_started": false,
    "explicit_spawned": true
  },
  "budget_minutes": 45,
  "acceptance": [
    "Pós-condição observável 1",
    "Pós-condição observável 2"
  ]
}
```

### Template de relatório

```json
{
  "report_id": "report-001",
  "lane_id": "lane-001",
  "state": "IMPLEMENTED",
  "before": {
    "captured_at": "<TIMESTAMP_ISO8601>",
    "checkout_root": "<CHECKOUT>",
    "head": "<HEAD_ATUAL>",
    "status_short": ["?? .scratch/arquivo-preexistente.txt"]
  },
  "after": {
    "captured_at": "<TIMESTAMP_ISO8601>",
    "checkout_root": "<CHECKOUT>",
    "head": "<HEAD_ATUAL>",
    "status_short": ["?? .scratch/arquivo-preexistente.txt"]
  },
  "artifacts": [
    {
      "path": "docs/SESSION_LEARNINGS.md",
      "kind": "markdown",
      "role": "Artefato final versionado",
      "sha256": "<SHA256_CALCULADO>",
      "bytes": 33843,
      "links": [
        {
          "path": "docs/SESSION_LEARNINGS.md",
          "kind": "artifact",
          "claim": "Claim sustentada pelo artefato final",
          "verification_status": "pending_root_read"
        }
      ]
    }
  ],
  "commands": [
    {
      "command": "sha256sum docs/SESSION_LEARNINGS.md",
      "cwd": "<CHECKOUT>",
      "exit_code": 0,
      "diagnostic": "sha256sum printed one digest line for the file",
      "postcondition": "Condição independente observada",
      "evidence": [
        {
          "path": "docs/SESSION_LEARNINGS.md",
          "kind": "artifact",
          "claim": "O artefato final existe no path declarado",
          "verification_status": "pending_root_read"
        }
      ]
    }
  ],
  "diagnostics": [],
  "postconditions": [
    {
      "condition": "Condição observável",
      "observed": true,
      "evidence": [
        {
          "path": "docs/SESSION_LEARNINGS.md",
          "kind": "artifact",
          "claim": "Artefato final contém a condição",
          "verification_status": "pending_root_read"
        }
      ]
    }
  ],
  "gates": [
    {
      "name": "Gate aplicável não executado pela lane",
      "state": "PENDING",
      "executed": false,
      "executed_by": "root",
      "evidence": []
    }
  ],
  "pending": ["Pendência: dependência ou ação concreta"],
  "blocked": [],
  "pending_human": [],
  "external_actions": []
}
```

### Template de decisão

```json
{
  "decision_id": "decision-001",
  "conflict": "Conflito factual, de autoridade ou escopo",
  "options": [
    {
      "option_id": "option-a",
      "summary": "Primeira opção",
      "consequences": ["Consequência verificável"]
    },
    {
      "option_id": "option-b",
      "summary": "Segunda opção",
      "consequences": ["Consequência verificável"]
    }
  ],
  "selected_option": "option-a",
  "rationale": "Justificativa pela autoridade aplicável",
  "owner": "Cricket",
  "due_at": "<TIMESTAMP_ISO8601>",
  "state": "PROPOSED",
  "authority": ["AGENTS.md §1"]
}
```

### Template de task card

```json
{
  "id": "task-001",
  "title": "Título factual",
  "description": "Descrição do trabalho observável",
  "owner": "Rat",
  "status": "active",
  "priority": "high",
  "source": "Requisito explícito do root para a rodada",
  "why_selected": "Motivo objetivo e limitado",
  "verifiability": "Como o root prova resultado e pós-condição",
  "risk": "low",
  "dependencies": [],
  "before": {
    "problem": "Estado anterior observado",
    "evidence": [
      {
        "path": "docs/SESSION_LEARNINGS.md",
        "kind": "source",
        "claim": "Baseline factual lida na lição 22, logs relativos/ausentes e relatórios sem artefato",
        "verification_status": "pending_root_read"
      }
    ]
  },
  "after": {
    "change": "Estado posterior factual",
    "files_changed": ["docs/COORDINATION_PLAYBOOK.md"],
    "evidence": [
      {
        "path": "docs/COORDINATION_PLAYBOOK.md",
        "kind": "artifact",
        "claim": "Artefato final",
        "verification_status": "pending_root_read"
      }
    ]
  },
  "validation": {
    "commands": ["sha256sum docs/COORDINATION_PLAYBOOK.md"],
    "result": "observed",
    "evidence": [
      {
        "path": "docs/COORDINATION_PLAYBOOK.md",
        "kind": "artifact",
        "claim": "Digest do artefato final calculado",
        "verification_status": "pending_root_read"
      }
    ]
  },
  "followups": ["Próxima ação segura"],
  "updated_at": "<TIMESTAMP_ISO8601>"
}
```

### Template de handoff

```json
{
  "handoff_id": "handoff-001",
  "source": "Rat",
  "destination": "Fox",
  "inputs": [
    {
      "path": "docs/SESSION_LEARNINGS.md",
      "kind": "source",
      "claim": "Insumo rastreável lido na lição 22, logs relativos/ausentes e relatórios sem artefato",
      "verification_status": "pending_root_read"
    }
  ],
  "output": [
    {
      "path": "docs/SESSION_LEARNINGS.md",
      "kind": "markdown",
      "role": "Entrega transferida",
      "sha256": "<SHA256_CALCULADO>",
      "bytes": 33843
    }
  ],
  "state": "READY",
  "acceptance": [
    "Receptor lê o artefato final",
    "Receptor compara relatório, diff e pós-condição"
  ],
  "next_owner": "Fox",
  "next_action": "Revisar e integrar, sem declarar aceite"
}
```

### Validação de preenchimento

O root e o linter devem rejeitar:

- campos entre chaves duplas não resolvidos, marcadores de trabalho pendente ou campos obrigatórios vazios;
- path relativo de artefato de rodada ou log;
- hash de 64 zeros ou bytes zero para arquivo que deve existir;
- gate `PASS` com `executed: false`;
- decisão com menos de duas opções;
- lane com início automático e spawn explícito simultâneos;
- task card sem ID, owner, dependências ou estado;
- handoff sem próximo dono e próxima ação;
- `verification_status` diferente de `pending_root_read` sem `root_note` quando emitido pelo root.

### Saída

- Formulários acima, publicados no playbook, e o schema comum de entrega distribuído como artefato da rodada Rat.

### Pós-condição

- Templates copiáveis, sem placeholders operacionais.
- Campos obrigatórios identificáveis pelo schema.
- Validação de preenchimento executável pelo root.
- Template swarm continua mantido e referenciado, não duplicado como autoridade.

### Critério de parada e falha/escalada

- Parar se um template exigir valor que só uma decisão de root pode fornecer. Registrar `PENDING-HUMAN` ou `BLOCKED`, conforme o bloqueador.
- Escalar conflito de prioridade ou autoridade ao Cricket. O template não autoriza ação externa.

---

## CP-27 — Matriz erro → prevenção → controle → evidência, com os casos das sessões

Casos sanitizados observados nas sessões. As fontes duráveis são `docs/SESSION_LEARNINGS.md`, `docs/SESSION_IMPROVEMENT.md`, `docs/EVIDENCE_REGISTER.md` e as skills locais. Logs detalhados permanecem fora do documento canônico e são localizados pelo relatório de cada rodada.

| # | Erro observado | Prevenção | Controle executável | Fonte durável |
|---|---|---|---|---|
| E1 | Sequência mecânica serial sem hipótese | Toda tentativa tem hipótese, método e pós-condição; no máximo duas sem informação nova | Checklist de intake e revisão do motivo da chamada | `docs/SESSION_LEARNINGS.md` |
| E2 | `todo update` com nomes inexistentes | Ler nomes atuais antes de atualizar; usar `rename_to` para preservar identidade e dependências | Ler o todo e confirmar o estado retornado | `docs/SESSION_LEARNINGS.md` lição 28 |
| E3 | Path resumido que não existe | Localizar o artefato e ler o arquivo real antes de citá-lo | Conferir existência e conteúdo do log, não copiar o resumo | `.jcode/skills/dfir-verify/SKILL.md` |
| E4 | Controle negativo com substring incompleta | Verificar regra, diagnóstico completo e exit no log original | Exigir a regra esperada e a falha pela condição mutada | `.jcode/skills/dfir-verify/SKILL.md` |
| E5 | Patch especulativo sem defeito reproduzido | Reproduzir comportamento, identificar contrato violado e escolher a menor mudança | Before/After e controle negativo antes do patch | `.jcode/skills/dfir-change/SKILL.md` |
| E6 | Afirmação de linha/API sem conferência | Confirmar fonte e versão atual antes de afirmar assinatura ou comportamento | Citação arquivo:linha e leitura da versão do lock quando aplicável | `.jcode/skills/dfir-change/SKILL.md` |
| E7 | Relatório com HEAD antigo, arquivos omitidos ou contagens divergentes | Recalcular HEAD, status, arquivos e contagens no fechamento | Comparar relatório ao `git status`, aos logs e ao diff final | `docs/SESSION_IMPROVEMENT.md` |
| E8 | Reescrita integral para edição local e baseline ausente | Congelar baseline por hash e usar a menor edição suficiente | Hash/diff por iteração e leitura do arquivo final | `.jcode/skills/dfir-change/SKILL.md` |
| E9 | Relatório cita stage/commit sem o arquivo versionado | Confirmar status e diff staged antes do commit | `git status --short` e `git diff --cached --name-only` | `docs/SESSION_IMPROVEMENT.md` |
| E10 | Markup ou pipe indevido persiste após anúncio de correção | Aceitar somente arquivo e diff reais, não anúncio | Controle textual de markup, pipes, whitespace e newline | `.jcode/skills/dfir-verify/SKILL.md` |
| E11 | Quadro estático tratado como prova de animação ou estética | Limitar a afirmação ao quadro/estado observado | `uncertain` ou `PENDING-HUMAN` quando a inspeção necessária não ocorreu | `AGENTS.md` §17 |
| E12 | Normalização apresentada como execução independente | Separar índice de execução e manter links recontáveis | Gate com `executed: false` não recebe PASS | `.jcode/skills/dfir-verify/SKILL.md` |
| E13 | Capturas de UI entram no contexto automático | Manter capturas locais; usar janela sintética, worker isolado e uma imagem por vez quando autorizado | Relatório textual sem imagem ou base64 | `AGENTS.md` §17 |
| E14 | Trabalho Rust pesado sem dono único ou com ID errado | Um dono executa sequencialmente e espera o mesmo ID de background | `bg wait` no ID do comando, nunca no ID de `await_members` | `docs/EVIDENCE_REGISTER.md` EVD-055/EVD-056/EVD-057 |
| E15 | Log de validação em caminho relativo | Gravar log absoluto fora do checkout e lê-lo depois | Conferir path, exit, diagnóstico e pós-condição | `docs/EVIDENCE_REGISTER.md` |
| E16 | Primeiro `Id` animado retorna `1.0` sem seed `false` | Semear `false` antes do primeiro `true`; cobrir `clear()` e reuso | Teste de regressão com `step_dt=1/60` | `docs/SESSION_LEARNINGS.md` lição 29 |
| E17 | `step_dt` padrão maior que a transição | Configurar harness com passo menor e limite de passos | Assertivas de progresso limitado e não decrescente | `docs/SESSION_LEARNINGS.md` lição 29 |
| E18 | Mesma tentativa repetida sem informação nova | Mudar de método ou consolidar após duas tentativas | Registrar hipótese nova ou encerrar a investigação | `docs/SESSION_IMPROVEMENT.md` §2 |
| E19 | Placeholders de entrega no documento final | Usar rótulos copiáveis somente em templates e preencher a instância real | Regras `placeholder_field` e `placeholder_token` do linter | `scripts/coordination_playbook_gate.py` |
| E20 | Worker decide risco crítico ou ação externa | Escalar ao root; ação externa exige autorização com ação, alvo e escopo | Registro da decisão e das autorizações na missão | `AGENTS.md` §17 e CP-16 |
| E21 | Fallback ou patch iniciado a partir de gate antigo | Reproduzir o estado atual e controlar variáveis de ambiente antes de corrigir | Repetir o teste com scratch explícito, log e hash dos insumos | `docs/SESSION_IMPROVEMENT.md` §2 |
| E22 | Texto anotado por `read` tratado como fonte bruta por automação | Remover numeração de apresentação e validar um controle positivo antes de montar | Falhar fechado se headings obrigatórios não forem extraídos; reler o arquivo escrito | `AGENTS.md` §15 e CP-06 |
| E23 | Heading CP ou de papel em fence tratado como estrutura | Mascarar fences CommonMark preservando offsets; manter placeholders globais por decisão explícita | `test_adversarial.py` cobre backtick, til, fechamento longo, EOF e CRLF | `docs/SESSION_LEARNINGS.md` lição 30 e `scripts/coordination_playbook_gate.py` |
| E24 | `FormatChecker` ativo sem backend do formato | Provar formato válido e inválido antes do aceite; registrar backend e versão isolados | Controles `date-time` válido/inválido e 12 mutações de contratos detectadas | `docs/SESSION_LEARNINGS.md` lição 32 e CP-14 |
| E25 | Artefato declarado ausente por lookup ou diretório incorreto | Enumerar o diretório canônico, fazer controle positivo e registrar stat/hash antes de alegar ausência | Inventário dos quatro relatórios com hashes e validação root independente | `docs/SESSION_LEARNINGS.md` lição 33 e CP-14 |
| E26 | Auditoria ad hoc mistura playbook, lições e `justfile` | Separar contextos e cobrir repositório real com falsos positivos e negativos | `scripts/coordination_playbook_repo_audit.py` e `test_repo_audit.py` | `scripts/coordination_playbook_repo_audit.py` e CP-21 |
| E27 | `assertIs` entre módulos recarregados por suítes distintas | Assertar registro e contrato `validate()`, não identidade de objeto entre imports | Suíte combinada no mesmo processo | `docs/SESSION_LEARNINGS.md` lição 31 e `tests/fixtures/coordination_playbook/test_adversarial.py` |

## Como usar a matriz

1. Escolher a linha cuja causa coincide com o comportamento observado, não apenas com o sintoma.
2. Executar o controle mais barato que possa refutar a hipótese.
3. Se a prevenção falhar novamente, registrar evidência sanitizada e promover um controle executável na autoridade apropriada.
4. Não criar nova lição permanente quando houver somente confirmação de uma lição existente; registrar a confirmação no arquivo vivo conforme `docs/SESSION_IMPROVEMENT.md`.

---

## CP-28 — Definition of Done, pendências honestas e revisão periódica

### Nível Lane: pronto para revisão

- [ ] Artefato escrito somente no escopo exclusivo da lane.
- [ ] Testes e prechecks contratados executados, com comandos, exits e diagnósticos.
- [ ] Relatório estruturado entregue ou lacuna de relatório registrada explicitamente; o root pode verificar o artefato sem transformar a ausência do índice em waiver.
- [ ] Nenhum `PASS`, `VERIFIED` ou `ACCEPTED` além da evidência.
- [ ] Pendências têm motivo, dono e próxima ação.
- [ ] Todo da lane atualizado e nenhum processo próprio deixado sem dono.

### Nível Fox: integração editorial pronta para o root

- [ ] Todos os blocos exigidos estão presentes uma vez e na ordem definida.
- [ ] Cinco papéis têm fluxo, checklist e limites coerentes com CP-02.
- [ ] Fontes duráveis são versionadas; dados transitórios ficam nos relatórios da rodada.
- [ ] Conflitos e omissões estão registrados para decisão root.
- [ ] Prechecks têm logs e não são apresentados como gates finais.
- [ ] Nenhum arquivo reservado ao root foi alterado pelo Fox.

### Nível Cricket: integração e verificação

- [ ] Arquivo final e diff semântico foram relidos pelo root.
- [ ] Linter do canônico, fixture positiva e todas as mutações negativas passaram no estado atual.
- [ ] Benchmark e testes de robustez aplicáveis foram revisados sem usá-los como medida de qualidade textual.
- [ ] Schema foi validado quando exigido e havia validador; indisponibilidade permanece PENDING.
- [ ] `just doc-gate` PASS com o linter ligado uma única vez.
- [ ] `git diff --check` limpo e nenhum conteúdo sensível ou arquivo alheio foi incluído.
- [ ] Pendências não executadas estão explícitas e não foram convertidas em waiver.

### Nível Cricket: commit e fechamento

- [ ] Commits pequenos e temáticos contêm somente arquivos da tarefa.
- [ ] `git status --short` foi revisado; somente mudanças alheias conhecidas e preservadas podem permanecer.
- [ ] `swarm list` e `bg list` foram verificados; workers e processos próprios foram encerrados ou têm dono explícito.
- [ ] `docs/SESSION_LEARNINGS.md` foi atualizado somente se houve lição nova, recorrência ou confirmação útil; caso contrário consta "sem nova lição de repositório".
- [ ] Relatório final lista artefatos, pós-condições, comandos/exits, diagnósticos, gates, commits, pendências e ações externas.
- [ ] Não houve push, publicação ou outra ação externa sem autorização explícita separada.
- [ ] `ACCEPTED` só é usado com evidência vinculada à tree/commit correspondente.

### Pendências honestas

| Tipo | Uso | Exemplo genérico |
|---|---|---|
| `PENDING` | Dependência ou execução ainda não realizada | "Validação de schema pendente: validador indisponível" |
| `BLOCKED` | Risco, conflito ou autorização impede prosseguir | "Integração bloqueada: controle negativo falhou" |
| `PENDING-HUMAN` | Julgamento humano necessário e não substituível | "Aceite estético não demonstrado" |
| `STALE` | Insumo da evidência mudou | "Playbook alterado após o último linter" |
| `WAIVED` | Exceção de governança autorizada, não verificação | "Waiver existente citado com ID e escopo" |

Toda pendência inclui o porquê, o artefato afetado, o dono e a próxima ação. Ausência de relatório não vira `WAIVED`, e limite de tempo não vira sucesso.

### Revisão periódica do playbook

Revisar somente quando houver mudança de autoridade, nova classe de erro, alteração de ferramenta/workflow, lição recorrente confirmada ou preparação de release candidato.

1. Cricket define o gatilho e o escopo da revisão.
2. Fox consolida as mudanças necessárias e as fontes proprietárias.
3. Especialistas atualizam somente seus blocos e controles.
4. Ox repete linter, auditoria de repositório, controles negativos, propriedades e benchmark; Rat atualiza artefatos; Lobster participa apenas se houver conteúdo visual/documental afetado.
5. Cricket faz integração final, gates e commit.
6. Registrar lições novas ou confirmações em `docs/SESSION_LEARNINGS.md`, sem retrospectiva permanente de rotina.

Não alterar especificação, ADRs, RTM, template swarm ou skills por deduplicação. O playbook permanece derivado e referencia essas autoridades.
