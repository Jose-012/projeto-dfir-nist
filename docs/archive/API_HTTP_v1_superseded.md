# API HTTP do `dfir-app`

Contrato das rotas locais (`127.0.0.1:8080` por padrão). Não há autenticação, sessão nem papéis: a aplicação é local e de um único operador (ADR-0007). Este documento descreve o que o código faz hoje; quando ele e o código divergirem, o código vence e este arquivo é o que deve ser corrigido. A implementação vive em `src/web/mod.rs`; a interface (`frontend/src/incident-ui.ts`) consome exatamente estas rotas.

## Convenções

- **Corpo**: JSON (`content-type: application/json`), exceto o upload de evidência (`multipart/form-data`).
- **Sucesso**: `200` com JSON, salvo onde indicado (`204` sem corpo).
- **Erro**: o corpo é **texto puro**, não JSON — apenas a mensagem.

  | Status | Quando |
  |---|---|
  | `400` | Entrada inválida: JSON malformado, campo obrigatório ausente, valor fora da lista aceita, ou regra de validação da persistência (ver abaixo). |
  | `404` | Incidente, evidência ou job inexistente (ou evidência já destruída). |
  | `408` | Requisição excedeu o teto de tempo (timeout do engine + margem). |
  | `500` | Falha de banco ou de storage. A mensagem é genérica (`database operation failed`, `storage operation failed`), sem detalhes internos. |
  | `502` | Falha do processo do engine (`analysis engine failed`). |

- **Validação da persistência**: erros de regra de negócio detectados na camada de persistência (IOC malformado, id de técnica ATT&CK fora do formato, profile sem entradas, recomendação sem achado de suporte…) respondem `400` com o motivo no corpo. Antes desta revisão eles chegavam como `500 database operation failed` — se você integrar com um cliente antigo, trate ambos.
- **Cabeçalhos de segurança** em todas as respostas: CSP restritiva (`default-src 'self'`, sem script/estilo inline), `x-content-type-options: nosniff`, `referrer-policy: no-referrer`; `x-request-id` é propagado.
- **Identificadores**: UUID v4 em texto, gerados pelo servidor.
- **Datas**: RFC 3339. Quando o servidor gera o valor usa `+00:00`; valores enviados pelo cliente devem ser RFC 3339 em UTC (preferencialmente com `Z`).
- **Versionamento**: toda escrita analítica é *append-only*. Reenviar cria uma nova versão que aponta para a anterior em `supersedes`; nada é apagado. As rotas `GET` de listagem devolvem só as versões **vigentes** (não referenciadas como `supersedes` por outra linha), salvo onde indicado.

### Enumerações aceitas

| Campo | Valores |
|---|---|
| `confidence` | `LOW`, `MEDIUM`, `HIGH`, `CONFIRMED` |
| `profile_type` | `CURRENT`, `TARGET` |
| `implementation_status` | `IMPLEMENTED`, `PARTIALLY_IMPLEMENTED`, `NOT_IMPLEMENTED`, `NOT_ASSESSED`, `NOT_APPLICABLE` |
| `tier` | `TIER_1_PARTIAL`, `TIER_2_RISK_INFORMED`, `TIER_3_REPEATABLE`, `TIER_4_ADAPTIVE` |
| `tactic` | `RECONNAISSANCE`, `RESOURCE_DEVELOPMENT`, `INITIAL_ACCESS`, `EXECUTION`, `PERSISTENCE`, `PRIVILEGE_ESCALATION`, `DEFENSE_EVASION`, `CREDENTIAL_ACCESS`, `DISCOVERY`, `LATERAL_MOVEMENT`, `COLLECTION`, `COMMAND_AND_CONTROL`, `EXFILTRATION`, `IMPACT` |
| `*_impact` | `NONE`, `LOW`, `MEDIUM`, `HIGH`, `CRITICAL` |
| `ioc_type` | `IP`, `DOMAIN`, `URL`, `EMAIL`, `SHA256`, `SHA1`, `MD5`, `FILE_PATH`, `HOSTNAME` |
| `category` (recomendação) | `IMMEDIATE`, `STRATEGIC` |
| `kind` (análise) | `EVTX_TIMELINE` (padrão), `YARA_SCAN` |

### Formato de resultado analítico

As rotas de listagem analítica devolvem objetos neste formato (`AnalyticalResultResponse`):

```json
{
  "id": "…",
  "result_type": "IOC",
  "correlation_key": "…",
  "evidence_id": "…" ,
  "incident_id": "…",
  "analysis_job_id": null,
  "rule_or_model_id": null,
  "rule_or_model_version": null,
  "confidence": "HIGH",
  "payload": { "…": "específico do tipo, ver abaixo" },
  "provenance": { "producer": "dfir-app", "method": "…" },
  "supersedes": null,
  "created_at": "2026-08-01T00:00:00+00:00"
}
```

`evidence_id`, `incident_id`, `analysis_job_id`, `rule_or_model_*`, `confidence` e `supersedes` podem ser `null`. O `payload` nunca contém bytes brutos de evidência.

---

## Saúde

| Rota | Resposta |
|---|---|
| `GET /health/live` | `204` — o processo está de pé. |
| `GET /health`, `GET /health/ready` | `204` se o banco responde a `SELECT 1`; `503` caso contrário. |

## Interface

| Rota | Resposta |
|---|---|
| `GET /` | HTML (página única). |
| `GET /assets/app.js`, `GET /assets/app.css` | Assets estáticos embutidos no binário. |

---

## Evidências e análises

### `POST /api/evidence`
Importa uma evidência. `multipart/form-data` com **um** campo `evidence` (arquivo, com nome; até 255 bytes, sem `\0`). O arquivo passa por quarentena, BLAKE3 + SHA-256 em streaming e é promovido ao storage. Limite de tamanho: `DFIR_MAX_UPLOAD_BYTES` (512 MiB por padrão). Mais de um arquivo por requisição → `400`.

Resposta:
```json
{ "id": "…", "original_filename": "Security.evtx", "storage_reference": "evidence/…",
  "format": "evtx", "size_bytes": 1048576, "blake3": "…", "sha256": "…",
  "imported_at": "…", "status": "…" }
```
`format` é só uma dica de exibição, derivada da extensão.

### `GET /api/evidence`
Lista até 100 evidências não destruídas, da mais recente para a mais antiga (mesmo formato acima).

### `DELETE /api/evidence/{evidence_id}`
Destruição irreversível do arquivo, registrada na auditoria. `204` em sucesso; `404` se inexistente ou já destruída; `400` se houver job `PENDING`/`PROCESSING` para a evidência.

### `GET /api/evidence/{evidence_id}/analytical-results`
Todo o histórico de resultados analíticos da evidência (inclui versões já substituídas), p. ex. matches YARA. Lista de resultados analíticos.

### `POST /api/analyze`
Executa o engine (processo separado) sobre uma evidência, de forma síncrona.

```json
{ "evidence_id": "…", "kind": "EVTX_TIMELINE", "max_output_bytes": 4194304, "max_items": 100000 }
```

- `kind` é opcional (padrão `EVTX_TIMELINE`, que só aceita evidência `evtx`; `YARA_SCAN` aceita qualquer formato).
- `max_output_bytes` (1..=4 MiB) e `max_items` (1..=100 000) são opcionais e só podem **reduzir** os limites do protocolo.

Resposta: `{ "request_id", "job_id", "evidence_id", "status", "output_reference", "message_count", "item_count" }`.

### `GET /api/jobs/{job_id}`
Estado do job: `{ job_id, request_id, evidence_id, status, kind, message_count, item_count, output_reference, error_code, error_message, created_at, started_at, finished_at }` (campos opcionais podem ser `null`). `error_code` pode ser `TIMEOUT`, `ENGINE_PROTOCOL_ERROR`, `OUTPUT_LIMIT`, `ENGINE_FAILED`, `INVALID_REQUEST`, `ENGINE_IO_ERROR`, `PERSISTENCE_ERROR`.

### `GET /api/jobs/{job_id}/results`
Timeline derivada do resultado do job:

```json
{ "job": { "…": "como em /api/jobs/{job_id}" },
  "events": [ { "id": "ev:record:42", "timestamp": "2025-01-02T03:04:05Z",
                "title": "Event ID 4624 · Microsoft-Windows-Security-Auditing",
                "description": "…", "category": "EVTX", "severity": "0",
                "event_id": "4624", "provider": "…", "channel": "Security",
                "evidence_id": "…", "raw_reference": "record:42", "record_id": "42" } ],
  "truncated": false }
```

Os eventos vêm de itens no contrato `dfir_protocol::NormalizedEvent`; itens que não o seguem (p. ex. `YARA_MATCH`) ou com `timestamp` não RFC 3339 são ignorados. A lista é ordenada cronologicamente, deduplicada por `id` e limitada a 1000 eventos (`truncated: true` quando cortada; o arquivo de resultado também é lido só até 2000 linhas).

---

## Incidentes

### `POST /api/incidents`
`{ "name": "…" }` — 1 a 200 caracteres após `trim`. Resposta: `{ "id", "name", "state": "OPEN", "created_at" }`. Registra `INCIDENT_CREATED` na auditoria.

### `GET /api/incidents`
Lista todos, do mais recente para o mais antigo (mesmo formato).

### `GET /api/incidents/{incident_id}`
Um incidente, ou `404`.

Todas as rotas abaixo respondem `404` se o incidente não existir.

---

## NIST CSF 2.0 (F1-03)

### `POST /api/incidents/{incident_id}/nist/profiles`
Registra uma nova versão do Current ou Target Profile. Se já houver versão vigente para o mesmo `profile_type`, ela é substituída (`supersedes`) sem ser apagada.

```json
{
  "profile_type": "CURRENT",
  "version_label": "v1 — avaliação inicial",
  "source": "Auditoria interna",
  "tier": "TIER_2_RISK_INFORMED",
  "tier_justification": "…",
  "entries": [
    { "subcategory_id": "GV.OC-01", "subcategory_label": "Contexto organizacional",
      "implementation_status": "PARTIALLY_IMPLEMENTED",
      "justification": "…", "evidence_reference": "…", "confidence": "MEDIUM" }
  ]
}
```

- Obrigatórios: `profile_type`, `version_label`, `source`, `entries` (ao menos 1) e, em cada entrada, `subcategory_id`, `subcategory_label`, `implementation_status`.
- `tier`, `tier_justification`, `justification`, `evidence_reference` e `confidence` são opcionais. O tier descreve o profile inteiro, nunca uma subcategoria.
- `subcategory_id` segue `FUNÇÃO.CATEGORIA-NN` com função `GV`, `ID`, `PR`, `DE`, `RS` ou `RC` (ex.: `GV.OC-01`); é validado só na forma — o catálogo oficial não é embutido (ADR-0011). Repetir um `subcategory_id` no mesmo envio → `400`.

Resposta: `{ "profile_id": "…" }`.

### `GET /api/incidents/{incident_id}/nist/profiles?type=CURRENT|TARGET`
Histórico completo do tipo pedido (padrão `CURRENT`), **da mais antiga para a mais recente**, incluindo versões substituídas:

```json
{ "profile_type": "CURRENT",
  "versions": [ { "id", "version_label", "source", "tier", "tier_justification",
                  "supersedes", "created_at",
                  "entries": [ { "subcategory_id", "subcategory_label", "implementation_status",
                                 "justification", "evidence_reference", "confidence" } ] } ] }
```

### `POST /api/incidents/{incident_id}/nist/gap-analysis`
Recalcula a gap analysis a partir dos profiles CURRENT e TARGET vigentes (determinística, reproduzível offline) e persiste um resultado `NIST_GAP` por lacuna. Sem corpo. Resposta: `{ "gap_result_ids": ["…"], "gap_count": 3 }`. `400` se faltar o Current ou o Target.

### `GET /api/incidents/{incident_id}/nist/gap-analysis`
Gaps vigentes (`result_type: "NIST_GAP"`). Payload:
`{ kind, subcategory_id, subcategory_label, current_status, target_status, priority, risk_level }`, com `priority` `HIGH` ou `MEDIUM`. Subcategorias `NOT_APPLICABLE` em qualquer dos lados não geram gap; entrada ausente no Current conta como `NOT_ASSESSED`.

---

## ATT&CK, caminho crítico e impacto (F1-04)

### `POST /api/incidents/{incident_id}/attack-mappings`
```json
{ "evidence_id": null, "tactic": "EXECUTION", "technique_id": "T1059",
  "sub_technique_id": "T1059.001", "basis": "PowerShell 4104 em record:42",
  "confidence": "MEDIUM", "analyst_note": "…" }
```
- Obrigatórios: `tactic`, `technique_id`, `basis` (pode ser `""`), `confidence`. `evidence_id`, `sub_technique_id` e `analyst_note` são opcionais.
- `technique_id` segue `T####`; `sub_technique_id`, quando enviado, é o identificador **completo** `T####.###` (ex.: `T1059.001`, não `001`). O catálogo do MITRE não é embutido; só a forma é validada.
- **Regra de evidência**: `basis` vazio impede `confidence` `HIGH` ou `CONFIRMED` (`400`). Um mapeamento sem `basis` é gravado com `evidence_status: "GAP"` — incerteza, não fato.
- **Chave de versionamento**: incidente + evidência (ou "nenhuma") + `technique_id`. Reenviar a mesma chave substitui a versão anterior. A tática e a sub-técnica **não** fazem parte da chave: a mesma técnica registrada sob duas táticas diferentes (ex.: T1078 em Acesso inicial e em Persistência) para a mesma evidência colide, e a segunda substitui a primeira. Se precisar de ambas, registre cada uma com uma evidência relacionada distinta.

Resposta: `{ "mapping_id": "…" }`.

### `GET /api/incidents/{incident_id}/attack-mappings`
Mapeamentos vigentes (`result_type: "ATTACK_MAPPING"`). Payload: `{ kind, tactic, technique_id, sub_technique_id, basis, analyst_note, evidence_status }`.

### `POST /api/incidents/{incident_id}/critical-path/stages`
```json
{ "evidence_id": null, "sequence_index": 0, "stage_label": "Acesso inicial via phishing",
  "basis": "…", "confidence": "MEDIUM" }
```
`sequence_index` é a posição (base 0) e a identidade de versionamento: descrever de novo o estágio N substitui a versão anterior do estágio N. A regra de `basis` é a mesma do mapeamento ATT&CK. Resposta: `{ "stage_id": "…" }`.

> As ligações entre estágios **não** são reatribuídas quando um estágio é substituído (ADR-0012): depois de redescrever um estágio, refaça as ligações dele.

### `POST /api/incidents/{incident_id}/critical-path/links`
`{ "from_stage_id": "…", "to_stage_id": "…" }` — liga dois estágios com uma relação `SEQUENCE_NEXT` (permite ramificação). Resposta: `{ "linked": true }`.

### `GET /api/incidents/{incident_id}/critical-path`
```json
{ "stages": [ /* resultados analíticos, result_type CRITICAL_PATH_LINK */ ],
  "edges": [ { "from": "<stage id>", "to": "<stage id>" } ] }
```
Payload de estágio: `{ kind: "CRITICAL_PATH_STAGE", sequence_index, stage_label, basis, evidence_status }`. A ordem de `stages` não é garantida; ordene por `payload.sequence_index`.

### `POST /api/incidents/{incident_id}/impact-assessment`
```json
{ "availability_impact": "HIGH", "confidentiality_impact": "MEDIUM", "integrity_impact": "NONE",
  "affected_assets": "…", "affected_data": "…", "operational_disruption": "…",
  "financial_impact": null, "confidence": "MEDIUM" }
```
Todos obrigatórios, exceto `financial_impact` — omita ou envie `null` quando não houver dado; o sistema nunca fabrica valor financeiro. Há uma única avaliação por incidente, versionada. Resposta: `{ "impact_assessment_id": "…" }`.

### `GET /api/incidents/{incident_id}/impact-assessment`
A avaliação vigente (resultado analítico) ou `null`.

---

## IOCs, recomendações e exportação (F1-05)

### `POST /api/incidents/{incident_id}/iocs`
Registra **uma ocorrência** (sighting) de um indicador.

```json
{ "evidence_id": "…", "ioc_type": "IP", "value": "198.51.100.7",
  "event_context": "…", "confidence": "HIGH", "observed_at": "2026-08-01T12:30:00Z" }
```
- Obrigatórios: `evidence_id` (evidência existente), `ioc_type`, `value`, `confidence`. `event_context` e `observed_at` são opcionais (sem `observed_at`, vale o instante do registro).
- O valor é normalizado e validado por tipo (IP via `std::net::IpAddr`; hashes hexadecimais com o tamanho certo; URL com esquema `http`/`https`/`ftp` e sem espaços; e-mail, domínio e hostname com checagens estruturais leves).
- Ocorrências do mesmo indicador (mesmo incidente + tipo + valor normalizado) são **agregadas** numa única linha vigente; cada envio gera uma nova versão.
- **`observed_at` é comparado como texto**: use sempre RFC 3339 em UTC, no mesmo formato, para que a ordem cronológica coincida com a ordem lexicográfica.

Resposta: `{ "ioc_result_id": "…" }`.

### `GET /api/incidents/{incident_id}/iocs`
IOCs vigentes (`result_type: "IOC"`). Payload:

```json
{ "kind": "IOC", "ioc_type": "IP", "normalized_value": "198.51.100.7",
  "first_seen": "…", "last_seen": "…",
  "sighting_count": 250, "sightings_dropped": 150,
  "sightings": [ { "evidence_id": "…", "event_context": null, "confidence": "HIGH", "observed_at": "…" } ] }
```

**Teto de ocorrências (`MAX_STORED_SIGHTINGS = 100`)**: `sightings` guarda no máximo as 100 ocorrências mais recentes (por `observed_at`). `sighting_count` é o total real, inclusive as que saíram do array, e `sightings_dropped = sighting_count − sightings.length`. `first_seen` e `last_seen` cobrem **todas** as ocorrências. As ocorrências descartadas continuam nas versões anteriores da linha (a tabela é append-only) e o histórico completo segue acessível pela cadeia `supersedes`; o teto limita o tamanho de cada linha, não o número de versões. Payloads gravados antes do teto (sem `sighting_count`) são aceitos e aparados no próximo registro.

### `POST /api/incidents/{incident_id}/recommendations`
```json
{ "category": "IMMEDIATE", "title": "Isolar o host", "description": "…",
  "confidence": "MEDIUM", "supporting_result_ids": ["<id de um achado>"], "supersedes": null }
```
- Obrigatórios: `category`, `title`, `description`, `supporting_result_ids` (ao menos um id de resultado analítico existente — rastreabilidade). `confidence` e `supersedes` são opcionais; `supersedes`, quando presente, deve ser o id de uma recomendação existente.
- Cada achado de suporte é ligado à recomendação por uma relação `DERIVED_FROM`.

Resposta: `{ "recommendation_id": "…" }`.

### `GET /api/incidents/{incident_id}/recommendations?category=IMMEDIATE|STRATEGIC`
Recomendações vigentes (`result_type: "RECOMMENDATION"`), filtráveis por categoria. Payload: `{ kind, category, title, description }`; os ids de suporte ficam em `provenance.supporting_result_ids`.

### `GET /api/incidents/{incident_id}/export`
Exportação dos achados a partir da **mesma função** que alimenta as listagens (RF-022):

```json
{ "incident_id": "…",
  "iocs": [ /* resultados analíticos */ ],
  "recommendations_immediate": [ /* … */ ],
  "recommendations_strategic": [ /* … */ ] }
```

---

## Limites conhecidos

- **Janela de listagem**: as rotas de resultados vigentes leem os 500 resultados analíticos mais recentes do incidente (de qualquer tipo) e filtram em memória. Um incidente com muitos resultados acumulados pode, em tese, deslocar uma linha de supersessão para fora da janela (ADR-0011). Aceito no volume esperado do MVP.
- **Sem paginação**: as listagens devolvem tudo o que cabe nos limites acima.
- **Análise síncrona**: `POST /api/analyze` só responde quando o engine termina (ou estoura o timeout); a concorrência é limitada por `DFIR_ENGINE_MAX_CONCURRENT_JOBS`.
- **Sem autoria**: `actor_id` é sempre `null` nas gravações; a aplicação não tem identidade de usuário.
