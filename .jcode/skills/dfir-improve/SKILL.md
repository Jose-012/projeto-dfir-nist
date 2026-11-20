---
name: dfir-improve
description: Executa o loop obrigatório em sessões DFIR/NIST que alteram arquivos ou rodam gates, com lições, orçamento, seleção de skills e fechamento verificável
allowed-tools: agentgrep, read, edit, apply_patch, write, bash, todo, swarm, bg
---

# Loop de autoaprimoramento DFIR/NIST

Use esta skill no início de uma sessão que alterará arquivos ou executará gates e retome o fechamento antes da resposta final. Para uma sessão somente de leitura e sem gates, faça a revisão em memória prevista em `docs/SESSION_IMPROVEMENT.md`, sem carregar skills por hábito.

Esta skill é um checklist derivado. `AGENTS.md` §16/§17 e `docs/SESSION_IMPROVEMENT.md` continuam sendo a autoridade operacional. Se o carregamento da skill falhar, o loop continua obrigatório. Carregar ou concluir este checklist não prova aprendizado, verificação ou aceite.

## 1. Abrir a sessão

1. Leia `AGENTS.md`, `docs/DOCUMENT_AUTHORITY.md`, `docs/SESSION_IMPROVEMENT.md` e somente as lições aplicáveis em `docs/SESSION_LEARNINGS.md`.
2. Registre HEAD, branch e `git status --short` antes de editar. Preserve mudanças alheias e não abra arquivos sensíveis sem necessidade autorizada.
3. No todo, transforme cada lição aplicável em ação observável ou registre por que não se aplica. Defina entrega, arquivos, pós-condições, orçamento e critério de parada.
4. Consulte `docs/EVIDENCE_REGISTER.md` antes de reservar um `EVD-*` futuro. IDs novos ficam em scratch até que arquivo, execução e registro existam.
5. Selecione somente as skills adicionais aplicáveis:

   | Gatilho | Skill adicional |
   |---|---|
   | Alterar código, contrato ou documentação | `dfir-change` |
   | Declarar integração ou verificação | `dfir-verify` |
   | Alterar regras `EVTX_DETECTION` | `dfir-detect`, além das duas anteriores |
   | Coordenar workers | `.jcode/swarm-prompt.md` e `AGENTS.md` §17, sem criar skill genérica de swarm |

Não trate skill global como local, não instale tooling implicitamente e não crie uma skill apenas para repetir a política de seleção de skills.

## 2. Executar com feedback

- Registre em scratch comandos, exits, diagnósticos, hashes e pós-condições necessários ao fechamento. Não copie dados de caso, nomes de evidências, eventos ou secrets.
- Confirme a pós-condição independente, não apenas exit 0. Uma busca vazia inesperada exige controle positivo ou método alternativo.
- Aplique os limites de investigação de `docs/SESSION_IMPROVEMENT.md`. Depois de duas tentativas sem informação nova, mude de método ou consolide a pendência, sem repetir a mesma chamada com outro nome.
- Após uma mutação de todo, swarm ou background, leia o estado retornado e confirme que a transição esperada ocorreu. Use IDs completos e não duplique workers nem builds. IDs de `await_members` pertencem somente a `todo.waiting_on` e ao acompanhamento de swarm, nunca a `bg wait`.
- Mantenha trabalho Rust pesado sequencial. Não cancele um build silencioso antes de inspecionar progresso e consumo.
- Não antecipe `PASS`, `VERIFIED` ou `ACCEPTED`. Não transforme proposta, hipótese ou lição em requisito sem a autoridade correspondente.

## 3. Integrar e verificar

1. Compare a entrega com o todo e a autoridade. Remova artefatos acidentais, dados sensíveis e mudanças fora da tarefa.
2. Execute `dfir-verify` ou os gates aplicáveis ao conteúdo alterado. Documentação não herda gates de código, e código não é verificado apenas pelo `doc-gate`.
3. Revise `git diff --check`, o diff semântico e `git status --short`. Confirme arquivos, comandos, exits e pendências.
4. Aguarde os workers necessários, revise suas entregas e verifique `swarm list` e `bg list`. Antes de assumir a escrita de um worker parado, confirme seu estado terminal e releia o artefato imediatamente antes de editar; trate escrita atrasada como nova entrega concorrente a revisar. Não deixe processos próprios sem dono.
5. Faça commits pequenos somente após os gates. Push e outras mudanças externas exigem autorização explícita e vínculo posterior do resultado ao commit.

## 4. Fechar o loop de aprendizado

- Para cada lição reutilizável nova, implemente a menor prevenção eficaz: instrução, checklist, teste, lint ou gate. Uma recomendação genérica sem controle não é prevenção.
- Atualize `docs/SESSION_LEARNINGS.md` somente para lição nova, recorrência ou confirmação observada. Não crie retrospectiva permanente de rotina.
- Confirme uma prevenção anterior somente por ação observada nesta sessão. Ausência de reclamação ou passagem de tempo não confirma aprendizado.
- Se não houver conteúdo novo ou confirmação aplicável, registre na resposta `sem nova lição de repositório`.
- Relate arquivos, commits, comandos e resultados reais, seguidos do que não foi executado e das pendências externas. Não converta `WAIVED` em verificação.
