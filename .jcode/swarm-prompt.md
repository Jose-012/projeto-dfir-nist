# Template de prompt para swarm no DFIR/NIST

> **Natureza derivada:** este modelo operacionaliza `AGENTS.md` §16/§17 e `CONVENTIONS.md`. Ele não altera requisitos, RTM, ADRs, arquitetura, escopo ou estados de release e não autoriza publicação, push ou qualquer outra ação externa por si só. Em caso de conflito, prevalece a hierarquia documental do repositório.

## Como preencher

Substitua todos os campos `{{...}}` antes de iniciar workers. Remova as orientações que não sejam necessárias ao prompt final, mas preserve os limites obrigatórios. Não preencha com dados de caso, secrets, nomes de evidências, conteúdo de eventos ou caminhos sensíveis. Se não houver autorização externa, escreva `NENHUMA`.

- **Missão:** {{MISSAO_E_RESULTADO_ESPERADO}}
- **Requisitos, ADRs, contratos e tickets aplicáveis:** {{REFERENCIAS_DE_AUTORIDADE}}
- **Escopo permitido:** {{ESCOPO}}
- **Fora de escopo:** {{FORA_DE_ESCOPO}}
- **Arquivos reservados à integração pelo coordenador:** {{ARQUIVOS_DE_INTEGRACAO}}
- **Orçamento total e horário limite:** {{ORCAMENTO_TOTAL}}
- **Pós-condições globais:** {{POS_CONDICOES_GLOBAIS}}
- **Gates e verificações aplicáveis:** {{GATES_E_VERIFICACOES}}
- **Modelos efetivos por role e escopo da fotografia:** {{COORDENADOR_SWARM_REVIEW_JUDGE_MEMORY_AMBIENT}}
- **Política de commits desta tarefa:** {{QUEM_COMMITA_E_QUANDO}}
- **Autorizações externas explícitas, com ação, alvo e escopo:** {{AUTORIZACOES_EXTERNAS_OU_NENHUMA}}

## Lanes autorizadas

Cada lane precisa produzir um artefato concreto e ter escopo exclusivo de escrita. Não crie lanes apenas de leitura, investigação ou revisão sem entrega. Escopos que se sobrepõem exigem redivisão antes do início. Dependências devem citar lanes anteriores, e o aceite deve conter pós-condições observáveis, não apenas comandos.

| Lane e dono | Artefato e entrega | Escopo exclusivo de escrita | Dependências | Orçamento | Aceite e pós-condições | Início, escolha somente um |
|---|---|---|---|---|---|---|
| {{LANE_1_E_DONO}} | {{ARTEFATO_1}} | {{ARQUIVOS_OU_FRONTEIRAS_1}} | {{DEPENDENCIAS_1_OU_NENHUMA}} | {{ORCAMENTO_1}} | {{ACEITE_1}} | {{AUTOMATICO_OU_SPAWN_1}} |
| {{LANE_2_E_DONO}} | {{ARTEFATO_2}} | {{ARQUIVOS_OU_FRONTEIRAS_2}} | {{DEPENDENCIAS_2_OU_NENHUMA}} | {{ORCAMENTO_2}} | {{ACEITE_2}} | {{AUTOMATICO_OU_SPAWN_2}} |

## Prompt operacional ao coordenador

### 1. Autoridade e abertura

1. Leia `AGENTS.md`, `CONVENTIONS.md`, `docs/DOCUMENT_AUTHORITY.md`, `docs/SESSION_IMPROVEMENT.md` e as lições aplicáveis em `docs/SESSION_LEARNINGS.md`. Acrescente somente os documentos exigidos pela missão.
2. Respeite esta precedência: demanda original; corpo normativo da especificação; decisões, ADRs, RTM, status e contratos; documentos derivados, índices e prompts; hipóteses, exemplos e sugestões. Não transforme inferência ou sugestão em requisito.
3. Registre HEAD, branch e `git status --short` no início, sem fixar esses valores neste template. Preserve mudanças preexistentes e arquivos fora da tarefa. Inclua no prompt de cada worker o caminho absoluto do checkout; `working_dir`, sozinho, não comprova o diretório efetivo da sessão.
4. Converta lições aplicáveis em ações observáveis do todo. Defina entrega, dono, escopo, pós-condições, orçamento e critérios de parada antes de workers ou comandos longos.
5. Mudanças em requisitos, RTM ou ADRs só podem ocorrer quando fizerem parte da missão e seguirem o processo de autoridade. Este prompt, por si só, não promove essas mudanças nem cria aceite.
6. Preserve a interface nativa, o engine separado com stdin/stdout + JSONL, as fronteiras de segurança e as classes de runtime. Não introduza dependência, serviço, transporte ou arquitetura nova sem a decisão e a validação exigidas.

### 2. Formação e coordenação

1. Use swarm somente para trabalho paralelo com entregas concretas e fronteiras independentes. Para tarefa trivial ou sem independência, execute diretamente sem workers.
2. Registre cada lane no todo antes do início. A lane é a fonte de verdade das tarefas, e o prompt do worker apenas acrescenta contexto compatível.
3. **Nunca combine `lane_owner = "worker"` ou `owner = "worker"` com `swarm spawn` na mesma lane.** Escolha início automático ou spawn explícito. Se usar spawn explícito, não configure início automático. Confirme o estado retornado para evitar workers duplicados.
4. Respeite dependências entre lanes. O coordenador mantém a autoridade, a integração e o resultado final, continua trabalhando enquanto houver trabalho próprio e aguarda os membros necessários antes de encerrar.
5. Copie IDs completos de workers e tarefas. IDs de `await_members` pertencem somente a `todo.waiting_on` e ao acompanhamento de swarm, nunca a `bg wait`. Acompanhe progresso, leia resultados antes de parar workers e confirme o resultado efetivo de cada parada. Não delegue ações destrutivas, irreversíveis ou externas.
6. Antes do spawn, registre uma fotografia dos modelos por role com `/agents`, `/subagent-model` e `swarm list_models` quando disponíveis. Se o runtime não estiver acessível, leia somente o sidecar `<id>.agent-models.meta` da sessão e as chaves de configuração pertinentes, sem transcript ou credenciais. A precedência é sessão, depois global; o fallback sem pin depende do role. Não confunda default global com override de sessão nem o modelo de review/judge/memory/ambient com o de swarm.
7. Com `agents.swarm_model_selection=false` ou allowlist vazia, o schema remove `model`: omita esse parâmetro e não contorne a política. Omita também `spawn_mode`, salvo pedido explícito do usuário. Modelos e modos efetivos são dados operacionais da missão, não requisitos nem autorização para trocar provedor.

### 3. Contrato dos workers

Cada worker deve:

1. Ler as instruções e skills aplicáveis e trabalhar somente na lane recebida. Antes de editar, executar `git rev-parse --show-toplevel` e `git rev-parse HEAD` no checkout absoluto informado pelo coordenador; se o diretório não corresponder, pedir correção por DM sem procurar ou criar outra cópia e sem duplicar worker.
2. Escrever exclusivamente nos arquivos ou fronteiras da tabela. Não editar arquivos do coordenador, de outra lane ou fora da tarefa. Solicitar redivisão ao coordenador se a entrega exigir ultrapassar esse limite.
3. Atualizar o todo da própria lane, preservar mudanças alheias e fazer a menor alteração suficiente.
4. Não iniciar trabalho Rust pesado sem a vez concedida pelo coordenador e não executar ações externas sem autorização específica.
5. Entregar artefato, testes ou decisão útil dentro do orçamento. Relatar arquivos, comandos, exits, diagnósticos, pós-condições observadas e pendências, sem declarar `PASS` ou `VERIFIED` além da evidência.
6. Fazer commit somente se a política da tarefa delegar essa ação à lane. Caso contrário, deixar o diff para revisão e commit do coordenador. Nunca fazer push ou publicar por iniciativa própria.

### 4. Skills e autoaprimoramento

1. Em toda sessão que alterar arquivos ou executar gates, carregue `.jcode/skills/dfir-improve/SKILL.md` e execute seu loop de abertura, feedback, integração e fechamento. Se o carregamento falhar ou o arquivo ainda não existir, o loop de `AGENTS.md` §16 e `docs/SESSION_IMPROVEMENT.md` continua obrigatório.
2. Carregue `.jcode/skills/dfir-change/SKILL.md` ao alterar código, contratos ou documentação.
3. Carregue `.jcode/skills/dfir-verify/SKILL.md` antes de declarar integração ou verificação.
4. Para incluir, alterar ou remover regras `EVTX_DETECTION`, carregue `.jcode/skills/dfir-detect/SKILL.md` em conjunto com `dfir-change` e `dfir-verify`.
5. Não carregue skills por hábito. Skills globais exigem utilidade específica. Não instale skills, pacotes, ferramentas ou catálogos implicitamente.
6. Skill carregada não é evidência de gate, aprendizado ou aceite. IDs futuros de evidência ficam em scratch até que execução, arquivo e registro existam.

### 5. Proteção de dados e ações externas

1. Não compartilhe com workers, MCPs, web ou outros serviços dados de caso, secrets, tokens, senhas, nomes de evidências, conteúdo de eventos ou caminhos sensíveis. Use apenas fixtures sanitizadas e identificadores lógicos necessários e autorizados.
2. Não abra dados operacionais para contornar uma limitação de teste. Logs e diagnósticos devem seguir as restrições de `AGENTS.md` §14.
3. Push, publicação, abertura ou alteração de PRs, comentários, dispatch de workflows, envios e qualquer outra ação externa exigem autorização explícita com ação, alvo e escopo em `{{AUTORIZACOES_EXTERNAS_OU_NENHUMA}}`. Autorização de leitura não autoriza escrita, e uma ação autorizada não libera as demais.
4. Para interações com GitHub, use o comando `gh`. O transporte Git de publicação usa `git push`, também somente com autorização explícita. Quando essa publicação for autorizada, autentique o transporte com `gh auth git-credential`, sem exibir tokens nem gravá-los no repositório. Não use MCP ou web para contornar essa exigência.
5. Resultado externo não substitui gates locais nem prova o HEAD atual sem vínculo com commit e tree.

### 6. Execução, orçamento e parada

1. Builds e testes Rust pesados permanecem sequenciais e coordenados pelo root. Não execute compilações pesadas concorrentes. Silêncio de build exige inspeção de progresso, processo e consumo antes de qualquer cancelamento.
2. Respeite o menor orçamento entre o total, a lane e o método. Como limite padrão, consolide após 15 minutos sem novo estado observável em um método ou 30 minutos em uma lane exploratória sem resultado útil, salvo orçamento mais restrito na tabela.
3. Após duas tentativas da mesma técnica sem informação nova, mude de método ou pare. Uma nova tentativa exige instrumentação ou pressuposto novo. Busca vazia inesperada exige controle positivo ou método alternativo.
4. Exit 0 prova apenas o término do comando. Sempre confirme a pós-condição independente e leia os diagnósticos.
5. Pare e consolide quando o aceite estiver atendido e revisto, quando o orçamento terminar, quando houver dependência ou autorização ausente, conflito de autoridade, risco a dados ou falta de progresso pelos critérios acima. Registre `BLOCKED`, `PENDING` ou `PENDING-HUMAN` conforme o caso, sem converter limite de tempo em sucesso.
6. Não deixe worker aberto enquanto o coordenador apenas espera. Consolide o resultado parcial útil e informe exatamente o que ficou pendente.

### 7. Integração, gates e commits

1. Aguarde os workers necessários e revise cada entrega, diff semântico e pós-condição antes de integrar. Confirme que somente arquivos da tarefa mudaram e que correções concorrentes foram preservadas. Antes de assumir a escrita de um worker parado, confirme seu estado terminal e releia o artefato imediatamente antes de editar. Trate uma escrita atrasada como nova entrega concorrente a revisar, nunca como ruído a sobrescrever.
2. Execute `git diff --check`, revise `git status --short` e rode os gates aplicáveis definidos pela autoridade. Como seleção usual: `just doc-gate` para documentação/contratos, `just verify` para código Rust, `just supply-chain` para dependências e o E2E com fixture sanitizada adequada quando o fluxo real do engine for afetado.
3. Documentação não herda gates de código, e código não é verificado apenas pelo doc-gate. Ferramenta ausente ou gate não executado deve ser relatado como pendente ou bloqueado. Não desabilite lint, testes ou controles para obter PASS.
4. Invalide evidências que se tornaram `STALE`. Não antecipe `PASS`, `VERIFIED` ou `ACCEPTED`, não confunda `WAIVED` com verificação e vincule evidências ao commit/tree correspondente.
5. Faça commits pequenos, temáticos e somente após os gates aplicáveis, contendo exclusivamente arquivos da tarefa. Não inclua dados de caso, exports, bancos operacionais, artefatos derivados ou mudanças alheias. O coordenador commita por padrão, salvo delegação explícita e compatível na política preenchida.
6. Não faça push nem publicação como consequência automática do commit ou deste prompt.

### 8. Fechamento obrigatório

1. Compare missão, tabela de lanes, todo e resultado. Revise comandos, exits, diagnósticos, arquivos, hashes quando pertinentes, gates e pendências.
2. Execute `swarm list` e `bg list`, aguarde ou encerre corretamente tarefas próprias e confirme que não restaram workers, processos de GUI, engine ou observers sem dono.
3. Termine com revisão semântica do diff, `git diff --check` e `git status --short`. Separe erro de execução, risco herdado e limitação.
4. Atualize `docs/SESSION_LEARNINGS.md` somente se houver lição nova, recorrência ou confirmação observada e somente quando esse arquivo estiver no escopo autorizado. Não crie retrospectiva permanente de rotina.
5. Relate artefatos, commits, comandos e resultados reais, seguidos do que não foi executado, das pendências e de qualquer ação externa autorizada. Se não houver conteúdo novo ou confirmação aplicável, registre `sem nova lição de repositório`.

Formato mínimo do relatório final:

```text
Artefatos e arquivos: {{...}}
Pós-condições confirmadas: {{...}}
Comandos, exits e diagnósticos: {{...}}
Gates executados e resultado: {{...}}
Commits: {{...}}
Ações externas autorizadas e realizadas: {{...}}
Pendências e não executados: {{...}}
Estado final de swarm/bg: {{...}}
```
