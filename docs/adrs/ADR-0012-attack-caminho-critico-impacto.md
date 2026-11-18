<!-- doc-type: decision | status: accepted -->
# ADR-0012 — ATT&CK mapping, caminho crítico e matriz de impacto (F1-04)

## Status

ACEITA — 2026-08-28

## Contexto

F1-04 pede três capacidades relacionadas (seções 14-16 da spec canônica): mapeamento de eventos/achados para técnicas ATT&CK, um grafo de caminho crítico do incidente, e uma matriz de impacto — com um requisito transversal explícito no ticket: "Relações não suportadas por evidência são marcadas como incerteza, não como fato."

Os três já tinham lugar reservado desde F1-01 (`AnalyticalResultType::AttackMapping`/`CriticalPathLink`/`ImpactAssessment`) e o mecanismo de encadeamento (`RelationType::SequenceNext`) também — F1-04 é, em boa parte, "ativar" infraestrutura que F1-01 já previu.

## Decisão

### Sem tabelas novas — os três são achados, não dados de entrada

Diferente de F1-03 (Current/Target Profile, que são fatos organizacionais declarados), um mapping ATT&CK, um estágio de caminho crítico e uma avaliação de impacto são **julgamentos analíticos** de um analista sobre a evidência — cabem diretamente em `analytical_results` (F1-01), sem necessidade de tabelas próprias. Novo módulo `src/persistence/attack_impact.rs`, sem migration nova.

### Catálogo de técnicas ATT&CK: mesma decisão de F1-03 (ADR-0011)

As 14 táticas oficiais do MITRE ATT&CK (Enterprise) são uma estrutura estável e pequena — hardcoded como enum `AttackTactic`. O catálogo de **técnicas** (~600+, atualizado periodicamente pelo MITRE) não é embutido: `technique_id`/`sub_technique_id` são validados apenas quanto à forma (`T####` / `T####.###`), pela mesma razão de risco de transcrição silenciosa já documentada na ADR-0011 — um catálogo de centenas de itens usado num relatório de incidente real, se transcrito de memória incorretamente, não seria pego por nenhum teste automatizado.

### "Incerteza, não fato": confidence HIGH/CONFIRMED exige uma `basis` não-vazia

Implementado como uma validação explícita (`require_evidence_for_high_confidence`), compartilhada entre ATT&CK mapping e estágios de caminho crítico: sem uma referência de evidência/raciocínio (`basis`) preenchida, a inserção com confidence `HIGH` ou `CONFIRMED` é **rejeitada** — não apenas desencorajada. O analista é forçado a registrar `LOW`/`MEDIUM` quando a evidência é insuficiente. Além disso, o payload persistido grava `evidence_status: "GAP" | "DOCUMENTED"` explicitamente, para que a seção 15 ("ausência de evidência deve ser exibida como lacuna, não preenchida por inferência silenciosa") valha na apresentação, não só na validação de entrada.

### Caminho crítico: estágios são resultados; a sequência é uma relação

Cada estágio do caminho crítico é um `AnalyticalResultType::CriticalPathLink` independente; a ordem entre eles é expressa via `RelationType::SequenceNext` (`analytical_result_relations`, já existente desde F1-01) — não uma lista/array serializada dentro de um payload. Isso permite grafos com ramificação (mais de um `SEQUENCE_NEXT` saindo do mesmo estágio), como a seção 15 pede ("grafo temporal/causal"), não apenas uma cadeia linear.

Um pequeno helper genérico, `analytical_results::insert_relation`, foi adicionado a F1-01 para isso (a única forma anterior de criar uma relação era via o parâmetro `related` no momento do insert original — insuficiente aqui, já que o próximo estágio de uma sequência tipicamente ainda não existe quando o estágio atual é criado).

`link_critical_path_stages` valida que ambos os lados são de fato `CRITICAL_PATH_LINK` antes de criar a relação — impede, por exemplo, ligar um estágio a um mapping ATT&CK por engano.

**Limitação assumida**: estágios não são versionados por `supersedes` de forma integrada com suas relações — corrigir a descrição de um estágio (mesmo `sequence_index`) cria uma nova linha que substitui a anterior via `correlation_key`, mas as relações `SEQUENCE_NEXT` continuam apontando para o id da linha antiga até serem recriadas manualmente. Aceitável para o MVP (recriar a relação é uma chamada extra); registrado aqui para não ser redescoberto como bug.

### Matriz de impacto: uma linha versionada por incidente

`correlation_key = "impact-assessment:{incident_id}"` — um incidente tem uma avaliação de impacto que evolui (nova submissão substitui a anterior via `supersedes`), não várias em paralelo. `financial_impact` é `Option<&str>` (texto livre, nunca um número fabricado) — `None` quando não há dados, satisfazendo literalmente "Não fabricar valores financeiros quando não houver dados" (seção 16). `financial_impact_known: bool` é gravado explicitamente no payload para que a ausência de dado seja uma informação visível, não apenas um campo nulo que uma UI possa interpretar como zero.

### Endpoint HTTP do caminho crítico monta o grafo com N+1 consultas

`GET /api/incidents/{id}/critical-path` busca os estágios vigentes e, para cada um, consulta suas relações de saída via `related_to` — N+1 consultas, não uma única query com JOIN. Aceitável para a quantidade de estágios esperada num incidente (dezenas, não milhares); documentado como simplificação deliberada.

## Alternativas consideradas

- **Um catálogo oficial de técnicas ATT&CK embutido**, para poder validar `technique_id` contra nomes/descrições reais. Rejeitada pela mesma razão da ADR-0011: risco de transcrição incorreta maior que o benefício, e o ticket não exige isso explicitamente.
- **Sequência do caminho crítico como um array de ids dentro do payload de um único resultado "caminho crítico".** Rejeitada: colide com o modelo de resultados versionados e auditáveis de F1-01 (cada estágio precisa de sua própria evidência/confiança/proveniência) e não suporta ramificação sem reinventar uma segunda camada de relações dentro do JSON.
- **Rejeitar silenciosamente (em vez de erro) uma tentativa de HIGH/CONFIRMED sem evidência, rebaixando automaticamente para MEDIUM.** Rejeitada: rebaixar silenciosamente esconderia do analista que o sistema alterou o que ele declarou — um erro explícito é mais alinhado com "não inventar fatos" do que uma correção silenciosa.

## Consequências

Positivas:
- Nenhuma migration nova — todo o modelo de F1-04 já existia em F1-01, validando o desenho original.
- `insert_relation`, agora genérico em F1-01, fica disponível para F1-05 também (uma recomendação referenciando os achados que a sustenta já usava o parâmetro `related` no insert original; agora há também a opção de ligar depois).

Negativas / dívida técnica assumida:
- Ver limitações documentadas acima (estágios não re-vinculam relações automaticamente ao serem substituídos; leitura do grafo é N+1).

## Evidência de execução

Nenhum gate Rust executado neste ambiente (sem `cargo`/`rustc`). 15 testes escritos em `src/persistence/attack_impact.rs` (validação de forma de technique id, round-trip dos enums, rejeição de technique id inválido, rejeição de HIGH/CONFIRMED sem evidência, aceitação de LOW sem evidência com `evidence_status: GAP` verificado no payload persistido, resupersessão de mapping e de impact assessment, criação e ligação de estágios de caminho crítico, rejeição de ligação envolvendo um resultado que não é `CRITICAL_PATH_LINK`, e não-fabricação de impacto financeiro) mais 2 novos testes em `analytical_results.rs` para `insert_relation`. Revisado manualmente linha a linha contra as assinaturas reais de `analytical_results.rs`, mas não compilado.
