<!-- doc-type: decision | status: accepted -->
# ADR-0013 — IOCs e recomendações operacionais (F1-05)

## Status

ACEITA — 2026-08-28

## Contexto

F1-05 fecha o backlog original: extrair IOCs com origem e evidência, normalizar duplicatas sem perder proveniência, e separar recomendações imediatas de estratégicas, cada uma referenciando os achados que a sustentam. Como F1-04, os dois conceitos (IOC, Recomendação) já tinham lugar reservado em F1-01 (`AnalyticalResultType::Ioc`/`Recommendation`).

A decisão de arquitetura real deste ticket é: como "normalizar duplicatas sem perder provenance" quando o mecanismo de versionamento que F1-01/03/04 usam (`supersedes` = substituição) não é o modelo certo para um IOC — ver a próxima seção.

## Decisão

### IOC: duplicata não é substituição, é acumulação

Diferente de Gap/ATT&CK mapping/impacto (onde uma nova versão **substitui** logicamente a anterior — a avaliação estava errada ou desatualizada), ver o mesmo indicador (ex.: o mesmo IP) pela segunda vez não invalida a primeira observação — as duas continuam sendo fatos verdadeiros. `record_ioc_sighting` ainda usa `supersedes` (mesma tabela, mesmo trigger append-only de F1-01), mas com semântica diferente: cada nova chamada busca o agregado atual (se existir), **acrescenta** a nova ocorrência à lista de `sightings` já acumulada, recalcula `first_seen`/`last_seen` a partir do conjunto completo, e insere isso como a nova versão do agregado. Nada é perdido — o `payload` de cada versão é um retrato completo (não incremental) do que se sabe até aquele ponto, e `history_for_correlation_key` mostra como o quadro evoluiu.

`correlation_key = "ioc:{incident}:{tipo}:{valor normalizado}"` — a deduplicação é por `(incidente, tipo, valor normalizado)`. IOCs do mesmo valor em incidentes diferentes **não** são agregados juntos (deliberado: o ticket não pede correlação cruzada entre incidentes; um registro de threat intel "conhecido malicioso" entre incidentes é uma capacidade maior, fora de escopo aqui).

### Validação de IOC é estrutural, não um serviço de reputação

`normalize_ioc_value` usa `std::net::IpAddr` da biblioteca padrão para validar IPs (sem dependência nova) e checagens de forma leves para os demais tipos (comprimento hexadecimal para hashes; presença de `@` para e-mail; esquema reconhecido para URL; minúsculas/trim para domínio/hostname). Não há parser RFC-completo de e-mail (RFC 5322) nem de URL (RFC 3986), e nenhuma consulta a serviço externo de reputação — o objetivo é rejeitar valores obviamente malformados e normalizar para deduplicação, não validar contra uma lista de ameaças conhecidas. Mesma disciplina de "necessidade demonstrada" das ADRs 0011/0012, aplicada aqui à profundidade de validação.

### Recomendações: sem chave natural de deduplicação

Diferente de IOCs/gaps, uma recomendação não tem um valor "canônico" que permita detectar duplicatas automaticamente. `insert_recommendation` aceita um `supersedes` **explícito e opcional**: se o chamador passa o id de uma recomendação existente, a nova a substitui (mesmo `correlation_key`, histórico preservado); caso contrário, uma `correlation_key` nova e única é gerada. Isso é uma escolha deliberada — automatizar isso "por assunto" exigiria um julgamento (duas recomendações são "a mesma"?) mais frágil do que deixar o analista decidir explicitamente.

`supporting_result_ids` (não vazio, obrigatório) é validado contra resultados analíticos existentes antes do insert e vira uma relação `RelationType::DerivedFrom` para cada um — reusa o mesmo mecanismo de relações de F1-01/04, sem nada novo.

### Exportação reusa a mesma função das listagens

RF-022 ("Exportação preserva a mesma origem usada na UI") é implementado literalmente: `export_incident_findings` chama a mesma função `current_results_for_incident` que `list_iocs`/`list_recommendations` usam — não há um segundo caminho de código reconstruindo os dados de forma independente, o que eliminaria por completo o risco de divergência entre o que a UI mostra e o que a exportação produz.

## Alternativas consideradas

- **Tratar cada sighting de IOC como uma linha independente (sem agregação), deixando a UI agrupar por valor normalizado.** Rejeitada: "duplicatas são normalizadas" no ticket pede explicitamente que a duplicação seja tratada no modelo, não empurrada para a camada de apresentação — além disso, sem uma linha "atual" agregada, calcular first_seen/last_seen exigiria varrer todo o histórico a cada leitura.
- **Detectar duplicata de recomendação automaticamente por similaridade de título/descrição.** Rejeitada: julgamento de similaridade textual é frágil e arriscaria mesclar recomendações genuinamente distintas ou deixar de mesclar reformulações da mesma — pedir que o analista decida explicitamente é mais simples e mais correto.
- **Embutir uma lista de reputação de IPs/domínios conhecidos maliciosos para enriquecer IOCs automaticamente.** Rejeitada: exigiria uma fonte de dados externa mantida e atualizada — fora do escopo de "correlação funciona offline" do próprio ticket, e um problema de manutenção de dados, não de arquitetura.

## Consequências

Positivas:
- IOCs vistos repetidamente ficam com uma visão única e legível (`sighting_count`, `first_seen`/`last_seen`) sem perder nenhum registro individual de origem.
- Exportação e UI são estruturalmente impossíveis de divergir, por reusarem a mesma função.

Negativas / dívida técnica assumida:
- Validação de e-mail/URL é estrutural, não RFC-completa — um valor tecnicamente inválido mas superficialmente parecido com um IOC real poderia passar. Aceitável para o objetivo de deduplicação/normalização, não de validação de segurança.
- Sem correlação cruzada entre incidentes — o mesmo IOC em dois incidentes diferentes hoje aparece como dois registros independentes, sem nenhum sinal de que é "o mesmo indicador visto de novo". Um registro de threat intel entre incidentes é uma capacidade de escopo maior, deixada para um ticket futuro caso demonstrada necessidade.

## Evidência de execução

Nenhum gate Rust executado neste ambiente (sem `cargo`/`rustc`). 13 testes escritos em `src/persistence/ioc_recommendations.rs`: validação de IP/domínio/hash/e-mail/URL/valor vazio (6 testes puros, sem banco), agregação de sightings preservando provenance (incluindo verificação de que ambas as versões continuam no histórico), não-agregação entre incidentes diferentes, exigência de ao menos um achado de suporte, rejeição de achado de suporte inexistente, ligação correta via `DerivedFrom`, e resupersessão de recomendação mantendo o mesmo grupo de correlação. Revisado manualmente linha a linha contra as assinaturas reais de `analytical_results.rs`, mas não compilado.

## Adendo — 2026-09-21 — teto de `sightings[]` no payload do IOC

**Problema**: o parágrafo "IOC: duplicata não é substituição, é acumulação" acima acrescentava cada ocorrência ao array `sightings` sem limite. Um indicador visto centenas de vezes produziria uma linha cada vez maior — e, como cada versão é um retrato completo e a tabela é append-only, o armazenamento total crescia de forma quadrática (soma de tamanhos que crescem a cada versão).

**Decisão**: o payload guarda no máximo `MAX_STORED_SIGHTINGS = 100` ocorrências — as mais recentes por `observed_at`, com desempate pela ordem de inserção — e um contador do total:

- `sighting_count` passa a ser o **total real** de ocorrências já registradas (antes era `sightings.len()`), inclusive as que saíram do array.
- `sightings_dropped = sighting_count − sightings.len()` torna o corte explícito para quem lê o payload (a interface mostra "N ocorrências mais antigas fora do detalhe").
- `first_seen` e `last_seen` continuam cobrindo **todas** as ocorrências: são carregados do payload anterior (mínimo e máximo com a nova ocorrência), e não recalculados a partir do array já aparado — senão `first_seen` avançaria toda vez que a ocorrência mais antiga fosse descartada.
- Payloads gravados antes do teto (sem `sighting_count`/`sightings_dropped`, possivelmente com mais de 100 ocorrências) são aceitos: o total parte de `sightings.len()` e o array é aparado no próximo registro.

**O que não se perde**: as ocorrências descartadas do payload vigente continuam nas versões anteriores da mesma cadeia `supersedes` (a tabela é append-only, com trigger), acessíveis por `history_for_correlation_key`. A frase "nada é perdido" do texto original continua valendo para o histórico; o que o teto limita é o **tamanho de cada linha**. O número de linhas segue crescendo em uma por ocorrência, então o armazenamento total passa de quadrático a linear (≈ 100 ocorrências por linha depois do teto).

**Alternativas rejeitadas**: (a) gravar apenas o delta em cada versão — quebra a propriedade "cada versão é um retrato completo" de F1-01 e complica toda leitura; (b) tabela separada de sightings — exigiria migration e mudaria o modelo analítico comum (ADR-0008) só para tratar um caso de cauda longa; (c) N configurável por variável de ambiente — sem demanda demonstrada, é só uma constante a mais para errar.

**Limite conhecido**: `observed_at` é comparado como texto. Datas RFC 3339 em UTC com o mesmo formato ordenam corretamente; formatos misturados (`Z` versus `+00:00`) podem inverter a ordem dentro do mesmo segundo. Documentado em `docs/API.md`.

Testes: `src/persistence/ioc_recommendations.rs` — agregação abaixo do teto, `first_seen` preservado após descarte, agregado do primeiro registro, e um teste de banco que registra além do teto e confere linha limitada, total correto e histórico completo.
