<!-- doc-type: backlog | status: proposed -->
# Plano priorizado de melhorias do MVP

## Natureza e base

Este documento é uma **proposta derivada**, não uma fonte de requisitos nem uma matriz de estado. Ele não cria tickets, não altera `RF-*`, não promove itens a `IMPLEMENTED`, `VERIFIED` ou `ACCEPTED` e não reabre escopo adiado. A autoridade continua sendo a demanda original, a especificação, as ADRs, a RTM e os gates, nessa ordem conforme `docs/DOCUMENT_AUTHORITY.md`.

- **REPO-FACT:** auditoria realizada sobre `6834a8a244981510ed3f152eea106ccf5dcc00c9`, com leitura posterior das mesmas estruturas antes desta redação.
- **REPO-FACT:** as políticas operacionais e a skill `dfir-detect` foram separadas no commit `c72c1d1`, após `just doc-gate` local com exit 0. Isso não é evidência de release do produto.
- **EXTERNAL-FACT:** o CI `44590903651` foi confirmado em `success` por `gh` para o HEAD `6834a8a`. Esse resultado não cobre commits posteriores.
- **INFERENCE:** as fatias abaixo usam os pontos de extensão de menor mudança identificados no código. A implementação ainda precisa de revisão de diff e gates próprios.
- **PROPOSAL:** prioridade, recorte e critérios de aceite deste documento só se tornam trabalho aprovado por decisão explícita do operador e pela atualização dos artefatos de autoridade aplicáveis.

As numerações das fatias são apenas âncoras locais deste plano. Não são IDs normativos nem substituem `F1-*`, `RF-*`, `EVD-*` ou ADRs.

**Ativação em 2026-11-20:** o operador autorizou explicitamente a execução das fatias 1–6 nesta sessão. As melhorias de produto usam os tickets existentes; os cinco recursos anteriormente adiados foram alocados por ADR-0030 em F1-36..F1-40. Esta autorização ativa trabalho, não antecipa implementação, gates ou aceite. A tabela abaixo preserva a ordem inicial recomendada; o estado normativo continua exclusivamente na RTM.

## 1. Ordem recomendada

| Ordem | Fatia | Classe | Motivo da prioridade | Dependência ou limite |
|---|---|---|---|---|
| 1 | Guarda de retenção e filtros RTK | Engineering/QA Tooling | Reduzir retenção de saídas potencialmente sensíveis antes de ampliar o uso | Não bloqueia release nem exige hook global |
| 2 | Candidato de detecção para eventos da timeline | Production Runtime | Melhorar revisão humana reutilizando referências e paginação existentes | Manter EVTX, sem migration nem novo schema do protocolo |
| 3 | Histórico NIST Current/Target na GUI | Production Runtime | Expor dados já entregues pelo serviço, sem alterar persistência | Somente leitura, sem mudar gap analysis |
| 4 | Linhagem de resultados analíticos | Production Runtime | Tornar histórico, relações e provenance inspecionáveis sem perda silenciosa | Promover consulta restrita a testes e definir escopo das relações |
| 5 | Hardening orientado por diagnóstico somente leitura | Production Runtime e procedimento | Melhorar o plano RF-052 sem modificar clientes | Não inclui Ansible nem remediação automática |
| 6 | Novas regras EVTX, uma família por vez | Production Runtime | Ampliar cobertura somente com sinal, falsos positivos e fixtures delimitados | Cada regra exige aprovação e rastreabilidade próprias |

As fatias 2 e 3 são independentes e podem ser paralelizadas com donos de escrita distintos. As demais devem permanecer sequenciais quando compartilharem serviço, backend da GUI ou contratos. Builds e testes Rust pesados continuam sequenciais neste host.

## 2. Fatia 1: RTK sem retenção indevida

### Fatos e proposta

- **REPO-FACT:** `.rtk/filters.toml` contém `schema_version = 1` e somente exemplos comentados. Não há filtro local ativo.
- **EXTERNAL-FACT:** o host tem RTK `0.49.0`, nenhum hook instalado e `recall mode: sqlite`. A verificação anterior executou 154 testes internos, mas ignorou filtros locais não confiáveis.
- **PROPOSAL:** antes de usar RTK em comandos que possam emitir nomes de evidências, eventos, caminhos de casos, exports ou secrets, selecionar `recall disabled`. Nenhuma configuração foi alterada por este plano.
- **PROPOSAL:** manter RTK fora de importação, processamento de EVTX, diagnósticos, auditoria, exports, inspeção SQLite e qualquer comando com dados de caso. Filtro de texto não é mecanismo de sanitização nem controle de confidencialidade.

### Implementação em passos pequenos

1. Registrar no tooling a política de `recall disabled` para trabalho com dados potencialmente sensíveis, com exceção opt-in apenas para fixtures sanitizadas e diretório de dados restrito ao usuário.
2. Adicionar somente filtros com ruído medido para comandos de engenharia como `git`, `cargo`, `just` e `gh`. Cada filtro deve preservar exit code, stderr, falhas, contagens e caminho de recuperação da saída bruta.
3. Criar testes inline positivos e negativos para cada filtro. Cobrir linhas de erro, advisories, resultados de testes, tokens semelhantes a secrets e textos que jamais podem ser descartados.
4. Revisar o TOML antes de `rtk trust`. Executar `rtk verify --require-all` no escopo confiável e medir a redução sem perda de diagnóstico.
5. Não instalar hook global por padrão. Se um hook for proposto, delimitar comandos, retenção, recuperação e rollback em decisão de tooling separada.

### Aceite

- Com `recall disabled`, comandos de engenharia com fixtures sanitizadas não criam registros de recall; nenhum teste usa dados de caso.
- Os filtros ativos têm testes inline, confiança explícita e resultado recuperável com `rtk proxy` quando necessário.
- Uma falha injetada continua visível e preserva o exit não zero.
- Nenhum dado sensível, hook global ou instalação implícita entra na mudança.

## 3. Fatia 2: candidato para eventos de origem na timeline

### Base e recorte

- **REPO-FACT:** `src/service/detection.rs::CandidateView` já expõe `job_id`, `event_refs` e `refs_truncated`. `src/gui/detection.rs::candidate_ui` mostra as referências, mas não navega.
- **REPO-FACT:** `timeline_events` e `TimelineEvent` já possuem `raw_reference`; `src/service/mod.rs::{timeline_page_core,push_timeline_filter}` consulta páginas no banco. `src/gui/backend.rs::Command::TimelinePage` já suporta evidência, job opcional, filtro, offset e total.
- **REPO-FACT:** `src/service/timeline_filter.rs` ainda não possui filtro por `raw_reference`.
- **INFERENCE:** acrescentar um termo estruturado `ref:` e reutilizar a timeline existente evita migration e mudança de `SCHEMA_VERSION`. O `job_id` do candidato identifica a detecção e **não** deve ser usado como se fosse o job da timeline.

### Implementação

1. Em `src/service/timeline_filter.rs`, adicionar referências ao `TimelineFilter`, ao parser e à documentação do termo `ref:record:N`. Valores separados por vírgula significam OU e a comparação é exata.
2. Em `src/service/mod.rs::push_timeline_filter`, aplicar igualdade sobre `raw_reference`, preservando binds e os demais filtros.
3. Em `src/gui/detection.rs::candidate_ui`, oferecer ação “Ver eventos na timeline” para as referências disponíveis. Se `refs_truncated`, informar que a seleção contém somente as referências preservadas no candidato.
4. Em `src/gui/evidence.rs::details` e `src/gui/timeline.rs`, abrir a timeline da mesma evidência, limpar páginas de outro filtro, solicitar offset zero e resolver o job EVTX concluído pelo caminho existente de `job_id: None`. Não alterar a decisão do candidato.
5. Atualizar somente a documentação de filtro/UI afetada. Não criar API HTTP, socket, migration ou item novo do protocolo.

### Aceite e testes

- `ref:record:5,record:9` retorna somente essas referências, sem casar `record:50`, e combina corretamente com outros filtros.
- O clique envia a evidência correta, `job_id: None`, filtro exato, offset zero e `want_total`; respostas antigas não sobrescrevem o novo filtro.
- Candidato sem referências tem estado explicável, e candidato truncado não afirma mostrar todos os eventos.
- Confirmar ou rejeitar continua sendo uma ação humana separada. A navegação não muda status, achados, IOCs nem auditoria de decisão.
- Testes focados em `timeline_filter`, timeline de serviço, `gui::detection`, `gui::timeline` e roteamento de backend, seguidos dos gates comuns da seção 8.

**Rastreabilidade:** F1-26, F1-28, RF-048, RF-050 e ADR-0025. Reuso da paginação e da timeline em banco conforme ADR-0020.

## 4. Fatia 3: histórico NIST completo e somente leitura

### Base e recorte inicial

- **REPO-FACT (base inicial):** `ProfileHistoryView` já chega à GUI por `src/gui/backend.rs::Event::NistProfileHistoryLoaded` e `src/gui/nist.rs::on_history`.
- **REPO-FACT (base inicial):** `src/gui/nist.rs::profile_column` usava somente `history.versions.last()` para as entradas e informava apenas a quantidade de versões anteriores.
- **INFERENCE:** um seletor de versão na GUI é suficiente para a primeira fatia. Não são necessários migration, novo comando de serviço ou mudança do modelo append-only.

### Implementação

1. Em `src/gui/nist.rs::{NistState,profile_column}`, manter seleção independente para Current e Target, com a versão vigente como padrão.
2. Exibir versão, fonte, tier, justificativa, data e entradas da versão selecionada, sem editar ou apagar histórico.
3. Deixar explícito quando a versão exibida não for a vigente. A seleção visual não pode recalcular gaps nem alterar qual versão é usada em novas análises.
4. Preservar a visão por Function e a ausência de score oficial do NIST CSF 2.0.
5. Ampliar os testes existentes em `src/gui/nist.rs`; não mudar persistência se a entrega completa de `ProfileHistoryView` permanecer suficiente.

### Aceite e testes

- Duas ou mais versões podem ser percorridas e cada seleção mostra as entradas e metadados correspondentes.
- Current e Target não compartilham seleção acidentalmente; lista vazia e versão única continuam estáveis.
- Abrir uma versão antiga não emite comando de criação, atualização, remoção ou recálculo.
- A versão vigente continua sendo o padrão após recarga e a indicação de gap `STALE` não é escondida.
- Testes `egui_kittest` de seleção e conteúdo, seguidos dos gates comuns. Eles não substituem o fluxo nativo X11 nem declaram AT-SPI.

**Rastreabilidade:** F1-03, RF-009, RF-010 e ADR-0011.

### Estado em 2026-11-23

Implementado o seletor independente de versão Current/Target com indicação de versão não vigente, sem comandos de escrita. `EVD-059` registra verify 701/701, E2E 123/123 e gates documentais na árvore integrada. O smoke X11 de `EVD-050` não navegou por essa seleção, portanto o aceite nativo específico e AT-SPI não são declarados.

## 5. Fatia 4: linhagem analítica inspecionável

### Base e decisão de desenho inicial

- **REPO-FACT (base inicial):** `src/persistence/analytical_results.rs::history_for_correlation_key` implementava o histórico ordenado, mas estava sob `#[cfg(test)]`. `related_to` já lia relações de saída.
- **REPO-FACT:** `src/service/mod.rs::AnalyticalResultResponse` usa `TryFrom<AnalyticalResultRecord>` e retorna erro para `payload` ou `provenance` JSON inválido. Essa conversão não pode ser substituída por `null` ou default.
- **REPO-FACT (base inicial):** `src/gui/evidence.rs` carregava resultados vigentes e apresentava o painel como “Resultados analíticos (YARA)”.
- **PROPOSAL:** o primeiro recorte contém o resultado selecionado, o histórico completo da mesma `correlation_key` e suas relações de saída. Grafo bidirecional, visualização global ou edição de relações não entram nesta fatia.

### Implementação

1. Promover `history_for_correlation_key` para uso de produção, mantendo ordem por `sequence` e os testes de supersessão.
2. Adicionar em `src/service/mod.rs` uma consulta de linhagem por resultado e escopo. Validar que o resultado pertence à evidência ou incidente solicitado antes de devolver histórico ou relações.
3. Converter cada registro por `AnalyticalResultResponse::try_from`. Identificar vigente, predecessor por `supersedes`, `RelationType`, payload e provenance sem reescrever os registros.
4. Ao expor `related_to`, não ocultar tipo de relação desconhecido nem alvo ausente. Propagar erro diagnosticável ou rejeitar a linhagem incompleta, com testes que impeçam retorno parcial silencioso.
5. Adicionar comandos/eventos assíncronos em `src/gui/backend.rs` e apresentação em `src/gui/evidence.rs`, fora da thread da UI. Renomear o painel somente se o conteúdo deixar de ser exclusivamente YARA.
6. Não criar tabela, migration, protocolo do engine ou novo runtime. Não executar otimização de consultas sem medida que a justifique.

### Aceite e testes

- O histórico aparece do mais antigo ao vigente, com todas as versões e relações de saída do resultado selecionado.
- Resultados de outra evidência ou incidente não são devolvidos, inclusive quando uma relação tenta cruzar o escopo.
- JSON inválido em qualquer item do histórico ou relação gera erro específico visível, nunca `payload: null`, `provenance: null` ou omissão silenciosa.
- A consulta é somente leitura e não cria nova versão, relação, achado ou decisão.
- Testes de persistência para histórico e relações, testes de serviço para escopo e dados malformados, e testes de GUI/backend para carregamento, erro e respostas atrasadas.
- Gates focados, `just verify`, E2E com fixture sanitizada e `just doc-gate`.

**Rastreabilidade:** F1-01 e ADR-0008 para o modelo comum; F1-04 e ADR-0012 para relações; regressão de conversão já registrada em `EVD-023`, sem reutilizar essa evidência para a nova funcionalidade.

### Estado em 2026-11-23

Implementado em `src/service/lineage.rs` e `src/gui/lineage.rs`, com histórico de produção, relações estritas e consulta em Evidências e no plano do incidente. Além do resultado selecionado, histórico, vigente e alvos das relações passam por `validate_scope`. O focado de linhagem passou 19/19 e `EVD-059` registra os gates integrados. O smoke X11 não abriu deliberatemente essa view, portanto seu aceite nativo específico não é declarado.

## 6. Fatia 5: hardening sem modificação automatizada de clientes

### Decisão de escopo

- **REPO-FACT:** ADR-0018 D-2 adia playbook Ansible/RF-012 para a Fase 4 e o substitui no MVP por itens estruturados RF-052. A demanda pede plano de hardening, não automação.
- **DECISÃO DE PLANEJAMENTO:** não automatizar instalação, configuração, desativação de controles, reinicialização, aplicação de políticas ou remediação em clientes no MVP. Esta decisão não exige nova ADR porque preserva ADR-0018.
- **PROPOSAL:** melhorar o conteúdo e a revisão dos itens de plano existentes. Qualquer execução automatizada futura exige decisão explícita do operador e processo de supersessão da ADR aplicável.

### Implementação segura

1. Em `src/service/plan.rs`, `src/persistence/plan_items.rs` e `src/gui/plan.rs`, revisar os campos já existentes de pré-condição, pós-condição e verificação para recomendações somente leitura, sem introduzir executor.
2. Produzir recomendações revisadas pelo analista com prioridade, esforço, custo, fase, dependências, risco, rollback conceitual e referência às evidências. Não transformar diagnóstico ausente em fato sobre o cliente.
3. Definir preflight manual: autorização, escopo de hosts, cópia de evidências, proveniência, ferramentas disponíveis, privilégio mínimo, espaço e critério de parada.
4. Qualquer diagnóstico deve ser somente leitura, local e explicitamente aprovado. Não coletar secrets, não enviar dados a serviços externos e não registrar nomes de evidências ou conteúdo de eventos em logs.
5. Manter a saída nos entregáveis existentes em `src/service/deliverables/`, distinguindo observado, inferido, não verificado e recomendado. Ansible continua fora do MVP.

### Aceite e testes

- Nenhum caminho de código executa comando de alteração, elevação de privilégio ou remediação em cliente.
- Cada recomendação tem pré/pós-condição e verificação manual, com fonte e estado de confiança explícitos.
- Falha de preflight impede a recomendação de prosseguir como se estivesse verificada; preservação e proveniência permanecem obrigatórias.
- Testes de validação e geração cobrem campos ausentes, dados contraditórios e separação entre observação e recomendação.
- Revisão humana do Entregável 4 e dos demais documentos afetados, sem declarar eficácia de hardening não executado.

**Rastreabilidade:** F1-06 absorvido por F1-29, RF-052, ADR-0018 D-2/D-3 e ADR-0023.

### Estado em 2026-11-23

Novos itens exigem pré/pós-condição, verificação, `source` e `confidence`; o formulário começa em `NOT_VERIFIED`/`LOW`, e registros legados sem os dois últimos campos continuam legíveis. `EVD-059` cobre os testes e o E2E integrados. Nenhum executor foi introduzido. A revisão humana do Entregável 4, a eficácia em clientes e o aceite de produção permanecem fora desse PASS.

## 7. Fatia 6: expansão controlada das regras EVTX

### Base e candidatos, não requisitos aprovados

- **REPO-FACT:** o conjunto atual é `2026.11.1`, com 19 regras em `docs/DETECTION_RULES.md`.
- **REPO-FACT:** Sysmon 10 já aparece na normalização da ADR-0025, mas isso não prova a existência de uma regra LSASS específica.
- **PROPOSAL:** avaliar separadamente Sysmon 10/LSASS (`T1003.001`), alteração de proteção do Defender (`T1562.001`) e transferência por CertUtil/BITSAdmin (`T1105`). Nenhuma dessas técnicas está aprovada nem implementada por constar aqui.
- **PROPOSAL:** não reservar IDs de regra neste documento. Na implementação, verificar unicidade e seguir `.jcode/skills/dfir-detect/SKILL.md`.

### Sequência por regra

1. Registrar a origem na demanda, o comportamento observável, provedor, Event ID, campos brutos, campos normalizados, auditoria necessária e falsos positivos plausíveis.
2. Implementar uma família por commit em `engine/src/detect.rs`, usando campos opcionais sem inferência. Campo necessário ausente impede o disparo.
3. Manter saída somente como candidato `PENDING`, confiança automática máxima `MEDIUM`, limitações explícitas e decisão humana individual.
4. Adicionar casos positivos, negativos próximos, fronteira, robustez e determinismo. Exemplos mínimos:
   - Sysmon 10: separar alvo LSASS de outro processo, provedor incorreto e `GrantedAccess` ausente; não declarar dump concluído.
   - Defender: separar comando de alteração de simples leitura de preferência e cobrir ofuscação como limitação; não declarar ausência de proteção pelo texto isolado.
   - CertUtil/BITSAdmin: separar transferência de usos administrativos como hash ou listagem; cobrir delimitadores e não afirmar download concluído sem telemetria correspondente.
5. Se a normalização mudar, alterar `protocol/src/normalize.rs` e testes compartilhados. Se faltar contrato novo, parar e atualizar especificação/schema antes da regra.
6. Alterar `RULE_SET_VERSION` ao criar/remover regra ou mudar limiar, avaliar `candidate_key` e decisões existentes, gerar o catálogo real com `cargo run --locked -p dfir-engine -- --list-rules` e regenerar `docs/DETECTION_RULES.md` sem redirecionar JSON sobre Markdown.

### Aceite e gates

- Cada regra tem rastreabilidade, metadados ATT&CK revisados, limitações e testes de presença e ausência.
- Reexecução preserva chaves e decisões, agrupamentos diferentes não colidem e os tetos de candidatos/referências/IOCs permanecem.
- `cargo test --locked -p dfir-engine detect::tests`; se aplicável, testes de normalização e `service::detection`; depois `just verify`, `just doc-gate` e E2E com fixture que realmente exercita o sinal.
- Não incluir CSV/JSONL, rede, SIEM, e-mail/identidade, YARA-X ou Sigma nesta fatia. Esses itens exigem outra autoridade e alguns permanecem adiados por ADR-0025.

## 8. Roteiro comum de validação e aceite

### Por mudança de código

1. Identificar a fatia e a autoridade antes de editar; manter commit pequeno e vinculado ao diff real.
2. Executar testes focados que demonstrem a pós-condição, não somente exit 0.
3. Executar `just verify` e `just doc-gate` no HEAD final da fatia.
4. Quando o fluxo real for afetado, executar `just e2e tests/fixtures/senai-demo.evtx` e confirmar que o cenário exercita a mudança.
5. Se o lock, dependências ou advisories forem afetados, executar `just supply-chain` e revisar exceções. Não instalar tooling implicitamente.
6. Confirmar CI no commit exato por `gh`, revisar jobs e diagnósticos, e registrar nova evidência somente depois que arquivo e resultado existirem.
7. Atualizar somente documentos afetados e a RTM quando houver mudança real de estado. Nenhum item deste plano herda `PASS` de evidências anteriores.

### Bloqueadores reais de release, separados das propostas

| Pendência | Estado factual | Critério de saída |
|---|---|---|
| Windows x86_64, F1-22 | `PENDING-HUMAN`; `WVR-001` até 2026-12-31 não é verificação | `just doctor`, `just verify`, E2E e `engine-smoke` nativos; registrar `SANDBOX_NOT_FULLY_APPLIED` sem alegar confinamento inexistente |
| Linux aarch64 | `PENDING-HUMAN`; `WVR-002` até 2026-12-31 não é verificação | Executar nativamente os mesmos gates e testes de sandbox, registrando arquitetura, kernel, Landlock e seccomp |
| Incidente real autorizado e sanitizado | Não substituído por DEMO, testes sintéticos ou datasets públicos | Preservar original, registrar proveniência e executar fluxo protegido; somente métricas sanitizadas entram na evidência |
| Fluxo nativo X11 | `PASS` delimitado em Linux x86_64 por `EVD-050`/`EVD-060`, incluindo seleção Zenity; outros backends, AT-SPI, resize e latência não promovidos | Manter o smoke pós-condicionado ao mudar a UI e executar separadamente qualquer limite residual que se torne requisito de aceite |
| Fechamento do RC, F1-12 | Bloqueado pelas pendências externas; o fluxo X11 delimitado não é aceite de produção | Resolver ou aceitar formalmente cada pendência antes de qualquer `ACCEPTED` final |

Essas pendências seguem `docs/FASE1_BACKLOG.md`, `docs/RELEASE_GATE.md` e `docs/EVIDENCE_REGISTER.md`. As melhorias propostas não as fecham, e a ausência de ambiente não transforma uma proposta em defeito verificado.

## 9. Decisões e pontos de ADR

| Tema | Decisão necessária antes de avançar | Tratamento documental |
|---|---|---|
| Ativação das fatias 2 a 6 | **Decidido em 2026-11-20:** operador ativou as fatias 1–6; execução segue os tickets existentes e evidências próprias | ADR-0030 para os cinco recursos extras; demais fatias sem ADR enquanto preservarem os contratos internos existentes |
| Filtro `ref:` e histórico NIST | Confirmar que o recorte somente leitura atende ao uso | Sem ADR se permanecer nos contratos internos existentes; atualizar documentação afetada |
| Linhagem | Confirmar histórico mais relações de saída como primeiro recorte | ADR somente se surgir novo modelo, mutação, compartilhamento externo ou mudança arquitetural |
| Hardening | Manter somente leitura e recomendações humanas | Sem nova ADR no recorte atual; automação de clientes/Ansible exige decisão e supersessão aplicável |
| Novas regras | Aprovar cada sinal, fixture e falsos positivos | Sem ADR para regra EVTX dentro de RF-050/ADR-0025; ADR e especificação se mudar fonte, arquitetura ou escopo |
| RTK | Aprovar retenção, filtros e eventual hook depois de medir utilidade | Decisão de Engineering/QA Tooling, não requisito de produto; mudança de supply chain segue gates próprios |

## 10. Ferramentas de agente e limites operacionais

- Políticas de swarm, skills e MCPs estão em `AGENTS.md` §17. Uma lane precisa de artefato, orçamento e dono exclusivo; início automático e `swarm spawn` não se combinam. `.jcode/swarm-prompt.md` é o template derivado para novas missões.
- Skills locais: `dfir-improve` para o loop de sessões com mudanças ou gates, `dfir-change` para mudanças, `dfir-verify` para gates e `dfir-detect` para regras EVTX. Carregar skill não é evidência.
- MCPs mantidos: `context7`, `jcodemunch` e `sequential-thinking`, somente quando trouxerem utilidade concreta. Não enviar dados de caso, nomes de evidências, eventos, caminhos sensíveis ou secrets.
- GitHub é consultado exclusivamente pelo comando `gh`, sem GitHub MCP.
- dbhub foi removido e não deve ser recriado sem necessidade demonstrada, banco sintético sanitizado e acesso somente leitura. Acesso SQL direto não substitui invariantes dos serviços.
- Coleta ativa de rede, importação CSV/JSONL e formatos além de EVTX permanecem adiados por ADR-0025. Não reabrir esses itens neste plano.
