<!-- doc-type: backlog | status: proposed -->
# Checklist final consolidado das análises de código e documentação

## 1. Natureza, ativação e fontes

Este documento é uma **proposta derivada**, não uma nova exigência, uma matriz de estado ou autorização para iniciar mudanças. Nenhum item foi executado como parte desta consolidação. Cada item só começa por ativação explícita do operador, com reprodução no HEAD então vigente, escopo exclusivo e gates próprios.

- **REPO-FACT:** base revisada `1c2bb5c209ff0902b8019a81928199986074e05e`, também publicada em `origin/main` antes desta análise.
- **REPO-FACT:** relatório documental `ANALYSIS_CONSOLIDATED.md`, 348 linhas, SHA-256 `3f3b9f12164a6cca6c89dc877f26397983664372c1887a4790d6f299ca0a26c9`.
- **REPO-FACT:** relatório de código `docs/CODE_ANALYSIS_REPORT.md`, 257 linhas, SHA-256 `99dbcb3c04ae7ffa2542dc861c03d88716ddfaae5be9472b6b667325c0ff3986`.
- **REPO-FACT:** os anexos `analysis_adrs.md` e `analysis_spec_docs.md` foram considerados no relatório documental. Os quatro arquivos permanecem não rastreados e inalterados a pedido do operador.
- **INFERENCE:** prioridade e recorte abaixo são propostas do coordenador após revisão independente e duas lanes de análise. Relatórios e artefatos de workers não substituem a autoridade do projeto.

A numeração dos itens é local deste checklist. Ela não cria IDs normativos nem substitui `RF-*`, `F1-*`, `EVD-*` ou ADRs.

## 2. Provas e métricas da triagem

### 2.1 Gaps reproduzidos em cópia descartável

1. **Recuperação de órfãos encerra job vivo de outra instância.** A mutação `second_instance_recovery_preserves_live_job` iniciou um engine de teste vivo e chamou `recover_orphaned_jobs` por uma segunda instância. O teste falhou com `(1, "FAILED")` em vez de `(0, "PROCESSING")`, exit 101. A limpeza do engine passou antes da assertiva final. Log: `validation/task-007-orphan-repro.txt`, SHA-256 `d897dd98dd51f1a18903b3d33279ab9811e9605dd8997ac62e014089bd95c65e`.
2. **Linha persistida acima da moldura é aceita.** A mutação `stored_results_enforce_protocol_line_limit` forneceu uma linha JSON válida acima de `MAX_LINE_BYTES`. `read_result_items` retornou `Ok([])` em vez de rejeitar antes da desserialização, exit 101. Log: `validation/task-007-line-limit-repro.txt`, SHA-256 `3e27ac6939a04c9d7b79458282b98aa451c95bf6878afeaabe039062e82a1bfb`.
3. **Controle de procedimento preservado.** A primeira tentativa do item 1 usou o checkout sem a mutação e executou zero testes. Isso não foi usado como evidência. Log: `validation/task-007-orphan-repro-zero-tests.txt`, SHA-256 `5e809475f6ae9568bae6bfb5a32326a518329e88455403a60309c28a3f58220d`.

As reproduções provam os comportamentos atuais, não uma futura correção. Os logs ficam no diretório de validação do run, fora do checkout.

### 2.2 Métricas que invalidam estimativas dos relatórios

| Métrica | Relatório | Estado verificado |
|---|---|---|
| Testes | 504 | A baseline fresca de `EVD-062` tem 713 aprovados e 1 skipped. Não usar 504 nem a contagem divergente do worker como baseline de redução. |
| Migrations | 0001 a 0011 | Existem 14 arquivos, 0001 a 0014. |
| Regras de detecção | 18 | `docs/DETECTION_RULES.md` lista 19 regras na versão 2026.11.1. |
| `service/mod.rs` | 2.400+ linhas | 4.393 linhas no HEAD revisado. |
| `NATIVE_UI_ARCHITECTURE.md` | cerca de 600 | 83 linhas. |
| `RELEASE_GATE.md` | cerca de 2.000 | 178 linhas. |
| `rtm.md` | cerca de 2.000 | 290 linhas. |
| `FASE1_STATUS.md` | cerca de 2.000 | 69 linhas. |
| `SUPPLY_CHAIN.md` | cerca de 400 | 192 linhas. |
| Especificação | 4.854 linhas | Confirmado. As duas cópias têm SHA-256 `034c5ff0079da87536fe60e96bb2908e1bdc673d38073f8abdb41dfc6b40d909` e são byte-equivalentes. |

**Consequência:** as metas de reduzir 30-40% dos testes ou 63% da documentação não têm baseline nem benefício semântico demonstrado. Elas não orientam este checklist.

## 3. Checklist para ativação posterior

### Prioridade 0: preservar jobs vivos entre instâncias

Origem: relatório de código, gap de `recover_orphaned_jobs` com múltiplos processos, confirmado pela reprodução do item 2.1.1. O código atual chama a recuperação na partida de cada GUI (`src/gui/backend.rs:687-695`) e seleciona todos os jobs `PENDING/PROCESSING` sem distinguir o dono (`src/service/mod.rs:1594-1633`). A própria função documenta que marcaria um job vivo se houvesse outro processo analisando.

- [ ] Reexecutar a reprodução no HEAD de trabalho e preservar o Before com um job realmente vivo e uma segunda instância.
- [ ] Definir a semântica de propriedade, lease ou liveness compatível com o cancelamento entre instâncias já suportado. Não impor instância única nem encerrar jobs legítimos como atalho. Registrar ADR se o contrato arquitetural mudar.
- [ ] Adicionar regressões com duas instâncias: uma com job vivo que deve ser preservado e outra com job comprovadamente morto que deve virar `FAILED/ORPHANED` com auditoria íntegra.
- [ ] Implementar a menor mudança que recupere apenas jobs órfãos, sem perder conclusão tardia válida, cancelamento persistido, índice ou cadeia de custódia.
- [ ] Confirmar Before falho e After verde, processo fake recolhido, nenhum engine órfão e nenhum índice parcial.
- [ ] Executar `just verify`, E2E com sandbox obrigatória e testes focados multiinstância. Atualizar contratos e RTM somente se o estado real mudar.

**Pós-condição independente:** iniciar uma segunda instância não altera um job vivo da primeira, enquanto um job cujo dono morreu é recuperado uma única vez, com auditoria e custódia válidas.

### Prioridade 1: limitar a moldura antes de ler e desserializar resultados

Origem: o relatório propõe um teto total em `results.jsonl`. A triagem reduziu e corrigiu o escopo. `MAX_LINE_BYTES` já é 8 MiB (`protocol/src/lib.rs:21`), mas os dois leitores persistidos usam `lines()` sem esse limite (`src/service/mod.rs:1773-1805` e `1914-2008`). A profundidade só é verificada depois que a linha inteira já foi alocada.

- [ ] Preservar a reprodução que aceita uma linha acima de `MAX_LINE_BYTES` em `read_result_items`.
- [ ] Adicionar Before equivalente para indexação de timeline, incluindo índice anterior preservado após a rejeição.
- [ ] Implementar leitura limitada que rejeita linha excessiva antes da desserialização, com memória proporcional à moldura e sem `lines()` sem teto. Reutilizar a constante e a política central de `dfir-protocol`.
- [ ] Cobrir os dois caminhos: limite exato aceito, limite+1 rejeitado, várias linhas válidas sem teto total e diagnóstico com número da linha, sem conteúdo do evento.
- [ ] Confirmar rollback completo na indexação e ausência de perda silenciosa de resultados válidos.
- [ ] Executar testes focados, `just verify` e E2E. Não adicionar teto total de saída nem alterar ADR-0020 sem uma nova decisão explícita.

**Pós-condição independente:** uma linha acima da moldura falha fechado antes do parse, sem alocação ilimitada e sem publicar índice parcial. Arquivos com muitas linhas válidas continuam completos, conforme ADR-0020.

### Prioridade 2: ampliar propriedades da conversão para timeline

Origem: relatório de código, fuzzing de `NormalizedEvent::from_item` e `timeline_event_from_item`. O protocolo já tem round-trip com JSON arbitrário (`protocol/tests/properties.rs:93-101`), mas a conversão específica da aplicação tem cobertura limitada.

- [ ] Inventariar os casos atuais e evitar duplicar as propriedades já existentes do protocolo.
- [ ] Usar `proptest` já presente para gerar entradas estruturalmente válidas e inválidas para `timeline_event_from_item`, incluindo IDs qualificados, campos desconhecidos, legado, tipos errados e dados ausentes.
- [ ] Provar ausência de panic, rejeição de itens que não são timeline e consistência entre campos normalizados e apresentados.
- [ ] Manter a profundidade e a moldura como controles anteriores à conversão. Não criar nova dependência de fuzzing nem converter propriedades úteis em poucos exemplos determinísticos.
- [ ] Executar propriedades focadas, `just verify` e E2E se a conversão de produção mudar.

**Pós-condição independente:** entradas arbitrárias dentro dos limites preservam a mesma invariante da conversão e nunca causam panic ou aceitação de um item de outro tipo.

### Prioridade 3: consolidar somente duplicação de teste comprovada

Origem: relatório de código, itens 1 e 2 da priorização. Existe setup repetido, mas `service::test_support::Fixture` já existe e `gui::test_harness` já é um módulo compartilhado inline em `src/gui/mod.rs:1080`. Não criar implementações paralelas nem presumir a economia de 500 linhas.

- [ ] Medir duplicação exata e decidir arquivo por arquivo se a extração preserva setup, migrações, permissões e isolamento.
- [ ] Reutilizar o `Fixture` existente quando compatível. Criar suporte de persistência somente para código realmente idêntico e usado por pelo menos dois módulos.
- [ ] Se a extração da GUI for útil, mover o módulo inline existente para um arquivo sem mudar sua política de glifos, IDs, foco, spinner ou geometria.
- [ ] Comparar inventário de testes e nomes antes/depois. Não remover testes, reduzir casos de propriedade nem transformar testes de layout em snapshots visuais.
- [ ] Executar os testes de cada módulo migrado e, ao final, `just verify`.

**Pós-condição independente:** mesmos cenários e assertivas, menos duplicação real, nenhum helper concorrente e nenhum teste perdido ou enfraquecido.

### Prioridade 4: pequenas correções e índices documentais

Origem: relatório documental, itens 14 a 17, mais uma lacuna encontrada pelo coordenador na ADR-0006. Estas ações não envolvem poda, arquivamento ou reescrita de decisões.

- [ ] Resolver a adenda vazia da ADR-0006 com uma nota que aponte para as decisões posteriores já existentes sobre memória e contenção, sem alterar retroativamente a decisão original.
- [ ] Acrescentar na ADR-0017 apenas um ponteiro para a ADR-0029 e a versão vigente 0.37. Preservar 0.33.3 e 0.36.2 como histórico.
- [ ] Criar `docs/adrs/README.md` com os 30 ADRs, status, `supersedes` e ligações, sem mover os arquivos nem reclassificá-los como "overkill".
- [ ] Se houver padronização, aplicar `Status`, `Supersedes`, `Superseded-by` e `Evidência` aos ADRs novos ou já editados. Não reescrever os 30 em massa.
- [ ] Vincular somente evidências existentes aos ADRs antigos. Não inventar execução retroativa nem transformar referência em novo `PASS`.
- [ ] Executar `just doc-gate` e revisar todos os links e a hierarquia documental.

**Pós-condição independente:** o índice resolve navegação, as notas refletem referências vigentes e todo conteúdo histórico permanece preservado.

### Prioridade 5: benchmark manual somente se houver meta de desempenho

Origem: relatório de código, item 7 e teste ignorado de 1 milhão de eventos. Já existe `timeline_one_million_events_index_and_paging_timing`, marcado como medição manual (`src/service/mod.rs:3662-3700`). Não é um gap de E2E nem deve entrar no gate padrão sem orçamento.

- [ ] Definir métrica, hardware, build, tempo máximo e espaço em disco antes da execução.
- [ ] Rodar somente o benchmark escolhido, nunca todos os ignorados por rotina, e preservar tempos de geração, indexação, páginas e filtro.
- [ ] Separar resultado de desempenho de correção funcional. Não prometer melhoria nem aceite de produção com uma única medição.
- [ ] Se houver otimização posterior, exigir baseline, comparação no mesmo ambiente e ausência de regressão funcional.

**Pós-condição independente:** existe uma medição reproduzível e identificada, sem transformar o benchmark em gate obrigatório ou aceite de latência.

## 4. Já atendido, sem nova execução

| Item dos relatórios | Evidência atual e destino |
|---|---|
| Centralizar `MAX_JSON_DEPTH` e helper | Já são públicos em `protocol/src/lib.rs:34` e `405`, usados por app e engine. Não repetir. |
| Adicionar `just verify` com workspace | Já existe em `justfile:89-95` e passou em `EVD-062` com 713 aprovados/1 skipped, doc-tests e Clippy. |
| Criar E2E real com sandbox e cancelamento | Já existe e passou 123/123 em `EVD-062`, incluindo contenção e cancelamento. |
| Criar `NATIVE_UI_ARCHITECTURE.md`, `DETECTION_RULES.md` e `CHANGELOG.md` | Os três existem e são atuais. Não recriar. |
| Corrigir buffer de 1 MiB | Já usa heap em `src/service/mod.rs:2411-2414`. |
| Corrigir `unwrap()` nos pontos citados | `wgpu_render_state` usa `let-else` com retorno e `value_to_string` usa `unwrap_or_else`. Não há o panic relatado nesses pontos. |
| Acrescentar gatilho de SBOM | Já existe em `ADR-0028:29`, exigindo nova ADR se houver distribuição a terceiros. |
| Corrigir Windows N2 como falsa garantia | ADR-0024:44-50 já declara parcial, não executado em Windows e `PENDING-HUMAN`. O release mantém waiver e não declara suporte. Não reescrever como se a limitação estivesse oculta. |
| Arquivar ADR-0005 e ADR-0009 | Já têm status `partially-superseded` e `superseded` com avisos no topo. Preservar no histórico de ADRs, sem arquivamento obrigatório. |
| Atualizar versões na ADR-0017 | A ADR já distingue 0.33.3 e 0.36.2; a nota 10 da especificação e a ADR-0029 registram 0.37. Somente o ponteiro opcional da prioridade 4. |

## 5. Não executar como escrito nos relatórios

| Proposta | Motivo da rejeição ou correção necessária |
|---|---|
| Adicionar `max_total_bytes` em `results.jsonl` | ADR-0020 exige saída sem teto total, com falha explícita somente quando o operador escolher um limite. O gap comprovado é a moldura por linha, tratado na prioridade 1. |
| Adicionar `file_mtime/size` ao `timeline_index` por padrão | Não foi demonstrada mutação legítima de um output `COMPLETED`. O índice é construído ao concluir e refeito apenas se ausente. Antes de qualquer migration, reproduzir um defeito real e avaliar integridade de conteúdo, não apenas metadados de filesystem. |
| Instalar `rstest` para parametrização cosmética | A dependência não está no workspace. Uma dependência nova exige necessidade, revisão do grafo, `cargo deny`, `cargo audit` e lock atualizado. Tabelas comuns bastam para muitos casos. |
| Converter proptest de referências lógicas em poucos testes unitários | Reduz a exploração de entradas e não prova equivalência de cobertura. Manter propriedades e acrescentar exemplos determinísticos apenas quando úteis. |
| Substituir `egui_kittest` por snapshots visuais ou remover testes de layout/scroll | A árvore interna e a geometria cobrem comportamento que uma imagem não prova. EVD-061 e EVD-062 confirmam resize e fluxo nativo, sem promover AT-SPI. Não reduzir essa cobertura. |
| Remover o teste de identidade estável do evento como duplicado | Identidade e round-trip são invariantes diferentes. Não remover. |
| Criar trait `ServiceContext` ou separar `text_safety` em novo crate | Não há necessidade demonstrada, e novas abstrações aumentariam superfície e validação sem resolver defeito atual. |
| Extrair helper transacional genérico por padrão | Transações críticas e invariantes não devem ficar escondidas. Qualquer extração exige caso concreto e prova de que rollback e auditoria permanecem explícitos. |
| Separar `service/mod.rs` apenas por número de linhas | A extração 1:1 é decisão deliberada da ADR-0017. Dividir módulos sem fronteira comportamental útil não é correção prioritária. |
| Tornar `read_result_items` streaming por padrão | A função já usa o leitor compartilhado em fluxo e coleta apenas resultados YARA, cujo volume é pequeno. O problema comprovado é a moldura, não essa coleta. |
| Criar `SECURITY.md` como requisito | O conteúdo já está distribuído em AGENTS, ADRs e documentos operacionais. Um índice novo só se houver lacuna de navegação demonstrada, sem duplicar o threat model. |
| Remover 11 seções "superseded" da especificação | As notas 1 a 11 e a hierarquia já definem o que prevalece. `DOCUMENT_AUTHORITY.md:68` exige preservar o histórico com notas, não apagá-lo. Pode haver simplificação editorial futura, não correção de violação. |
| Mover diretrizes LLM das seções 45-47 para `AGENTS.md` | `DOCUMENT_AUTHORITY.md:17` declara ambos como autoridade de regras LLM. Não quebrar essa relação sem decisão formal. |
| Separar `RELEASE_GATE.md` e `SUPPLY_CHAIN.md` por suposto excesso | Têm 178 e 192 linhas, não os tamanhos relatados. As definições e os registros já têm papéis atribuídos. |
| Reduzir `FASE1_STATUS.md` a índice | O arquivo tem 69 linhas e já é histórico cronológico, não a matriz de estado. A RTM continua sendo a única matriz. |
| Mover parsers futuros, air-gapped e performance para fora da especificação | Roadmap e fases futuras fazem parte da especificação canônica. Não confundir conteúdo planejado com requisito do MVP nem reabrir escopo por poda editorial. |
| Mesclar ADR-0020 e ADR-0021 ou mover ADRs "técnicos" | As decisões têm linhagem e autoridade próprias. Preferir referências cruzadas pontuais, sem apagar ou deslocar histórico. |
| Mover as seções 50 e 53 da especificação por rotina | Índice de decisões e histórico são parte do documento canônico. Qualquer mudança exige processo de autoridade e cópia raiz byte-equivalente, não uma limpeza automática. |
| Fixar benchmark de 2x na ADR-0006 | O limiar é inventado. A reavaliação atual já inclui bug, requisito, segurança, licença e benchmark. Definir métrica apenas diante de alternativa real. |
| Buscar redução percentual de testes ou documentação | O objetivo é preservar comportamento e clareza. Redução de linhas sem inventário semântico e pós-condições não é aceite. |

## 6. Cobertura integral das recomendações documentais

A tabela usa a numeração 1 a 21 da seção 6 de `ANALYSIS_CONSOLIDATED.md`. Nenhuma recomendação ficou sem destino.

| Item | Destino consolidado |
|---|---|
| 1. Corrigir Windows N2 | Já delimitado pela ADR-0024 e waiver. Não reescrever como risco oculto. Verificação externa continua `PENDING-HUMAN`, fora deste checklist. |
| 2. Remover 11 seções da especificação | Não executar. Notas de supersedição e preservação histórica prevalecem. |
| 3. Arquivar ADR-0005/0009 | Não é necessário. Os status e avisos já existem. |
| 4. Mover diretrizes LLM | Não executar. A autoridade compartilhada está declarada. |
| 5. Separar `RELEASE_GATE` | Não executar pelo tamanho ou histórico alegado. |
| 6. Mover parsers futuros | Não executar. Permanecem roadmap, não MVP. |
| 7. Mover air-gapped/atualização | Não executar como poda de rotina. |
| 8. Mover performance | Não mover. Somente benchmark condicionado da prioridade 5. |
| 9. Consolidar escopo EVTX | Não mesclar ADRs. Referência cruzada opcional, sem mudança de escopo. |
| 10. Mesclar ADR-0020/0021 | Não executar. Preservar decisões separadas e sua linhagem. |
| 11. Mover ADRs técnicos | Não executar. Inclui decisões explicitamente reconhecidas por AGENTS. |
| 12. Transformar `FASE1_STATUS` em índice | Não executar. Já é log cronológico de 69 linhas. |
| 13. Separar `SUPPLY_CHAIN` | Não executar pelo tamanho alegado. |
| 14. Atualizar versões na ADR-0017 | Apenas ponteiro opcional da prioridade 4. |
| 15. Padronizar ADRs | Somente novos/editados, sem reescrita em massa. Prioridade 4. |
| 16. Retroativar evidências | Somente links para evidências reais existentes. Prioridade 4. |
| 17. Criar índice de ADRs | Aceito como proposta opcional na prioridade 4. |
| 18. Simplificar seção 50 | Sem ação obrigatória. Só por decisão editorial futura com cópia equivalente. |
| 19. Mover seção 53 | Não executar. Preservar histórico canônico. |
| 20. Acrescentar gatilho SBOM | Já existe em ADR-0028:29. |
| 21. Fixar critério numérico na ADR-0006 | Não inventar 2x. Reabrir somente com alternativa e métrica reais. |

## 7. Cobertura integral das recomendações de código

A tabela usa os dez itens da seção 8 de `docs/CODE_ANALYSIS_REPORT.md`. Os demais achados aparecem nas seções 3 a 5.

| Item | Destino consolidado |
|---|---|
| 1. Suporte comum de persistência | Opcional, com inventário semântico e sem perda de testes. Prioridade 3. |
| 2. Consolidar `gui/test_harness` | Opcional, movendo o módulo compartilhado existente somente se houver benefício. Prioridade 3. |
| 3. Helper transacional genérico | Não por padrão. Invariantes e transações críticas permanecem explícitas. |
| 4. Centralizar profundidade JSON | Já implementado em `dfir-protocol`. |
| 5. `file_mtime/size` no índice | Não por padrão. Reproduzir mutação legítima antes de propor migration. |
| 6. `SECURITY.md` e arquitetura nativa | Arquitetura nativa já existe. Documento de segurança só se houver lacuna de navegação demonstrada. |
| 7. Benchmark ignorado | Somente medição manual orçada da prioridade 5. |
| 8. `rstest` em testes críticos | Não adicionar dependência para esse fim sem necessidade e supply chain. |
| 9. `rstest` em validações | Mesma decisão do item 8. |
| 10. Trocar proptest por unitários | Não executar. Preservar e ampliar propriedades, inclusive na prioridade 2. |

Mapeamento dos demais achados: recuperação multiinstância na prioridade 0, moldura na prioridade 1, fuzzing da conversão na prioridade 2, E2E e documentos existentes na seção 4, teto total e alegações de `unwrap()` na seção 5. Buffer, coleta YARA, divisão do serviço e abstrações opcionais não viram trabalho sem defeito demonstrado.

## 8. Regras de execução e fechamento

Para cada item ativado:

1. Confirmar HEAD, status, autoridade, escopo e dono exclusivo. Revalidar a afirmação porque este checklist descreve uma revisão pontual.
2. Preservar Before com diagnóstico e pós-condição independente. Busca vazia ou zero testes exige controle positivo.
3. Fazer a menor mudança suficiente e testes After. Não marcar o checkbox por inspeção estática.
4. Executar `just verify` para código e `just doc-gate` para documentação. E2E é obrigatório quando o fluxo real do engine for afetado.
5. Executar `just supply-chain` se manifest, lock ou grafo mudar. Não instalar dependência ou tooling implicitamente.
6. Repetir smoke X11 pós-condicionado se a UI mudar. Não promover estética, AT-SPI, latência ou outras plataformas sem evidência própria.
7. Atualizar somente documentos afetados e a RTM quando houver mudança real de estado. Se a especificação mudar, manter as duas cópias byte-equivalentes.
8. Commitar por item coerente após os gates. Push, publicação ou outra ação externa exige autorização nova e explícita.

## 9. Estado de entrega

Todos os checkboxes ficam intencionalmente abertos para início posterior. A triagem e as reproduções não contam como execução dos itens. Nenhum `EVD-*` novo, `VERIFIED`, `ACCEPTED`, mudança de escopo ou aceite de produção decorre deste documento.
