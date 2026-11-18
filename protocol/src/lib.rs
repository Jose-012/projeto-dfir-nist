pub mod normalize;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

pub const PROTOCOL_VERSION: &str = "1";
/// Versão 4 (F1-26/F1-28/F1-22): o evento normalizado ganha campos estruturais
/// opcionais, surge a análise `EVTX_DETECTION` com o item `DETECTION_CANDIDATE`
/// e o engine passa a informar o estado do sandbox em um `PROGRESS`. Itens
/// persistidos na versão 3 continuam legíveis (campos novos são opcionais).
pub const SCHEMA_VERSION: &str = "4";
pub const REQUEST_MESSAGE: &str = "REQUEST";
pub const PROGRESS_MESSAGE: &str = "PROGRESS";
pub const RESULT_MESSAGE: &str = "RESULT";
pub const ERROR_MESSAGE: &str = "ERROR";
/// Tamanho máximo de UMA linha JSONL (moldura do protocolo, não limite de
/// saída). O engine fecha cada lote antes de estourar este valor; só um único
/// registro cujo JSON passe de 8 MiB falha. Era 64 KiB, o que derrubava o job
/// inteiro diante de um registro EVTX grande (ADR-0020).
pub const MAX_LINE_BYTES: usize = 8 * 1024 * 1024;
/// Maior inteiro que o JSON (RFC 8785/JCS, JavaScript) representa sem perda:
/// 2^53 - 1. Os tetos abaixo usam exatamente este valor porque os limites do
/// pedido entram nos metadados da cadeia de auditoria, e a canonicalização
/// rejeita inteiros fora de ±2^53 (um `i64::MAX` quebrava toda análise).
const JSON_SAFE_INTEGER_MAX: u64 = (1 << 53) - 1;
/// Teto técnico de `max_output_bytes`: ~9 PB, **sem limite prático**
/// (ADR-0020). O app envia este valor por padrão; um teto menor só vale se o
/// operador pedir explicitamente (`dfir-app analyze --max-output-bytes`).
pub const MAX_OUTPUT_BYTES: u64 = JSON_SAFE_INTEGER_MAX;
/// Teto técnico de `max_items`: ~9 quadrilhões, **sem limite prático**
/// (ADR-0020), pelo mesmo motivo de `MAX_OUTPUT_BYTES`.
pub const MAX_ITEMS: u64 = JSON_SAFE_INTEGER_MAX;
pub const MAX_JSON_DEPTH: usize = 256;

/// EVTX parsing into timeline events (the only analysis kind before F1-02).
pub const ANALYSIS_KIND_EVTX_TIMELINE: &str = "EVTX_TIMELINE";
/// YARA-X scan against the curated ruleset (F1-02), format-agnostic.
pub const ANALYSIS_KIND_YARA_SCAN: &str = "YARA_SCAN";
/// Regras internas de detecção sobre os eventos do EVTX (F1-28). Só produz
/// **candidatos**: nada vira achado sem confirmação do analista.
pub const ANALYSIS_KIND_EVTX_DETECTION: &str = "EVTX_DETECTION";
const KNOWN_ANALYSIS_KINDS: [&str; 3] = [
    ANALYSIS_KIND_EVTX_TIMELINE,
    ANALYSIS_KIND_YARA_SCAN,
    ANALYSIS_KIND_EVTX_DETECTION,
];
/// Prefixo da fase de um `PROGRESS` que carrega o estado do sandbox do engine.
pub const SANDBOX_PHASE_PREFIX: &str = "sandbox:";
/// `kind` dos itens de candidato de detecção.
pub const ITEM_KIND_DETECTION_CANDIDATE: &str = "DETECTION_CANDIDATE";
/// Referências de evento guardadas por candidato; o restante vira contagem.
pub const MAX_CANDIDATE_EVENT_REFS: usize = 20;
/// IOCs guardados por candidato.
pub const MAX_CANDIDATE_IOCS: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    pub protocol_version: String,
    pub schema_version: String,
    pub message_type: String,
    pub request_id: String,
    pub job_id: String,
    pub input_reference: String,
    pub output_reference: Option<String>,
    pub max_output_bytes: u64,
    pub max_items: u64,
    /// Which analysis the engine should run over `input_reference`. One of
    /// [`ANALYSIS_KIND_EVTX_TIMELINE`] or [`ANALYSIS_KIND_YARA_SCAN`].
    pub analysis_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Progress {
    pub protocol_version: String,
    pub schema_version: String,
    pub message_type: String,
    pub request_id: String,
    pub job_id: String,
    pub percent: u8,
    pub phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResultMessage {
    pub protocol_version: String,
    pub schema_version: String,
    pub message_type: String,
    pub request_id: String,
    pub job_id: String,
    pub status: String,
    pub items: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorMessage {
    pub protocol_version: String,
    pub schema_version: String,
    pub message_type: String,
    pub request_id: String,
    pub job_id: String,
    pub code: String,
    pub message: String,
}

/// `normalization_status` de um evento completamente normalizado.
pub const NORMALIZATION_STATUS_NORMALIZED: &str = "NORMALIZED";

fn default_normalization_status() -> String {
    NORMALIZATION_STATUS_NORMALIZED.into()
}

/// Evento de timeline normalizado, emitido pelo engine em `RESULT.items` para
/// `EVTX_TIMELINE` e consumido pelo app para montar a timeline.
///
/// Mora neste crate — e não no engine ou no app — porque é o contrato
/// fundacional da ferramenta desde a Fase 0: antes o engine montava um
/// `json!({...})` à mão e o app *adivinhava* a forma por duck-typing
/// (`.get("campo")`), um acoplamento invisível que nenhum compilador
/// verificava. Diferente disso, os itens de `YARA_SCAN` continuam genéricos
/// (`serde_json::Value`, ADR-0010): são um tipo de análise entre vários, não
/// o formato base do produto, e `ResultMessage::items` segue sendo
/// `Vec<Value>` justamente para acomodar os dois.
///
/// A forma serializada é idêntica à que o engine sempre emitiu (mesmas seis
/// chaves), então `SCHEMA_VERSION` não muda. Campos desconhecidos são
/// ignorados na leitura (ex.: o antigo `record_offset`); só
/// `normalization_status` tem default, para tolerar resultados persistidos
/// antes de o campo ser exigido nos fixtures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedEvent {
    pub event_record_id: u64,
    /// Preferencialmente o `SystemTime` do evento; senão, o timestamp do
    /// cabeçalho do registro.
    pub timestamp: String,
    /// Registro EVTX convertido para JSON pelo parser (`Event/System/...`).
    pub data: Value,
    pub evidence_id: String,
    /// Identidade estável do registro dentro da evidência (`record:<id>`);
    /// nunca depende do offset BinXML.
    pub raw_reference: String,
    #[serde(default = "default_normalization_status")]
    pub normalization_status: String,
    // --- Campos estruturais (F1-26). Todos opcionais: ausentes em resultados
    // persistidos antes da versão 4 do esquema e em eventos que não os têm.
    /// Identificador do evento (`4624`, `7045`...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// Máquina que gerou o registro (`Computer`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    /// Conta principal do evento (a que fez logon, foi criada ou executou).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dst_ip: Option<String>,
    /// Processo ou serviço envolvido (caminho ou nome).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_line: Option<String>,
    /// O que aconteceu, em palavras: `logon`, `logon_failed`, `service_installed`...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// `success`, `failure` ou ausente quando o evento não tem resultado.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    /// Origem do dado: `EVTX` hoje.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
}

/// Campos estruturais extraídos de um registro (F1-26), prontos para anexar a
/// um [`NormalizedEvent`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventFields {
    pub event_id: Option<String>,
    pub provider: Option<String>,
    pub channel: Option<String>,
    pub host: Option<String>,
    pub user: Option<String>,
    pub src_ip: Option<String>,
    pub dst_ip: Option<String>,
    pub process: Option<String>,
    pub command_line: Option<String>,
    pub action: Option<String>,
    pub outcome: Option<String>,
    pub source_type: Option<String>,
}

/// IOC que uma regra encontrou num evento que disparou (candidato).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandidateIoc {
    /// `IP`, `DOMAIN`, `URL`, `HOSTNAME`, `FILE_PATH`, `SHA256`...
    pub ioc_type: String,
    pub value: String,
    /// `record:<id>` do evento onde apareceu.
    pub event_ref: String,
}

/// Candidato de detecção emitido por uma regra (F1-28). É uma **hipótese**
/// com a referência dos eventos de origem; só o analista a transforma em
/// mapeamento ATT&CK, achado e IOC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectionCandidate {
    pub kind: String,
    pub evidence_id: String,
    pub rule_id: String,
    pub rule_version: String,
    pub rule_set_version: String,
    pub title: String,
    pub description: String,
    /// Uma das 14 táticas ATT&CK em maiúsculas (`INITIAL_ACCESS`...).
    pub tactic: String,
    pub technique_id: String,
    #[serde(default)]
    pub sub_technique_id: Option<String>,
    /// `LOW`, `MEDIUM`, `HIGH` ou `CRITICAL`.
    pub severity: String,
    /// `LOW` ou `MEDIUM`: uma regra automática nunca afirma mais que isso.
    pub confidence: String,
    /// Chave estável (regra + agrupamento) para não duplicar em nova execução.
    pub candidate_key: String,
    /// O agrupamento em texto (`conta=x; origem=y`).
    pub aggregation: String,
    pub count: u64,
    pub first_seen: String,
    pub last_seen: String,
    pub event_refs: Vec<String>,
    #[serde(default)]
    pub refs_truncated: bool,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub src_ip: Option<String>,
    #[serde(default)]
    pub iocs: Vec<CandidateIoc>,
    /// O que a regra NÃO prova (limite do método).
    pub limitations: String,
}

impl DetectionCandidate {
    pub fn into_item(self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    /// `None` quando o item não é um candidato (ex.: evento de timeline).
    pub fn from_item(item: &Value) -> Option<Self> {
        if item.get("kind").and_then(Value::as_str) != Some(ITEM_KIND_DETECTION_CANDIDATE) {
            return None;
        }
        Self::deserialize(item).ok()
    }
}

impl NormalizedEvent {
    pub fn new(
        event_record_id: u64,
        timestamp: impl Into<String>,
        data: Value,
        evidence_id: impl Into<String>,
    ) -> Self {
        Self {
            event_record_id,
            timestamp: timestamp.into(),
            data,
            evidence_id: evidence_id.into(),
            raw_reference: Self::raw_reference_for(event_record_id),
            normalization_status: NORMALIZATION_STATUS_NORMALIZED.into(),
            event_id: None,
            provider: None,
            channel: None,
            host: None,
            user: None,
            src_ip: None,
            dst_ip: None,
            process: None,
            command_line: None,
            action: None,
            outcome: None,
            source_type: None,
        }
    }

    /// Anexa os campos estruturais extraídos do registro (F1-26).
    pub fn with_fields(mut self, fields: EventFields) -> Self {
        self.event_id = fields.event_id;
        self.provider = fields.provider;
        self.channel = fields.channel;
        self.host = fields.host;
        self.user = fields.user;
        self.src_ip = fields.src_ip;
        self.dst_ip = fields.dst_ip;
        self.process = fields.process;
        self.command_line = fields.command_line;
        self.action = fields.action;
        self.outcome = fields.outcome;
        self.source_type = fields.source_type;
        self
    }

    /// Referência estável a um registro EVTX: `record:<event_record_id>`.
    pub fn raw_reference_for(event_record_id: u64) -> String {
        format!("record:{event_record_id}")
    }

    /// Converte para o item JSON transportado em `ResultMessage::items`.
    pub fn into_item(self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    /// Lê um item de `RESULT.items`. Retorna `None` quando o item não segue
    /// este contrato — por exemplo um `YARA_MATCH`, que compartilha o mesmo
    /// `results.jsonl` mas não é um evento de timeline.
    pub fn from_item(item: &Value) -> Option<Self> {
        Self::deserialize(item).ok()
    }
}

impl Progress {
    pub fn accepted(request: &Request) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION.into(),
            schema_version: SCHEMA_VERSION.into(),
            message_type: PROGRESS_MESSAGE.into(),
            request_id: request.request_id.clone(),
            job_id: request.job_id.clone(),
            percent: 0,
            phase: "accepted".into(),
        }
    }

    /// Estado do sandbox do engine, enviado antes de qualquer leitura da
    /// evidência. `summary` é curto e sem caminhos.
    pub fn sandbox(request: &Request, summary: &str) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION.into(),
            schema_version: SCHEMA_VERSION.into(),
            message_type: PROGRESS_MESSAGE.into(),
            request_id: request.request_id.clone(),
            job_id: request.job_id.clone(),
            percent: 1,
            phase: format!("{SANDBOX_PHASE_PREFIX}{summary}"),
        }
    }

    pub fn parsing(request: &Request) -> Self {
        let phase = match request.analysis_kind.as_str() {
            ANALYSIS_KIND_YARA_SCAN => "scanning_yara",
            ANALYSIS_KIND_EVTX_DETECTION => "detecting_evtx",
            _ => "parsing_evtx",
        };
        Self {
            protocol_version: PROTOCOL_VERSION.into(),
            schema_version: SCHEMA_VERSION.into(),
            message_type: PROGRESS_MESSAGE.into(),
            request_id: request.request_id.clone(),
            job_id: request.job_id.clone(),
            percent: 10,
            phase: phase.into(),
        }
    }
}

impl ResultMessage {
    pub fn batch(request: &Request, items: Vec<Value>, completed: bool) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION.into(),
            schema_version: SCHEMA_VERSION.into(),
            message_type: RESULT_MESSAGE.into(),
            request_id: request.request_id.clone(),
            job_id: request.job_id.clone(),
            status: if completed { "COMPLETED" } else { "PARTIAL" }.into(),
            items,
        }
    }

    pub fn completed(request: &Request, items: Vec<Value>) -> Self {
        Self::batch(request, items, true)
    }
}

impl ErrorMessage {
    pub fn new(request_id: &str, job_id: &str, code: &str, message: &str) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION.into(),
            schema_version: SCHEMA_VERSION.into(),
            message_type: ERROR_MESSAGE.into(),
            request_id: request_id.into(),
            job_id: job_id.into(),
            code: code.into(),
            message: sanitize_message(message),
        }
    }
}

pub fn json_depth_exceeds_limit(bytes: &[u8], max_depth: usize) -> bool {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for &byte in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' | b'[' => {
                depth = depth.saturating_add(1);
                if depth > max_depth {
                    return true;
                }
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    false
}

fn sanitize_message(message: &str) -> String {
    message
        .chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .take(1024)
        .collect()
}

pub fn validate(value: &Value, max_line_bytes: usize) -> Result<Request, String> {
    let raw = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if raw.len() > max_line_bytes {
        return Err("protocol line exceeds configured limit".into());
    }

    let request: Request = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    if request.protocol_version != PROTOCOL_VERSION {
        return Err("unsupported protocol_version".into());
    }
    if request.schema_version != SCHEMA_VERSION {
        return Err("unsupported schema_version".into());
    }
    if request.message_type != REQUEST_MESSAGE {
        return Err("message_type must be REQUEST".into());
    }
    if !bounded_identifier(&request.request_id) || !bounded_identifier(&request.job_id) {
        return Err(
            "request_id/job_id must be non-empty, bounded, and free of control characters".into(),
        );
    }
    validate_logical_reference(&request.input_reference, "input_reference")?;
    if let Some(output) = request.output_reference.as_deref() {
        validate_logical_reference(output, "output_reference")?;
    }
    if request.max_output_bytes == 0 || request.max_output_bytes > MAX_OUTPUT_BYTES {
        return Err("max_output_bytes outside safe configured range".into());
    }
    if request.max_items == 0 || request.max_items > MAX_ITEMS {
        return Err("max_items outside safe configured range".into());
    }
    if !KNOWN_ANALYSIS_KINDS.contains(&request.analysis_kind.as_str()) {
        return Err("unsupported analysis_kind".into());
    }
    Ok(request)
}

fn bounded_identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.chars().all(|ch| !ch.is_control())
}

fn validate_logical_reference(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 1024
        || value.contains('\0')
        || value.contains('\\')
        || value.contains(':')
        || Path::new(value).is_absolute()
        || value.split('/').any(|part| part == ".." || part.is_empty())
    {
        return Err(format!(
            "{field} must be a relative logical reference without traversal"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ANALYSIS_KIND_EVTX_TIMELINE, ANALYSIS_KIND_YARA_SCAN, ErrorMessage, MAX_ITEMS,
        MAX_JSON_DEPTH, MAX_LINE_BYTES, MAX_OUTPUT_BYTES, NORMALIZATION_STATUS_NORMALIZED,
        NormalizedEvent, PROTOCOL_VERSION, Progress, ResultMessage, SCHEMA_VERSION,
        json_depth_exceeds_limit, validate,
    };
    use serde_json::json;

    fn request_json(input_reference: &str) -> serde_json::Value {
        json!({
            "protocol_version": PROTOCOL_VERSION,
            "schema_version": SCHEMA_VERSION,
            "message_type": "REQUEST",
            "request_id": "req",
            "job_id": "job",
            "input_reference": input_reference,
            "output_reference": null,
            "max_output_bytes": 1024,
            "max_items": 10,
            "analysis_kind": ANALYSIS_KIND_EVTX_TIMELINE
        })
    }

    #[test]
    fn rejects_excessive_json_depth_without_parsing() {
        let shallow = b"{{\"a\":1}}";
        assert!(!json_depth_exceeds_limit(shallow, MAX_JSON_DEPTH));
        let deep = format!(
            "{}0{}",
            "[".repeat(MAX_JSON_DEPTH + 1),
            "]".repeat(MAX_JSON_DEPTH + 1)
        );
        assert!(json_depth_exceeds_limit(deep.as_bytes(), MAX_JSON_DEPTH));
        let quoted = format!("\"{}\"", "[".repeat(MAX_JSON_DEPTH + 1));
        assert!(!json_depth_exceeds_limit(quoted.as_bytes(), MAX_JSON_DEPTH));
    }

    #[test]
    fn default_ceilings_do_not_limit_realistic_outputs_but_zero_is_still_invalid() {
        // ADR-0020: o teto padrão não corta saída real (dezenas de milhões de
        // eventos / dezenas de GiB), mas 0 e valores acima do teto continuam
        // inválidos.
        const {
            assert!(MAX_OUTPUT_BYTES >= 1 << 50);
            assert!(MAX_ITEMS >= 1 << 50);
            // Precisa caber na canonicalização JSON da cadeia de auditoria.
            assert!(MAX_OUTPUT_BYTES < 1 << 53 && MAX_ITEMS < 1 << 53);
        }
        let mut request = request_json("evidence/demo.evtx");
        request["max_output_bytes"] = serde_json::json!(MAX_OUTPUT_BYTES);
        request["max_items"] = serde_json::json!(MAX_ITEMS);
        assert!(validate(&request, MAX_LINE_BYTES).is_ok());
        request["max_items"] = serde_json::json!(0);
        assert!(validate(&request, MAX_LINE_BYTES).is_err());
    }

    #[test]
    fn rejects_traversal_and_oversized_lines() {
        assert!(validate(&request_json("../secret"), MAX_LINE_BYTES).is_err());
        assert!(validate(&request_json("C:secret.evtx"), MAX_LINE_BYTES).is_err());
        assert!(validate(&request_json("evidence/file:stream"), MAX_LINE_BYTES).is_err());
        let oversized = request_json(&"a".repeat(MAX_LINE_BYTES));
        assert!(validate(&oversized, MAX_LINE_BYTES).is_err());
    }

    #[test]
    fn concrete_messages_serialize_without_trait_objects() {
        let request =
            validate(&request_json("evidence/demo.evtx"), MAX_LINE_BYTES).expect("valid request");
        let _ = serde_json::to_string(&Progress::accepted(&request)).expect("progress serializes");
        let _ = serde_json::to_string(&ResultMessage::completed(&request, vec![]))
            .expect("result serializes");
        let error = ErrorMessage::new("req", "job", "ERR", "bad");
        let json = serde_json::to_string(&error).expect("error serializes");
        assert!(json.contains("ERROR"));
    }

    #[test]
    fn accepts_known_analysis_kinds_and_rejects_unknown_ones() {
        let mut yara = request_json("evidence/sample.bin");
        yara["analysis_kind"] = json!(ANALYSIS_KIND_YARA_SCAN);
        assert!(validate(&yara, MAX_LINE_BYTES).is_ok());

        let mut bogus = request_json("evidence/sample.bin");
        bogus["analysis_kind"] = json!("NOT_A_REAL_KIND");
        assert!(validate(&bogus, MAX_LINE_BYTES).is_err());
    }

    #[test]
    fn the_detection_kind_is_accepted_and_has_its_own_phase() {
        let mut detection = request_json("evidence/sample.evtx");
        detection["analysis_kind"] = json!(super::ANALYSIS_KIND_EVTX_DETECTION);
        let request = validate(&detection, MAX_LINE_BYTES).expect("detection kind is valid");
        assert_eq!(Progress::parsing(&request).phase, "detecting_evtx");
    }

    #[test]
    fn the_sandbox_status_travels_in_a_progress_with_a_recognisable_prefix() {
        let request = validate(&request_json("evidence/sample.evtx"), MAX_LINE_BYTES).expect("ok");
        let progress = Progress::sandbox(&request, "landlock=abi7;seccomp=on");
        assert!(progress.phase.starts_with(super::SANDBOX_PHASE_PREFIX));
        assert_eq!(progress.phase, "sandbox:landlock=abi7;seccomp=on");
        assert_eq!(progress.message_type, super::PROGRESS_MESSAGE);
    }

    #[test]
    fn structured_fields_are_omitted_when_empty_and_present_when_set() {
        let bare = sample_event().into_item().expect("serializes");
        for key in [
            "event_id",
            "user",
            "src_ip",
            "host",
            "outcome",
            "source_type",
        ] {
            assert!(
                bare.get(key).is_none(),
                "{key} must not be serialized when empty"
            );
        }
        let full = sample_event()
            .with_fields(super::EventFields {
                event_id: Some("4624".into()),
                user: Some("fsir".into()),
                src_ip: Some("10.0.0.5".into()),
                host: Some("DC01".into()),
                outcome: Some("success".into()),
                source_type: Some("EVTX".into()),
                ..Default::default()
            })
            .into_item()
            .expect("serializes");
        assert_eq!(full["event_id"], "4624");
        assert_eq!(full["src_ip"], "10.0.0.5");
        assert!(full.get("dst_ip").is_none());
        let back = NormalizedEvent::from_item(&full).expect("round trip");
        assert_eq!(back.user.as_deref(), Some("fsir"));
        assert_eq!(back.outcome.as_deref(), Some("success"));
    }

    #[test]
    fn items_persisted_before_the_structured_fields_are_still_readable() {
        // Forma exata da versão 3 do esquema: seis chaves, sem campo novo.
        let v3 = json!({
            "event_record_id": 9,
            "timestamp": "2026-01-02T03:04:05Z",
            "data": {"Event": {"System": {"EventID": 4625}}},
            "evidence_id": "evidence/x.evtx",
            "raw_reference": "record:9",
            "normalization_status": "NORMALIZED"
        });
        let event = NormalizedEvent::from_item(&v3).expect("v3 item is accepted");
        assert_eq!(event.event_record_id, 9);
        assert!(event.event_id.is_none() && event.user.is_none() && event.source_type.is_none());
    }

    #[test]
    fn a_detection_candidate_round_trips_and_is_not_confused_with_an_event() {
        let candidate = super::DetectionCandidate {
            kind: super::ITEM_KIND_DETECTION_CANDIDATE.into(),
            evidence_id: "evidence/x.evtx".into(),
            rule_id: "R-AUTH-001".into(),
            rule_version: "1".into(),
            rule_set_version: "2026.10.1".into(),
            title: "Rajada de falhas de logon".into(),
            description: "d".into(),
            tactic: "CREDENTIAL_ACCESS".into(),
            technique_id: "T1110".into(),
            sub_technique_id: None,
            severity: "MEDIUM".into(),
            confidence: "LOW".into(),
            candidate_key: "k".into(),
            aggregation: "conta=x".into(),
            count: 12,
            first_seen: "2026-01-02T03:04:05Z".into(),
            last_seen: "2026-01-02T03:05:05Z".into(),
            event_refs: vec!["record:1".into()],
            refs_truncated: false,
            host: None,
            user: Some("x".into()),
            src_ip: Some("203.0.113.7".into()),
            iocs: vec![super::CandidateIoc {
                ioc_type: "IP".into(),
                value: "203.0.113.7".into(),
                event_ref: "record:1".into(),
            }],
            limitations: "l".into(),
        };
        let item = candidate.clone().into_item().expect("serializes");
        assert_eq!(super::DetectionCandidate::from_item(&item), Some(candidate));
        assert_eq!(NormalizedEvent::from_item(&item), None);
        assert_eq!(
            super::DetectionCandidate::from_item(&sample_event().into_item().unwrap()),
            None
        );
    }

    #[test]
    fn errors_are_sanitized_and_bounded() {
        let message = "secret\npassword\u{0}\t".to_string() + &"x".repeat(2000);
        let err = ErrorMessage::new("req", "job", "ERR", &message);
        assert!(err.message.len() <= 1024);
        assert!(!err.message.contains('\u{0}'));
    }
    fn sample_event() -> NormalizedEvent {
        NormalizedEvent::new(
            42,
            "2025-01-02T03:04:05Z",
            json!({"Event": {"System": {"EventID": 4624}}}),
            "ev_01",
        )
    }

    #[test]
    fn normalized_event_uses_the_record_reference_as_stable_identity() {
        assert_eq!(NormalizedEvent::raw_reference_for(42), "record:42");
        let item = sample_event().into_item().expect("event serializes");
        assert_eq!(item["event_record_id"], 42);
        assert!(item.get("record_offset").is_none());
        assert_eq!(item["raw_reference"], "record:42");
        assert_eq!(item["evidence_id"], "ev_01");
        assert_eq!(
            item["normalization_status"],
            NORMALIZATION_STATUS_NORMALIZED
        );
    }

    #[test]
    fn normalized_event_wire_format_keeps_exactly_the_historical_keys() {
        // Trava a compatibilidade com o `json!` que o engine emitia antes do
        // tipo existir: se uma chave for adicionada/removida, SCHEMA_VERSION
        // precisa mudar junto.
        let item = sample_event().into_item().expect("event serializes");
        let mut keys: Vec<&str> = item
            .as_object()
            .expect("item is an object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "data",
                "event_record_id",
                "evidence_id",
                "normalization_status",
                "raw_reference",
                "timestamp",
            ]
        );
    }

    #[test]
    fn normalized_event_round_trips_through_an_item() {
        let event = sample_event();
        let item = event.clone().into_item().expect("event serializes");
        assert_eq!(NormalizedEvent::from_item(&item), Some(event));
    }

    #[test]
    fn normalized_event_reading_ignores_unknown_fields_and_defaults_the_status() {
        let item = json!({
            "event_record_id": 7,
            "timestamp": "2025-01-02T03:04:05Z",
            "evidence_id": "ev_01",
            "raw_reference": "record:7",
            "record_offset": 9999,
            "data": {}
        });
        let event = NormalizedEvent::from_item(&item).expect("legacy-shaped item is accepted");
        assert_eq!(event.normalization_status, NORMALIZATION_STATUS_NORMALIZED);
    }

    #[test]
    fn normalized_event_reading_rejects_items_that_are_not_timeline_events() {
        let yara_match = json!({
            "kind": "YARA_MATCH",
            "rule_id": "eicar_test_string",
            "evidence_id": "evidence/sample.bin"
        });
        assert_eq!(NormalizedEvent::from_item(&yara_match), None);
        let missing_timestamp = json!({"event_record_id": 1, "evidence_id": "e", "raw_reference": "record:1", "data": {}});
        assert_eq!(NormalizedEvent::from_item(&missing_timestamp), None);
    }
}
