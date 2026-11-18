<!-- doc-type: status | status: current -->
# Rastreabilidade da demanda original → requisitos → tickets

Fonte: `docs/source/DEMANDA_ORIGINAL_SENAI_12412.md` (SAGA SENAI 12412, Avivatec). Esta matriz é a ponte entre a **demanda** e os RFs da seção 39 da especificação. Item da demanda sem linha aqui é defeito de documentação.

**Estados** (nível de cobertura do projeto, reconciliado em 2026-10-27; não é nível de verificação — para verificação ver `docs/FASE1_STATUS.md`):

- `COBERTO` — existe implementação do requisito (ainda `PENDING-HUMAN` onde indicado na RTM).
- `PARCIAL` — existe parte; a lacuna está nomeada.
- `PLANEJADO` — tem RF e ticket; nada implementado.
- `ADIADO` — decisão registrada de não cobrir no protótipo (ADR-0018; para as fontes além de EVTX, a decisão do operador da ADR-0025). **Não** é `PARCIAL`: `PARCIAL` sugere trabalho em andamento.

**Leitura de `COBERTO` nas linhas de identificação (DEM-ID-01, 03, 04, 05):** existe detecção por regras **sobre o que o EVTX registrou**, com confirmação do analista. Não significa que o ataque seria encontrado: a regra depende de a auditoria correspondente estar ligada no Windows (`docs/DETECTION_RULES.md`), e **sem log de rede não há como confirmar exfiltração**, só a tentativa de usar a ferramenta. Verificado em EVTX real (`security.evtx`, que não contém um ataque) e em eventos sintéticos de teste, **nunca em um incidente real**.

## 1. Fontes de evidência (Detalhamento, parágrafo 1)

| ID | Item da demanda | RF | Tickets | Estado | Lacuna / observação |
|---|---|---|---|---|---|
| DEM-FONTE-01 | Coleta, preservação e análise de evidências | RF-001..004, RF-054 | F1-31 | PARCIAL | Preserva, custodia e grava a proveniência que o operador declara (origem, quem forneceu, método, autorização; F1-31, ADR-0023); a análise automática é só de EVTX e não há coleta ativa (Fase 4) |
| DEM-FONTE-02 | Logs de servidores | RF-005, RF-034, RF-048 | F1-16, F1-26 | COBERTO (EVTX) | EVTX com campos estruturais (F1-26, ADR-0025); `PENDING-HUMAN`. Outros formatos: ver DEM-FONTE-04/05/07 |
| DEM-FONTE-03 | Registros de autenticação | RF-034, RF-050 | F1-26, F1-28 | COBERTO (EVTX) | Via EVTX Security: logon (4624/4625/4648), privilégios (4672), contas e grupos; regras R-AUTH, R-ACCESS, R-PRIV. Autenticação de nuvem/diretório fora do escopo (ADIADO) |
| DEM-FONTE-04 | Tráfego de rede | RF-034, RF-049 | F1-27, F1-31 | ADIADO (decisão do operador, ADR-0025) | Só o que o EVTX registra de rede (5156, Sysmon 3, quando a auditoria está ligada). Importador CSV/JSONL (Zeek/Wireshark) e PCAP: fora do protótipo |
| DEM-FONTE-05 | Sistemas de monitoramento / SIEM | RF-034, RF-049 | F1-27 | ADIADO (decisão do operador, ADR-0025) | Exportações CSV/JSONL não são importadas; o analista pode registrar achados e IOCs à mão |
| DEM-FONTE-06 | Endpoints e estações de trabalho | RF-005, RF-034, RF-050 | F1-16, F1-28 | COBERTO (EVTX) | EVTX de estação (4688 com linha de comando, PowerShell 4104, Sysmon quando presente); coleta especializada é Fase 4 (RF-023) |
| DEM-FONTE-07 | Dispositivos de segurança | RF-034, RF-049 | F1-27 | ADIADO (decisão do operador, ADR-0025) | Exportações de firewall/IDS não são importadas |

## 2. Identificação (Detalhamento, parágrafo 2)

| ID | Item da demanda | RF | Tickets | Estado | Lacuna / observação |
|---|---|---|---|---|---|
| DEM-ID-01 | Vetor inicial do ataque | RF-011, RF-018, RF-050, RF-051 | F1-25, F1-28 | COBERTO | Candidatos `R-ACCESS-001` (logon remoto de IP público), `R-AUTH-002` (sucesso depois de rajada), `R-AUTH-001`; o analista confirma e o item vira mapeamento ATT&CK, achado e IOC (F1-28, ADR-0025); o registro manual segue existindo; `PENDING-HUMAN` |
| DEM-ID-02 | Vulnerabilidades exploradas | RF-051, RF-011 | F1-25 | COBERTO | Aba "Achados" (F1-25, ADR-0023): tipo, severidade, confiança, base, ATT&CK e NIST; `PENDING-HUMAN`. O Entregável 2 é gerado (F1-07) |
| DEM-ID-03 | Movimentação lateral | RF-018, RF-050 | F1-28 | COBERTO | `R-LATERAL-001..004` (RDP interno, credenciais explícitas, ADMIN$/C$, serviço de execução remota); `PENDING-HUMAN` |
| DEM-ID-04 | Escalonamento de privilégios | RF-018, RF-050 | F1-28 | COBERTO | `R-PRIV-001..002` (privilégios sensíveis em conta de usuário; membro adicionado a grupo privilegiado); `PENDING-HUMAN`. Verificado no `security.evtx` real (grupo Administrators, contas `fsir` e `archir`) |
| DEM-ID-05 | Tentativas de exfiltração | RF-019, RF-020, RF-050 | F1-28 | COBERTO (limitado) | `R-EXFIL-001..002`, `R-COLL-001`: **tentativa** de usar ferramenta de envio ou conexão de saída por ferramenta de linha de comando. É o item mais fraco: exige auditoria 4688/5156/Sysmon e **não confirma a transferência sem log de rede**; `PENDING-HUMAN` |
| DEM-ID-06 | Impacto em sistemas e serviços | RF-020 | — | COBERTO | Matriz de impacto com confiança |
| DEM-TL-01 | Linha do tempo do incidente | RF-007, RF-048 | F1-18, F1-26 | COBERTO (EVTX) | EVTX completo: todos os eventos, ordenados por UTC, paginados e filtrados no SQLite, com campos estruturais e filtro `campo:valor` (ADR-0020/0021/0025) e avisos de arquivo truncado; só EVTX |
| DEM-CR-01 | Causa raiz (Benefício 1) | RF-051 | F1-25 | COBERTO | Níveis causa-raiz / provável / fator contribuinte, com lacunas explícitas (F1-25, ADR-0023); `PENDING-HUMAN` |

## 3. Avaliação NIST CSF 2.0 (Detalhamento, parágrafo 3)

| ID | Item da demanda | RF | Tickets | Estado | Lacuna / observação |
|---|---|---|---|---|---|
| DEM-NIST-01 | Governança (GV) | RF-009, RF-010, RF-053 | F1-30 | COBERTO | Perfis/gap por subcategoria e visão agregada por Function (F1-30, ADR-0023); `PENDING-HUMAN`. O Entregável 3 é gerado (F1-07) |
| DEM-NIST-02 | Identificação de ativos (ID) | idem | F1-30 | COBERTO | idem |
| DEM-NIST-03 | Controles de proteção (PR) | idem | F1-30 | COBERTO | idem |
| DEM-NIST-04 | Monitoramento (DE) | idem | F1-30 | COBERTO | idem |
| DEM-NIST-05 | Resposta a incidentes (RS) | idem | F1-30 | COBERTO | idem |
| DEM-NIST-06 | Capacidade de recuperação (RC) | idem | F1-30 | COBERTO | idem |

## 4. Plano de fortalecimento (Detalhamento, parágrafo 4) — todos via `PlanItem` (RF-052, F1-29)

| ID | Item da demanda | Domínio do item de plano | Estado |
|---|---|---|---|
| DEM-PLAN-01 | Políticas de hardening de SO e servidores | `OS_HARDENING` | COBERTO |
| DEM-PLAN-02 | Revisão de configurações de segurança em redes e dispositivos | `REDE_DISPOSITIVOS` | COBERTO |
| DEM-PLAN-03 | Melhoria de autenticação e controle de acesso | `AUTENTICACAO_ACESSO` | COBERTO |
| DEM-PLAN-04 | Implantação/aprimoramento de monitoramento de segurança | `MONITORAMENTO` | COBERTO |
| DEM-PLAN-05 | Criptografia para dados armazenados e em trânsito | `CRIPTO_REPOUSO`, `CRIPTO_TRANSITO` | COBERTO |
| DEM-PLAN-06 | Revisão da gestão de chaves criptográficas | `GESTAO_CHAVES` | COBERTO |
| DEM-PLAN-07 | Políticas e procedimentos de resposta a incidentes | `POLITICA_IR` | COBERTO |

## 5. Entregáveis (Detalhamento, parágrafo 5)

Os seis requisitos de entrega são empacotados em cinco documentos (spec §4.2, decisão de apresentação). Todos saem do ticket F1-07, **um único relatório por documento** (sem perfis executivo/técnico — ADR-0018). **Verificação (2026-10-27, rustc 1.97.1):** os cinco documentos foram gerados e conferidos pelo e2e headless (110 verificações) sobre o **incidente de demonstração** (cenário sintético, rotulado DEMO; `docs/DEMO.md`), não sobre um incidente real.

| ID | Entregável da demanda | Documento (spec §4.2) | RF | Tickets | Estado |
|---|---|---|---|---|---|
| DEM-ENT-01 | Relatório completo de análise forense | Entregável 1 | RF-013 | F1-07, F1-25 | COBERTO |
| DEM-ENT-02 | Identificação das vulnerabilidades exploradas | Entregável 2 | RF-013, RF-051 | F1-07, F1-25 | COBERTO |
| DEM-ENT-03 | Avaliação de maturidade NIST CSF 2.0 | Entregável 3 | RF-013, RF-053 | F1-07, F1-30 | COBERTO |
| DEM-ENT-04 | Recomendações de melhoria da postura | Entregável 3 | RF-013, RF-022, RF-052 | F1-07, F1-29 | COBERTO |
| DEM-ENT-05 | Plano de hardening e fortalecimento da infraestrutura | Entregável 4 | RF-013, RF-052 | F1-07, F1-29 | COBERTO |
| DEM-ENT-06 | Plano de implementação de criptografia e proteção de dados | Entregável 5 | RF-013, RF-052 | F1-07, F1-29 | COBERTO |

Benefícios esperados 2–10 da demanda decorrem dos entregáveis acima e do plano (resposta a incidentes → DEM-PLAN-07; vulnerabilidades → DEM-ID-02; resiliência/proteção de dados/hardening/criptografia → DEM-PLAN-01..06; monitoramento e detecção → DEM-PLAN-04 e RF-050; alinhamento NIST → DEM-NIST-*).

## 6. Restrições da demanda

| ID | Restrição (texto original) | Tratamento | RF | Tickets | Estado |
|---|---|---|---|---|---|
| RB-001 | Disponibilidade limitada de logs por retenção insuficiente | Lacunas de fonte/período são registradas e entram nos relatórios | RF-054 | F1-31 | COBERTO |
| RB-002 | Dependência da colaboração de diferentes áreas | Proveniência registra a área/sistema de origem e quem forneceu | RF-054 | F1-31 | COBERTO |
| RB-003 | Possível indisponibilidade de sistemas durante análises/testes | Análise 100% offline sobre cópia preservada; nenhum teste ativo contra o ambiente do cliente (o engine roda contido, sem rede: ADR-0024) | RF-014, RF-033 | F1-22 | COBERTO |
| RB-004 | **Orçamento limita implementar tudo de imediato** | Itens de plano carregam prioridade, esforço, classe de custo e fase; recomendações imediatas × estratégicas já existem | RF-052, RF-022 | F1-29 | COBERTO |
| RB-005 | Alinhamento com políticas internas e requisitos regulatórios | Item de plano carrega referência regulatória/política interna | RF-052 | F1-29 | COBERTO |
| RB-006 | Confidencialidade e privacidade durante a investigação | Execução local, offline; redação/mascaramento na exportação | RF-014, RF-055 | F1-32 | COBERTO |

> **Correção de leitura (v6.5.0):** em v6.4.5 a RB-004 era tratada como "preferir componentes open source". A restrição original é sobre o orçamento **do cliente** para implementar as recomendações. Preferir componentes open source continua sendo uma decisão de engenharia válida, mas não é o tratamento da RB-004.

## 7. Contexto registrado na demanda

- Ideias 310326 e 310327 ("Comprometimento de Credenciais e Fraude Financeira via Phishing — BEC") são **ideias registradas sob a demanda**, não a demanda. A demanda descreve "o incidente cibernético ocorrido" de forma genérica. O BEC é o cenário do dataset de demonstração (spec §2.3, §42; `dfir-app demo`, versão `bec-demo-1`) sem limitar a plataforma. É **sintético**: não decorre do arquivo EVTX que a demo importa.
