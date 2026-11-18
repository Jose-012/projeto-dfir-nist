<!-- doc-type: decision | status: accepted -->
# ADR-0018 — Realinhamento à demanda original (SENAI 12412): poda de escopo e adições necessárias

## Status

ACEITA — 2026-10-02. Supersede parcialmente: a classificação de requisitos de ADR-0017 (ver adendo dela), `RF-012`, `RF-025`, `RF-026`, `RF-032`, `RF-041`, `RF-045` e `RF-046` da especificação v6.4.5 (versões substitutas na v6.5.0).

## Contexto

Uma auditoria completa (documentação + código, 2026-10-01/02; ver `docs/archive/REVIEW_2026-10-02-auditoria-demanda.md`) comparou o projeto com o texto **original** da demanda SAGA SENAI 12412 (agora versionado em `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md`).

Constatações que motivam esta decisão:

1. A demanda é um engajamento de avaliação com dois blocos: (A) investigar o incidente e (B) transformar a investigação em plano de fortalecimento. Seis entregáveis, cinco deles do bloco B. O projeto investiu quase tudo no bloco A (preservação, custódia, engine isolado) e deixou o bloco B sem modelo de dados.
2. Parte do backlog não tem origem na demanda e adiciona superfície de ataque ou custo sem retorno para os entregáveis: pacote portátil `.dfirpkg`, atualização offline assinada, retry/backoff/DLQ para parse determinístico local, benchmark, geração de playbooks Ansible e duplicação de relatórios em "perfis" executivo/técnico.
3. A restrição RB-004 foi lida como "preferir open source"; a demanda trata de **orçamento do cliente** para implementar as recomendações (logo, recomendações precisam de prioridade, esforço, custo e fases).
4. O evento "normalizado" não era normalizado (`NormalizedEvent` guarda só `timestamp` + JSON cru), o que impede filtros úteis, detecção, extração de IOC e qualquer fonte de log além de EVTX.
5. A fonte nº 1 de requisitos (a demanda) não estava no repositório.
6. `RF-016` e `RF-044` ainda falavam de XSS/HTTP, obsoletos desde ADR-0017.

## Decisão

### D-1 — Fonte de verdade da demanda entra no repositório

`docs/source/DEMANDA_ORIGINAL_SENAI_12412.md` (transcrição) é a fonte primária de requisitos de negócio. `docs/DEMAND_TRACE.md` mapeia cada item da demanda a RF/ticket/estado. Ambos são mantidos junto com a RTM; item da demanda sem linha em `DEMAND_TRACE.md` é defeito de documentação.

### D-2 — Corte e adiamento (nada é apagado; IDs não são reutilizados)

| Item | Decisão | Motivo |
|---|---|---|
| `.dfirpkg` (`RF-025`, F1-08) | **Adiado para Fase 4** | Não pedido pela demanda; cria uma nova entrada não confiável (zip-slip, bomba de compressão, manifest forjado). A necessidade real (entregar o resultado) é coberta por exportação dos entregáveis + manifest de hashes (F1-07). |
| Atualização offline assinada (`RF-026`, F1-09) | **Adiado para Fase 4** | Engenharia de distribuição de software, sem relação com a demanda. Fica só: backup do banco antes de migration + procedimento manual documentado (`RF-032` reduzido). |
| Retry/backoff, `PERMANENTLY_FAILED`, DLQ (`RF-041`) | **Substituído** por falha inspecionável com código de erro | Reprocessar um parse determinístico de arquivo corrompido não muda o resultado; falha transitória é rara em app local monousuário. Mantidos: recovery na partida, shutdown limpo, checagem de espaço (`RF-042`, `RF-043`). |
| Benchmark `just perf` (`RF-045`) | **Fase 4 / opcional** | Não é requisito de negócio. |
| Perfis executivo/técnico dos relatórios (parte de `RF-046`) | **Removido** | A demanda pede "relatório completo", uma visão. O resumo executivo já é seção do Entregável 1. |
| Playbook Ansible (`RF-012`) | **Fase 4** (conforme o roadmap §38, que já o colocava lá) | A demanda pede **plano** de hardening, não automação. Substituído no MVP por `RF-052` (item de plano estruturado). |
| Assinatura de relatório, âncora extra de custódia, `cargo vet`, Job Object/limite de memória no Windows, grafo SVG do caminho crítico, breadcrumbs, cancelamento de job | **Fora do MVP** (registrados como melhorias futuras, sem ticket) | Custo sem retorno para os entregáveis; `cargo vet` dispensado por ser projeto solo (permanecem `cargo deny` + `cargo audit`); **a dispensa só passa a valer quando F1-15 atualizar `AGENTS.md` §8 e `docs/SUPPLY_CHAIN.md`** — até lá, o requisito antigo continua registrado como `PENDING`. |
| Clauses de XSS/HTTP em `RF-016`/`RF-044` | **Reescritas** | UI nativa (ADR-0017) não tem superfície HTML/HTTP; permanecem injection/traversal de template, caminho e envelope JSONL. |

### D-3 — Adições necessárias (novos RFs 048–055)

| RF | Adição | Origem na demanda |
|---|---|---|
| RF-048 | Evento normalizado com campos estruturados | pré-requisito de filtros, detecção, IOC, multi-fonte |
| RF-049 | Importação estruturada CSV/JSONL (mapeamento declarativo) | "diferentes fontes": SIEM, autenticação, firewall, e-mail/identidade (cenário BEC) |
| RF-050 | Regras determinísticas de detecção → **candidatos** a achado, confirmados pelo analista; extração de IOC com referência ao evento | "vetor inicial, movimentação lateral, escalonamento, exfiltração" |
| RF-051 | Finding (vulnerabilidade/condição explorada), causa-raiz/provável/fatores, lacunas | entregáveis 1 e 2 |
| RF-052 | Item de plano (domínio, prioridade, esforço, custo, fase, referência regulatória/política, pré/pós-condição, verificação) | entregáveis 3, 4, 5; as 7 ações do plano; restrições 4 e 5 |
| RF-053 | Visão NIST CSF por Function (GV/ID/PR/DE/RS/RC), descritiva, sem score oficial | entregável 3; "governança, ativos, proteção, monitoramento, resposta, recuperação" |
| RF-054 | Proveniência da evidência, cobertura declarada de formatos, registro de lacunas/retenção | restrições 1 e 2; spec §4.3 |
| RF-055 | Redação/mascaramento de dados pessoais na exportação | restrição 6 |

A decisão de **detecção determinística no MVP** é desta ADR: a spec (§18/`RF-021`) coloca *exportação e validação de Sigma* na Fase 4, o que continua valendo; o que entra no MVP é apenas um conjunto pequeno e versionado de regras internas sobre eventos normalizados. A decisão de **importador estruturado** amplia deliberadamente a "ESCOPO DE PARSING AUTOMÁTICO DO MVP" da §4.3 (antes: somente EVTX), porque o cenário principal do demo (BEC) usa logs de e-mail/identidade em JSON/CSV e não é demonstrável apenas com EVTX. A regra de honestidade de escopo da §4.3 permanece: a UI declara exatamente o que é analisado e o que é só preservado.

### D-4 — Classificação correta de ADR-0017

A troca para eframe/egui foi **decisão do operador** (2026-09-22), não requisito da demanda; a demanda original não menciona tecnologia de interface. A ADR-0017 recebe adendo corrigindo a classificação; a precedência de `AGENTS.md` §1 não é invocada para essa decisão.

### D-5 — Estado de verificação desta ADR

Esta ADR é documental. As correções de código e lock aplicadas na mesma data estão em `CHANGELOG.md` (6.5.0) e `docs/FASE1_STATUS.md`, com o nível de evidência exato de cada uma.

## Alternativas consideradas

- **Manter o backlog original e só acrescentar tickets.** Rejeitada: preserva trabalho sem origem na demanda e aumenta o risco do prazo.
- **Cortar sem registrar (apagar tickets/RFs).** Rejeitada: IDs normativos nunca são reutilizados (`AGENTS.md` §10); o histórico de decisão precisa ser rastreável.
- **Tratar os gaps de fonte (rede, SIEM) com parsers dedicados (PCAP, syslog, etc.).** Rejeitada para o MVP: o importador estruturado cobre exportações de SIEM/firewall/IdP/e-mail com custo muito menor; PCAP segue na Fase 4.

## Consequências

- O backlog passa a ter 21 tickets novos (F1-13..F1-33) e 2 tickets adiados (F1-08, F1-09); `docs/FASE1_BACKLOG.md` e a RTM são reescritos.
- `F1-12` (release candidate) reconcilia só os RFs que permanecem MVP.
- Risco aceito: o importador estruturado e as regras de detecção são trabalho novo e não verificado; entram como `PLANNED`.

## Atualização — 2026-10-11

Depois desta ADR o backlog ganhou F1-34 (CLI headless, ADR-0019) e F1-35 (observabilidade e testes de GUI, ADR-0022); as ADR-0020 e ADR-0021 deram tratamento a F1-17, F1-18, F1-10 (parcial), F1-19 (parcial) e F1-20 (parcial). O estado vigente de cada ticket está em `docs/rtm.md`.
