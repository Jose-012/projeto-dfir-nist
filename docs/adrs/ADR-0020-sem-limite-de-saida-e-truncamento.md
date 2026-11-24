<!-- doc-type: decision | status: accepted -->
# ADR-0020 — Sem limite de saída e sem truncamento silencioso

## Status

ACEITA — 2026-10-04, a pedido explícito do operador ("remova o limite de saída e truncamento"). Decisão de arquitetura (nível 3); não altera RF. Complementa ADR-0010 (YARA) e ADR-0016 (engine/protocolo).

## Contexto

O primeiro teste ponta a ponta com engine real (ADR-0019, EVTX real de 2,1 MB, 2261 eventos) mostrou que os limites existentes cortavam ou derrubavam análises legítimas:

- a saída do engine ocupa ~1,2 KB por evento; o teto de 4 MiB (`MAX_OUTPUT_BYTES`) e de 100 mil itens (`MAX_ITEMS`) comportava ~3,5 mil eventos, então EVTX de produção falhavam com `OUTPUT_LIMIT` (AUD-05);
- a timeline mostrava 1000 eventos (`MAX_TIMELINE_EVENTS`) e lia no máximo 2000 linhas do `results.jsonl` (`MAX_RESULT_LINES`) (AUD-06);
- cada linha JSONL tinha teto de 64 KiB: um único registro EVTX com JSON maior derrubava o job inteiro;
- listagens de resultados analíticos cortavam em 500 linhas (`MAX_LISTED_RESULTS`) e o catálogo de evidências em 100 (AUD-08);
- a persistência de YARA gravava no máximo 1000 matches (`MAX_PERSISTED_YARA_MATCHES`);
- o YARA-X, com `max_scan_size(256 MiB)`, **varria só os primeiros 256 MiB em silêncio** (confirmado no código da crate): um marcador depois disso nunca seria visto;
- o scanner descartava em silêncio as ocorrências de um padrão além da centésima.

## Decisão

1. **Saída do engine sem limite prático.** `MAX_OUTPUT_BYTES` e `MAX_ITEMS` passam a ser o maior inteiro seguro de JSON (2^53 − 1), enviado por padrão pelo app. Um teto menor só vale se o operador pedir (`dfir-app analyze --max-output-bytes N --max-items N`); nesse caso o job falha com `OUTPUT_LIMIT` ou `ITEM_LIMIT`, nunca trunca em silêncio. (Detalhe técnico: `i64::MAX` foi tentado primeiro e quebrava toda análise, porque os limites entram na cadeia de auditoria e a canonicalização RFC 8785 rejeita inteiros acima de 2^53; há teste de regressão.)
2. **Moldura do protocolo:** `MAX_LINE_BYTES` (e o padrão de `DFIR_ENGINE_MAX_LINE_BYTES`) sobe de 64 KiB para 8 MiB. Continua sendo limite de moldura, não de saída: o engine fecha cada lote antes de passar dele.
3. **Timeline completa.** `job_results` lê o `results.jsonl` inteiro em fluxo (converte cada item em `TimelineEvent` e descarta o JSON bruto), ordena por UTC e devolve **todos** os eventos. O campo `truncated` de `ResultsResponse` foi removido. A GUI cacheia os índices filtrados e a tabela já era virtualizada, então milhares de eventos não custam por quadro.
4. **Listagens completas:** sem `LIMIT` em `list_by_evidence`, `list_by_incident` e `list_evidence`; nova `list_current_by_evidence` devolve só os resultados vigentes (não substituídos).
5. **YARA:** sem `max_scan_size`; todo match é persistido; reexecutar o scan agora **substitui** o resultado vigente da mesma chave (`supersedes`) em vez de duplicá-lo. As ocorrências por padrão são contadas com exatidão (teto do scanner 1.000.000); o item lista até 5.000 ocorrências por padrão e, se houver mais, traz `pattern_match_counts` com o total exato e `matches_truncated = true` — amostragem **explícita e sinalizada**, necessária para que uma linha JSONL não carregue milhões de offsets.
6. **Recuperação na partida:** jobs deixados `PENDING`/`PROCESSING` por um app que fechou no meio da análise viram `FAILED` (`ORPHANED`) com auditoria (`recover_orphaned_jobs`), desbloqueando o descarte da evidência.

## O que NÃO mudou (e por quê)

- **Tempo máximo do engine** (`DFIR_ENGINE_TIMEOUT_SECONDS`, 300 s por padrão, até 3600 s) e memória virtual do processo (`RLIMIT_AS`): sem eles um arquivo hostil trava ou esgota o host. Um EVTX muito grande pode estourar o tempo; o erro é explícito (`TIMEOUT`), e o limite é configurável.
- **Tamanho máximo de importação** (`DFIR_MAX_UPLOAD_BYTES`): limite de entrada, não de saída.
- **Limite de profundidade JSON** e **erros de registro tolerados** (`MAX_RECORD_ERRORS`): proteção do parser.
- **Retenção de 100 ocorrências recentes por IOC** (`MAX_STORED_SIGHTINGS`): o `sighting_count` continua exato.

## Consequências e riscos aceitos

- **Disco:** sem teto, um arquivo hostil cuja expansão para JSON seja desproporcional pode ocupar bastante disco antes do timeout. Mitigação parcial: o timeout; mitigação completa: checagem de espaço livre (F1-10, ainda pendente).
- **Memória na leitura da timeline:** proporcional ao número de eventos (não ao tamanho do JSON). Para dezenas de milhões de eventos isso deixa de ser razoável; a solução definitiva é persistir os eventos no banco e paginar (F1-18, ainda pendente). Até lá, um EVTX com milhões de registros pode esgotar a memória da GUI.
- **Achado aberto relacionado (AUD-36):** um EVTX **truncado** termina `COMPLETED` com menos eventos (1587 de 2261 no teste) e **sem aviso**, porque erros do parser não chegam ao app (AUD-04, F1-17). Remover limites não cria esse problema, mas torna mais importante resolvê-lo.

## Atualização — 2026-10-11

Os riscos listados em "Consequências e riscos aceitos" foram tratados pela **ADR-0021**: memória da timeline (índice e paginação no SQLite), disco (`DFIR_MIN_FREE_DISK_BYTES`) e o achado AUD-36 (avisos `EVTX_TRUNCATED`/`EVTX_RECORDS_MISSING`). Permanecem como risco: Windows sem medição de disco e escala acima de dezenas de milhões de eventos.

## Atualização — 2026-11-24

O item 6 da decisão é superseded pela [**ADR-0031**](ADR-0031-ownership-de-jobs-por-lock.md) somente quanto à identificação de órfãos: um lock exclusivo por job preserva processos vivos de outra instância, e a transição/auditoria condicional recupera somente jobs sem dono. A saída sem teto total e a moldura de 8 MiB permanecem inalteradas.
