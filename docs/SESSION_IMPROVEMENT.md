<!-- doc-type: procedure | status: current -->
# Autoaprimoramento entre sessões

Procedimento operacional obrigatório para agentes, subordinado a `AGENTS.md`. Não altera requisitos, arquitetura, estados de tickets nem critérios de release. O registro de fatos reutilizáveis fica em `docs/SESSION_LEARNINGS.md`.

## Loop obrigatório

```text
LER -> APLICAR -> OBSERVAR -> CORRIGIR -> VERIFICAR -> REGISTRAR
 ^                                                       |
 +------------- próxima sessão confirma -----------------+
```

Toda sessão executa o loop. Uma sessão somente de leitura faz a revisão em memória e não edita o repositório. Uma sessão que altera arquivos ou executa gates deixa evidência do fechamento nos comandos e na resposta. O registro permanente só muda quando houver lição reutilizável nova, recorrência ou mudança de pendência; não se cria relatório por rotina.

## 1. Início da sessão

1. Execute `git status --short` e identifique HEAD e mudanças preexistentes antes de editar.
2. Leia `AGENTS.md`, este procedimento e as entradas recentes de `docs/SESSION_LEARNINGS.md` aplicáveis à tarefa.
3. Converta cada lição aplicável em uma ação concreta do plano ou em um motivo explícito de não aplicabilidade. Reler sem mudar comportamento não fecha o loop.
4. Defina escopo, pós-condições, orçamento de investigação e critérios de parada antes de workers ou comandos longos.
5. Reserve futuros IDs de evidência somente em scratch. Consulte `docs/EVIDENCE_REGISTER.md`; não cite um ID como evidência vigente antes que arquivo e registro existam.
6. Verifique quais evidências anteriores podem estar `STALE` por mudança de código, lock, toolchain, dataset ou banco de advisories. Idade sozinha não prova validade nem invalidade.

## 2. Durante a execução

- Registre em scratch fatos necessários para o fechamento: comando, exit, diagnóstico, hash e pós-condição. Não copie secrets, nomes de evidências ou conteúdo de eventos.
- Um exit 0 prova apenas o término do comando. Confirme a pós-condição independente e leia stderr/logs. Clique enviado não prova navegação; resize enviado não prova geometria válida; helper sem erro de shell pode ter emitido erro de protocolo.
- Uma busca vazia inesperada não prova ausência. Faça um controle positivo conhecido ou use leitura/diff alternativo antes de concluir.
- Duas tentativas da mesma técnica sem nova informação exigem mudança de hipótese ou parada. Uma terceira tentativa precisa ter instrumentação ou pressuposto novo, não ser repetição com outro nome.
- Como padrão, limite a 15 minutos um método sem novo estado observável e a 30 minutos uma lane exploratória sem resultado útil. Ao atingir o limite, consolide `BLOCKED`/`PENDING-HUMAN` ou obtenha decisão explícita para um novo orçamento. Não deixe worker aberto enquanto o coordenador apenas espera.
- Compilação ou teste longo conhecido não é investigação improdutiva. Antes de cancelar por silêncio, confira progresso, processo e consumo; trabalho Rust pesado continua sequencial neste host.
- Use IDs completos retornados por `bg`, `swarm` e `todo.waiting_on`. Liste membros antes de parar workers; não invente prefixos nem suponha que um worker ausente ainda esteja ativo.
- Não antecipe `PASS`, `VERIFIED` ou aceite em documentação viva. Primeiro execute e registre a evidência; depois atualize a referência vigente e repita o gate documental.
- Preserve narrativa histórica e detalhes fora da tarefa. Em RTM, ADRs e evidências, prefira acréscimos cirúrgicos e revise o diff para detectar perdas acidentais.

## 3. Fechamento da sessão

1. Compare plano e resultado: comandos/exits, arquivos, hashes, testes, pendências e o que não foi executado.
2. Revise `git diff --check`, o diff semântico e `git status --short`. Encerre workers e tarefas próprios; não deixe processos de GUI, engine ou observers sem dono.
3. Separe:
   - **erro de execução:** ação ou omissão da sessão;
   - **risco herdado:** condição anterior revelada pela sessão;
   - **limitação:** algo não demonstrado, sem convertê-lo em defeito do produto.
4. Para cada lição reutilizável nova, implemente a menor prevenção eficaz: instrução, checklist, teste, lint ou gate. Uma frase genérica como "ter mais cuidado" não é prevenção.
5. Atualize `docs/SESSION_LEARNINGS.md` com o formato abaixo somente se houver conteúdo novo. Se não houver, responda `sem nova lição de repositório` e não faça commit de rotina.
6. Se a mesma falha ocorrer novamente, marque a lição como `recorrente` e promova a prevenção para o controle executável mais barato que possa detectá-la. Não resolva recorrência apenas repetindo texto.
7. Na próxima sessão aplicável, confirme a prevenção com uma ação observada. Só então marque `confirmada`; passagem de tempo ou ausência de reclamação não confirma aprendizado.

## Formato mínimo de uma lição

```text
Data e escopo
Classe: erro de execução | risco herdado | limitação
Fato observado e evidência sanitizada
Causa ou mecanismo, sem especular além do observado
Prevenção implementada e local da regra/controle
Confirmação: pendente | confirmada | recorrente, com sessão/comando
```

## Antipadrões

`scripts/doc_gate.py` verifica a presença dos dois arquivos vivos e suas ligações em `AGENTS.md`, `CLAUDE.md`, `CONVENTIONS.md`, `PROMPT_NOVA_CONVERSA.md` e `docs/DOCUMENT_AUTHORITY.md`. Esse controle impede remoção/desconexão silenciosa, mas não prova que uma sessão aplicou a regra; essa confirmação continua exigindo ação observada.

- Criar uma retrospectiva permanente para toda rotina ou copiar toda a narrativa da conversa.
- Guardar a lição apenas em scratch, onde a próxima sessão não a encontrará.
- Transformar a lição em requisito de produto, ADR ou escopo sem decisão própria.
- Reescrever o passado para parecer que o erro não ocorreu.
- Declarar o loop concluído porque uma regra foi escrita, sem verificar sua aplicação posterior.
