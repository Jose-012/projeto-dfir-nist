<!-- doc-type: decision | status: accepted -->
# ADR-0025 — Evento normalizado (F1-26), detecção por regras com confirmação (F1-28) e escopo EVTX (F1-27)

## Status

ACEITA — 2026-10-27, em resposta ao pedido do operador de concluir as etapas e "finalizar todos os pedidos e restrições da demanda original". Fecha F1-26 e F1-28, e **registra formalmente** a decisão do operador sobre F1-27.

## Contexto

1. **Lacuna medida contra a demanda.** A demanda manda identificar vetor inicial, movimentação lateral, escalonamento de privilégios e tentativas de exfiltração. Até a v6.5.0 isso era **registro manual do analista**; nada era detectado. O rastreio marcava "COBERTO" porque a tela de registro existia.
2. **Dados que o EVTX já traz e a aplicação ignorava.** O parser entregava o registro inteiro, mas a timeline guardava só título, descrição, Event ID, provedor e canal. Conta, máquina, IP de origem, processo e linha de comando ficavam enterrados no JSON.
3. **Escopo (decisão do operador, 2026-10-13, ADR-0023 contexto 2; reafirmada em 2026-10-27).** O projeto é protótipo/MVP; ao medir lacunas, formatos de log além de EVTX ficam de fora.
4. **Princípio da spec (§4.3 e `F1-28`).** Regra interna só produz candidato; nada vira achado sem confirmação do analista.

## Decisão

### D-1 — Evento normalizado com campos estruturais (F1-26)

`NormalizedEvent` ganha, como **opcionais**, `event_id`, `provider`, `channel`, `host`, `user`, `src_ip`, `dst_ip`, `process`, `command_line`, `action`, `outcome` e `source_type` (esquema do protocolo 3 → **4**). Regras:

- **Só o que o registro diz.** Campo ausente continua ausente; `-`, `(NULL)` e vazio viram "ausente". Nada é inferido.
- **O mesmo extrator no engine e no app** (`dfir_protocol::normalize`). O engine escreve os campos nos resultados; o app os **recalcula do próprio registro** para resultados persistidos na versão 3, que continuam legíveis (teste com a forma exata da v3).
- Mapeamentos por Event ID para: 4624/4625 (logon), 4634/4647, 4648, 4672, 4688, 4697/7045, 4698, 4720, 4722–4726/4738/4740/4781, 4728/4732/4756, 5140/5145, 5156/5157, 1102/104, 4719, Sysmon 1/3/10 e PowerShell 4103/4104. Os demais eventos recebem os campos de sistema e a conta, se o registro a traz.
- Valores têm teto de tamanho (512; 4096 para linha de comando) e sem NUL.

**Timeline no banco:** migration 0012 adiciona as colunas e **invalida** o índice de timeline (`DELETE FROM timeline_index`), de modo que cada job é reindexado a partir do `results.jsonl` na primeira consulta, com os campos novos, inclusive os jobs antigos.

**Filtro estruturado** (o mesmo campo de texto, sem mudar a interface): `event:`, `user:`, `host:`, `ip:`, `src:`, `dst:`, `proc:`, `action:`, `outcome:`, `provider:`, `channel:`, `from:`, `to:`; termos diferentes combinam por E, vírgula é OU, aspas agrupam, o que não é `campo:valor` conhecido segue sendo texto livre. **Data inválida é erro, não texto livre**, e a interface a valida antes de enviar (um erro de filtro derrubava o estado da timeline e escondia a própria caixa; encontrado ao projetar o filtro).

### D-2 — Detecção por regras (F1-28)

Nova análise `EVTX_DETECTION` no engine (mesmo parser, mesmos limites, mesmo sandbox). **19 regras** versionadas (`RULE_SET_VERSION`), catálogo em `docs/DETECTION_RULES.md` (gerado de `dfir-engine --list-rules`; um teste confere que a doc lista todas). Cobertura dos quatro pedidos da demanda:

| Pedido da demanda | Regras | Natureza do sinal |
|---|---|---|
| Vetor inicial | `R-ACCESS-001`, `R-AUTH-002` (+ `R-AUTH-001`) | logon remoto de IP público; sucesso depois de rajada de falhas; rajada/spray |
| Movimentação lateral | `R-LATERAL-001..004` | RDP interno; credenciais explícitas a outro servidor; ADMIN$/C$; serviço de execução remota (PsExec) |
| Escalonamento de privilégios | `R-PRIV-001`, `R-PRIV-002` | SeDebug/SeTcb/SeLoadDriver em conta de usuário; membro adicionado a grupo privilegiado |
| Tentativa de exfiltração | `R-EXFIL-001`, `R-EXFIL-002`, `R-COLL-001` | linha de comando de envio (rclone, curl -T, Invoke-WebRequest -Method Post...); conexão de saída a IP público por ferramenta de linha de comando; compactação |
| Apoio | `R-PERS-001..003`, `R-EVAS-001..002`, `R-EXEC-001`, `R-CRED-001` | conta/serviço/tarefa criados; log apagado; auditoria alterada; PowerShell codificado; extração de credenciais |

**Cada candidato** carrega: regra e versão, conjunto de regras, tática/técnica ATT&CK sugerida, gravidade, **confiança que nunca passa de MEDIUM** (regra automática não afirma mais; o catálogo é testado para isso), agrupamento, contagem, período, **referências aos eventos de origem** (`record:N`, capadas em 20: as 15 primeiras e as 5 últimas, com marca de truncamento), IOCs vistos (IP, URL, host, caminho) e **o que a regra não prova** (obrigatório e testado). Tudo é limitado: 5 000 candidatos, 50 000 chaves de rajada, 20 IOCs por candidato; estourar o teto vira `DETECTION_LIMIT_REACHED`, não omissão silenciosa.

**Decisão do analista:** `PENDING` → `CONFIRMED` ou `REJECTED`, final (um gatilho do banco recusa mudar o que a regra afirmou e reabrir uma decisão).
- **Confirmar** (`candidate-confirm`, botão na tela) cria no incidente escolhido o **mapeamento ATT&CK**, o **achado** e os **IOCs**, com a base citando regra, versão e eventos de origem. Os passos gravam seus vínculos à medida que terminam: uma falha no meio é **retomada** sem duplicar.
- **Rejeitar** exige **motivo**, que fica na auditoria.
- Rodar a detecção de novo **não duplica nem desfaz decisões** (chave estável: regra + versão + agrupamento).
- Auditoria: `DETECTION_CANDIDATE_CONFIRMED` / `_REJECTED`. Candidatos saem com a evidência quando ela é descartada (carregam contas e IPs; RB-006); achados e IOCs já confirmados ficam no incidente.

**O que fica explícito no relatório (Entregável 1, §4.3):** por evidência, quantos candidatos foram confirmados, rejeitados e **pendentes** ("não avaliados e não constam como achado"), ou "regras não executadas", ou "evidência descartada". Não se confunde "rodou e nada disparou" com "não rodou".

### D-3 — Limites que a documentação deve repetir

- **A ausência de candidato não prova ausência de ataque.** A regra só enxerga o que o log registrou: sem auditoria de criação de processo (4688) não há regra de linha de comando; sem a plataforma de filtragem (5156) ou Sysmon não há conexão de saída; **sem log de rede não há como confirmar exfiltração**, só a tentativa de usar a ferramenta.
- Limiares são heurísticas fixas (10 falhas em 5 minutos; sucesso em até 30 minutos). Não aprendem, não têm *baseline* do ambiente, não conhecem a lista de origens autorizadas.
- Casamento por texto de linha de comando pega a tentativa (inclusive a de equipe de segurança) e não pega ferramenta renomeada ou ofuscada.
- `R-PRIV-001` fica com gravidade BAIXA e diz que todo administrador recebe os privilégios; é uma lista de quem os usa, não prova de abuso.

### D-4 — F1-27 (importador CSV/JSONL): **adiado por decisão do operador**

Não é implementado. RF-049 e o ticket F1-27 passam a `DEFERRED — decisão do operador (ADR-0025)`, com o mesmo critério da ADR-0023: o escopo do protótipo é EVTX. Consequência declarada, não escondida: **fontes de rede, SIEM, dispositivos de segurança e e-mail/identidade na nuvem ficam fora do que a ferramenta analisa** (as regras de "encaminhamento de e-mail" e "login de origem incomum" do ticket F1-28 dependem delas). O rastreio da demanda marca essas linhas como `ADIADO (decisão do operador)`, não como `PARCIAL` — `PARCIAL` sugere que o trabalho está em andamento.

Se o operador mudar de ideia, F1-27 é independente: entra por um importador que produza o mesmo `NormalizedEvent`, e as regras passam a valer sobre ele sem alteração.

### D-5 — Detalhes técnicos que mudam contratos

- Protocolo: `SCHEMA_VERSION` 4; `ANALYSIS_KIND_EVTX_DETECTION`; item `DETECTION_CANDIDATE`; `PROGRESS` com fase `sandbox:` (ADR-0024).
- Migration 0012 (também recria `analysis_jobs`, porque o `CHECK` do `kind` listava só `EVTX_TIMELINE` e `YARA_SCAN`; ver ADR-0026 para a cópia prévia). O defeito do `CHECK` só apareceu ao rodar a detecção com o banco de verdade: os testes de serviço não rodam o engine.
- Relógio: o engine mede janelas com um conversor próprio de data (sem dependência nova); testado contra instantes conhecidos, fusos, ano bissexto e virada de século.

## Consequências

- DEM-ID-01/03/04/05 passam de "registro manual" a **detecção candidata + confirmação**, ainda `PENDING-HUMAN` e **só sobre o que o EVTX registrou**.
- O critério de F1-12 "nenhuma linha da RTM em `PENDING`/`PLANNED` para RFs MVP" é atendido, **com RF-049 fora do MVP por decisão do operador**.
- A confirmação em lote não existe (um candidato por vez): é intencional para o protótipo; um ruído aceito vira 5 000 cliques, e o filtro de situação da tela mostra no máximo 100 por vez.
