<!-- doc-type: decision | status: accepted -->
# ADR-0011 — NIST Current/Target Profile e Gap Analysis reproduzível (F1-03)

## Status

ACEITA — 2026-08-27

## Contexto

F1-03 pede Current/Target Profile persistidos com versão/fonte e uma Gap Analysis "reproduzível a partir dos dados... sem Internet" (RF-010), com cada gap tendo origem/contexto para auditoria e sem que alterações de profile apaguem versões anteriores.

Duas decisões de arquitetura precisavam ser tomadas: (1) o que persistir sobre o catálogo oficial do NIST CSF 2.0, e (2) como o Gap se relaciona com o modelo comum já estabelecido em F1-01.

## Decisão

### O catálogo oficial de subcategorias do CSF 2.0 não é embutido

Diferente do ruleset YARA (F1-02, ADR-0010), este ticket **não** embute uma cópia do catálogo oficial de ~106 subcategorias do NIST CSF 2.0. Transcrever esse catálogo de memória para uma ferramenta de avaliação de conformidade real é um risco de erro silencioso — um código ou descrição levemente incorreta não seria pega por nenhum teste automatizado, e o impacto de um erro aqui (uma subcategoria mal descrita numa avaliação de conformidade) é mais sério do que em uma regra de detecção heurística. Em vez disso:

- `subcategory_id` é validado apenas quanto à forma estrutural `FUNCTION.CATEGORY-NN` (ex.: `GV.OC-01`) com FUNCTION em `{GV,ID,PR,DE,RS,RC}` — essa estrutura de 6 Functions é estável e está no texto da própria seção 13 da spec canônica, não é algo que dependa de uma transcrição detalhada arriscada.
- `subcategory_label` (a descrição) é fornecida pelo analista, a partir da publicação oficial do NIST que ele já deveria ter em mãos para conduzir a avaliação.

### Current/Target Profile são dados de entrada; Gap é derivado

`nist_profiles`/`nist_profile_entries` (novas tabelas, migration `0008`) guardam o que um analista **declara** sobre o estado da organização — não são "achados" no sentido de F1-01. O Gap, por outro lado, é literalmente o que F1-01 já havia reservado como `AnalyticalResultType::NistGap`: um resultado **derivado**, calculado por uma função pura (`compute_gaps`) que compara as entradas de um profile CURRENT contra um profile TARGET, sem nenhuma chamada de rede, e persistido com confiança/proveniência/histórico via a infraestrutura de F1-01.

Isso mantém a mesma separação achados-vs-inputs que F1-01 já havia estabelecido ao deixar CurrentProfile/TargetProfile explicitamente fora do seu próprio escopo.

### Versionamento: mesma disciplina append-only de F1-01, duas tabelas

Um profile inteiro (CURRENT ou TARGET) é versionado como uma unidade: uma nova versão é uma nova linha em `nist_profiles` com `supersedes` apontando para a anterior, carregando seu próprio conjunto de linhas em `nist_profile_entries` — a versão anterior nunca é alterada ou apagada, satisfazendo literalmente "Alterações de profile não apagam versões anteriores". Ambas as tabelas têm os mesmos triggers `BEFORE UPDATE`/`BEFORE DELETE` de `analytical_results`.

`insert_profile` detecta e liga o `supersedes` automaticamente (o chamador não precisa saber qual era a versão anterior) — mas nunca sobrescreve; só insere.

### Tier é contexto do profile, nunca por subcategoria

A seção 13.2 é explícita: "Não representar Tier como score oficial por Function, Category ou Subcategory". `tier`/`tier_justification` vivem em `nist_profiles` (um valor por versão de profile inteiro), e a coluna nem existe em `nist_profile_entries`.

### Algoritmo do Gap (determinístico, puro, testável isoladamente)

`implementation_status` tem uma ordem operacional (não-oficial do NIST, convenção do produto): `NOT_IMPLEMENTED(0) < PARTIALLY_IMPLEMENTED(1) < IMPLEMENTED(2)`; `NOT_ASSESSED`/`NOT_APPLICABLE` ficam fora dessa escala. Para cada subcategoria do TARGET:

- `NOT_APPLICABLE` em qualquer um dos dois lados → sem gap;
- ausência de entrada correspondente no CURRENT → tratada como `NOT_ASSESSED` (gap de prioridade `HIGH` — estado desconhecido é tratado conservadoramente);
- `current_rank < target_rank` → gap; prioridade `HIGH` se o salto for de 2 posições (ex.: NOT_IMPLEMENTED→IMPLEMENTED), `MEDIUM` caso contrário;
- subcategorias presentes só no CURRENT (sem entrada no TARGET) não geram gap — o target não as elegeu como relevantes.

`priority` também é usado como `risk_level` no payload persistido — não é uma metodologia de risco completa (isso é impacto/RF-020, fora do escopo de F1-03), apenas uma aproximação determinística até uma avaliação de impacto mais rica existir (F1-04).

Confiança do gap é a mais conservadora entre a confiança declarada no CURRENT e no TARGET (`MEDIUM` como padrão documentado quando nenhum dos dois declarou).

### Recomendação fica fora (F1-05)

A seção 13.1 lista "recomendação" como um campo conceitual de Gap Analysis, mas isso é literalmente o trabalho de F1-05 (`AnalyticalResultType::Recommendation`). F1-03 não gera recomendações — um resultado `NistGap` pode ser referenciado por uma `Recommendation` futura via `RelationType::DerivedFrom` (mecanismo já construído em F1-01), quando F1-05 for implementado.

### Pré-requisito descoberto: não existia CRUD de incidents

Implementar F1-03 expôs uma lacuna: não havia nenhuma forma de criar um `incident` via a API (a tabela existe desde a Fase 0, mas sem rota HTTP). Como `NistGap` exige `incident_id` (decisão já tomada em F1-01) e um profile é naturalmente escopado a um incidente, um CRUD mínimo de incidents (`POST/GET /api/incidents`, `GET /api/incidents/{id}`) foi adicionado nesta mesma rodada — não é escopo inflado, é uma dependência real que só ficou visível ao implementar o ticket.

## Alternativas consideradas

- **Embutir o catálogo oficial completo do CSF 2.0** (como foi feito com o ruleset YARA). Rejeitada: risco de transcrição incorreta em conteúdo usado para conformidade real; o ticket não exige isso explicitamente ("categorias/subcategorias relevantes podem ser comparadas" não implica um catálogo pré-carregado).
- **Modelar Current/Target Profile como `AnalyticalResultType` novos em vez de tabelas próprias.** Rejeitada: profiles são dados declarados por um analista (entrada), não achados derivados pelo sistema — a mesma distinção que a própria ADR de F1-01 já havia registrado ao deixar isso de fora do seu escopo.
- **Calcular o Gap sob demanda, sem persistir.** Rejeitada: RF-010 pede resultados "reproduzíveis" com histórico auditável — persistir cada execução (com `supersedes` entre execuções sucessivas) é o que torna isso auditável, não apenas calculável.

## Consequências

Positivas:
- Gap Analysis é literalmente uma função pura testável sem banco de dados (`compute_gaps`), o que dá confiança alta em RF-010 mesmo sem poder rodar os testes neste ambiente.
- CRUD de incidents, antes ausente, agora existe e desbloqueia qualquer ticket futuro que precise de contexto de incidente (F1-04, F1-05).

Negativas / dívida técnica assumida:
- `get_nist_gap_analysis` (leitura dos gaps vigentes) depende de `analytical_results::list_by_incident`, que trunca em 500 resultados mais recentes de qualquer tipo para o incidente — um incidente com muitos outros resultados analíticos acumulados poderia, em teoria, deslocar uma linha de supersessão de gap para fora dessa janela. Aceitável para o volume esperado no MVP; documentado no código.
- Sem o catálogo oficial embutido, nada impede um analista de digitar um `subcategory_label` inconsistente com o `subcategory_id` (ex.: rotular `GV.OC-01` com a descrição de outra subcategoria) — a validação estrutural não pega esse tipo de erro humano. Aceito como trade-off pela razão já explicada.

## Evidência de execução

Nenhum gate Rust executado neste ambiente (sem `cargo`/`rustc`). `compute_gaps` e suas variantes têm 7 testes puros (sem banco) cobrindo: gap real, sem gap, entrada ausente tratada como NOT_ASSESSED, `NOT_APPLICABLE` em ambos os lados, subcategoria só no current, determinismo entre chamadas repetidas, e confiança combinada. Mais 5 testes de integração (SQLite em arquivo temporário, mesmo padrão de F1-01/F1-02): rejeição de profile vazio, rejeição de subcategory_id inválido, versionamento com histórico preservado, exigência de ambos os profiles antes do gap analysis, e persistência+resupersessão de gaps como `analytical_results`. Revisado manualmente linha a linha contra as assinaturas reais de `analytical_results.rs`/`audit.rs`/`web/mod.rs`, mas não compilado.
