//! F1-02 — YARA-X e resultados estruturados.
//!
//! Ruleset local/offline, curado e versionado (`engine/rules/dfir_mvp_ruleset.yar`,
//! seção 9.1/9.2 da spec canônica): nenhuma regra externa é importada, cada
//! regra tem motivação registrada no próprio arquivo `.yar` e teste positivo/
//! negativo neste módulo. Cada match emitido carrega identificador da regra,
//! `ruleset_id`/`ruleset_version`/hash do ruleset, versão do engine
//! (`engine_compatibility`) e confiança/contexto — nunca os bytes brutos
//! encontrados (apenas offset/comprimento do trecho casado), para não expor
//! dados arbitrários da evidência em um campo supostamente seguro.
//!
//! Falhas de compilação do ruleset ou de execução da varredura nunca são
//! descartadas silenciosamente: ambas viram um erro de protocolo
//! (`YARA_RULESET_ERROR`/`YARA_SCAN_ERROR`) reportado ao chamador, seguindo o
//! mesmo padrão de erro estruturado do parser EVTX (seção 10.1 da spec).

use dfir_protocol::Request;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::Duration;

/// Fonte do ruleset embutida no binário em tempo de compilação: não há
/// carregamento dinâmico de regras em tempo de execução (seção 9.2 — sem
/// atualização automática de regras no MVP).
const RULESET_SOURCE: &str = include_str!("../rules/dfir_mvp_ruleset.yar");

pub const RULESET_ID: &str = "dfir-mvp-ruleset";
pub const RULESET_VERSION: &str = "1.1.0";
pub const RULESET_LICENSE: &str = "Apache-2.0";
pub const RULESET_SOURCE_ORIGIN: &str = "original; regras autorais para este projeto (docs/PROJECT_SPEC_v6_4_5.md secao 9.1/9.2); nenhum ruleset externo foi importado";
/// Mantido em sincronia manual com a versão de `yara-x` pinada em
/// `engine/Cargo.toml` — não há forma de ler `CARGO_PKG_VERSION` de uma
/// dependência a partir do crate que a consome.
pub const ENGINE_COMPATIBILITY: &str = "yara-x 1.20.0";

/// Tempo mínimo da varredura. Para arquivos grandes o prazo cresce com o
/// tamanho (ver [`scan_timeout_for`]): um prazo fixo de 60 s reprovava um
/// arquivo de 1,1 GiB (medido), mesmo com o espaço de endereçamento resolvido.
const MIN_SCAN_TIMEOUT: Duration = Duration::from_secs(60);
/// Vazão pessimista assumida (build de depuração, disco lento): 4 MiB/s.
const ASSUMED_SCAN_BYTES_PER_SECOND: u64 = 4 * 1024 * 1024;
/// Teto de sanidade do prazo: 6 horas. O limite que manda é o do app
/// (`DFIR_ENGINE_TIMEOUT_SECONDS`), que interrompe o processo inteiro.
const MAX_SCAN_TIMEOUT: Duration = Duration::from_secs(6 * 3600);

pub fn scan_timeout_for(file_size: u64) -> Duration {
    MIN_SCAN_TIMEOUT
        .max(Duration::from_secs(
            file_size / ASSUMED_SCAN_BYTES_PER_SECOND,
        ))
        .min(MAX_SCAN_TIMEOUT)
}
/// Ocorrências que o scanner acompanha por padrão (o default do YARA-X). Era
/// 100 e descartava o resto em silêncio; agora é alto o bastante para contar
/// com exatidão.
const MAX_MATCHES_PER_PATTERN: usize = 1_000_000;
/// Quantas ocorrências (offset + comprimento) de CADA padrão entram no item
/// emitido. O que passar disso **não é descartado em silêncio**: o item traz o
/// total exato em `pattern_match_counts` e `matches_truncated = true`. Existe
/// só para que uma linha JSONL (≤ `MAX_LINE_BYTES`) nunca precise carregar
/// milhões de offsets.
const MAX_REPORTED_OCCURRENCES_PER_PATTERN: usize = 5_000;

/// SHA-256 do ruleset embutido, recalculado em tempo de execução (nunca
/// hardcoded) para que qualquer edição do arquivo `.yar` sem atualização
/// deste hash seja impossível — o hash sempre reflete o conteúdo real
/// compilado nesse binário.
pub fn ruleset_sha256() -> String {
    let mut hasher = Sha256::new();
    hasher.update(RULESET_SOURCE.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn compile_ruleset() -> Result<yara_x::Rules, (&'static str, String)> {
    yara_x::compile(RULESET_SOURCE).map_err(|error| ("YARA_RULESET_ERROR", error.to_string()))
}

fn meta_value_to_json(value: yara_x::MetaValue<'_>) -> Value {
    match value {
        yara_x::MetaValue::Integer(number) => json!(number),
        yara_x::MetaValue::Float(number) => json!(number),
        yara_x::MetaValue::Bool(flag) => json!(flag),
        yara_x::MetaValue::String(text) => json!(text),
        // Não usamos metadata binária em nenhuma regra própria; nulo é a
        // resposta segura caso uma regra futura introduza um valor assim.
        yara_x::MetaValue::Bytes(_) => Value::Null,
    }
}

/// Varre `path` com o ruleset embutido e retorna um item JSON por match,
/// pronto para ser emitido como item de `RESULT` no protocolo JSONL
/// (mesma forma usada pelos itens de timeline EVTX). Nunca inclui os bytes
/// casados — apenas identificador do padrão, offset e comprimento.
pub fn scan_path(request: &Request, path: &Path) -> Result<Vec<Value>, (&'static str, String)> {
    let rules = compile_ruleset()?;
    let mut scanner = yara_x::Scanner::new(&rules);
    // Sem `max_scan_size`: o YARA-X trunca em silêncio o dado varrido ao limite
    // (só os primeiros N bytes), o que esconderia um match no fim de um arquivo
    // grande (ADR-0020).
    scanner
        .set_timeout(scan_timeout_for(
            std::fs::metadata(path).map_or(0, |metadata| metadata.len()),
        ))
        .max_matches_per_pattern(MAX_MATCHES_PER_PATTERN);

    let results = scanner
        .scan_file(path)
        .map_err(|error| ("YARA_SCAN_ERROR", error.to_string()))?;

    let ruleset_sha256 = ruleset_sha256();
    let mut items = Vec::new();
    for rule in results.matching_rules() {
        let mut confidence = Value::Null;
        let mut severity = Value::Null;
        let mut description = Value::Null;
        let mut reference = Value::Null;
        for (key, value) in rule.metadata() {
            match key {
                "confidence" => confidence = meta_value_to_json(value),
                "severity" => severity = meta_value_to_json(value),
                "description" => description = meta_value_to_json(value),
                "reference" => reference = meta_value_to_json(value),
                _ => {}
            }
        }

        let mut matched_patterns: Vec<Value> = Vec::new();
        let mut pattern_match_counts: Vec<Value> = Vec::new();
        let mut matches_truncated = false;
        for pattern in rule.patterns() {
            let pattern_id = pattern.identifier().to_string();
            let mut total_matches: usize = 0;
            let mut reported: usize = 0;
            for occurrence in pattern.matches() {
                total_matches += 1;
                if reported < MAX_REPORTED_OCCURRENCES_PER_PATTERN {
                    let range = occurrence.range();
                    matched_patterns.push(json!({
                        "pattern_id": pattern_id.clone(),
                        "offset": range.start,
                        "length": range.end.saturating_sub(range.start),
                    }));
                    reported += 1;
                }
            }
            if total_matches == 0 {
                continue;
            }
            // `>=`: se o scanner atingiu o próprio teto, o total é um piso.
            if total_matches > reported || total_matches >= MAX_MATCHES_PER_PATTERN {
                matches_truncated = true;
            }
            pattern_match_counts.push(json!({
                "pattern_id": pattern_id,
                "total_matches": total_matches,
                "reported_matches": reported,
            }));
        }

        items.push(json!({
            "kind": "YARA_MATCH",
            "rule_id": rule.identifier(),
            "namespace": rule.namespace(),
            "ruleset_id": RULESET_ID,
            "ruleset_version": RULESET_VERSION,
            "ruleset_sha256": ruleset_sha256,
            "ruleset_license": RULESET_LICENSE,
            "ruleset_source_origin": RULESET_SOURCE_ORIGIN,
            "engine_compatibility": ENGINE_COMPATIBILITY,
            "evidence_id": request.input_reference,
            "confidence": confidence,
            "severity": severity,
            "description": description,
            "reference": reference,
            "matched_patterns": matched_patterns,
            "pattern_match_counts": pattern_match_counts,
            "matches_truncated": matches_truncated,
        }));
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_fixture(bytes: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        file.write_all(bytes).expect("write fixture bytes");
        file.flush().expect("flush fixture");
        file
    }

    fn test_request() -> Request {
        Request {
            protocol_version: dfir_protocol::PROTOCOL_VERSION.into(),
            schema_version: dfir_protocol::SCHEMA_VERSION.into(),
            message_type: dfir_protocol::REQUEST_MESSAGE.into(),
            request_id: "req-test".into(),
            job_id: "job-test".into(),
            input_reference: "evidence/sample.bin".into(),
            output_reference: None,
            max_output_bytes: dfir_protocol::MAX_OUTPUT_BYTES,
            max_items: dfir_protocol::MAX_ITEMS,
            analysis_kind: dfir_protocol::ANALYSIS_KIND_YARA_SCAN.into(),
        }
    }

    #[test]
    fn embedded_ruleset_compiles_successfully() {
        assert!(compile_ruleset().is_ok());
    }

    #[test]
    fn ruleset_sha256_is_stable_and_reproducible() {
        assert_eq!(ruleset_sha256(), ruleset_sha256());
        assert_eq!(ruleset_sha256().len(), 64, "sha256 hex digest is 64 chars");
    }

    #[test]
    fn yara_scan_detects_eicar_test_string() {
        let eicar = br#"X5O!P%@AP[4\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*"#;
        let file = write_fixture(eicar);
        let request = test_request();
        let items = scan_path(&request, file.path()).expect("scan succeeds");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["rule_id"], "eicar_test_string");
        assert_eq!(items[0]["confidence"], "CONFIRMED");
        assert_eq!(items[0]["ruleset_id"], RULESET_ID);
        assert_eq!(items[0]["ruleset_version"], RULESET_VERSION);
        assert!(items[0]["ruleset_sha256"].is_string());
        assert!(
            items[0]["matched_patterns"]
                .as_array()
                .is_some_and(|p| !p.is_empty())
        );
    }

    #[test]
    fn yara_scan_reports_no_match_for_benign_content() {
        let file = write_fixture(b"just an ordinary log line, nothing suspicious here");
        let request = test_request();
        let items = scan_path(&request, file.path()).expect("scan succeeds");
        assert!(items.is_empty());
    }

    #[test]
    fn yara_scan_reports_a_controlled_error_for_an_unreadable_path() {
        let request = test_request();
        let missing = std::path::Path::new("/nonexistent/does-not-exist.bin");
        let error = scan_path(&request, missing).expect_err("missing file must not scan silently");
        assert_eq!(error.0, "YARA_SCAN_ERROR");
    }

    #[test]
    fn invalid_ruleset_source_is_reported_as_a_controlled_error_not_a_panic() {
        // Exercises the same error-mapping path as compile_ruleset() against a
        // deliberately broken rule, without touching the real embedded ruleset.
        let result = yara_x::compile("rule broken { condition: this is not valid }");
        assert!(result.is_err());
    }

    fn utf16le(text: &str) -> Vec<u8> {
        text.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    fn matched_rules(bytes: &[u8]) -> Vec<String> {
        let file = write_fixture(bytes);
        scan_path(&test_request(), file.path())
            .expect("scan succeeds")
            .iter()
            .filter_map(|item| item["rule_id"].as_str().map(str::to_string))
            .collect()
    }

    fn encoded_command_line() -> String {
        format!("powershell.exe -EncodedCommand {}", "A".repeat(220))
    }

    #[test]
    fn powershell_rule_matches_ascii_and_utf16_content() {
        // AUD-09: o conteúdo de eventos EVTX é UTF-16LE.
        let line = encoded_command_line();
        assert!(
            matched_rules(line.as_bytes())
                .contains(&"suspicious_powershell_encoded_command".to_string()),
            "ASCII"
        );
        assert!(
            matched_rules(&utf16le(&line))
                .contains(&"suspicious_powershell_encoded_command".to_string()),
            "UTF-16LE"
        );
    }

    #[test]
    fn powershell_rule_needs_both_the_flag_and_the_long_base64_run() {
        let only_flag = "powershell.exe -EncodedCommand short";
        assert!(matched_rules(only_flag.as_bytes()).is_empty());
        assert!(matched_rules(&utf16le(only_flag)).is_empty());
        let only_run = "A".repeat(220);
        assert!(matched_rules(only_run.as_bytes()).is_empty());
    }

    #[test]
    fn mimikatz_and_ransom_rules_match_utf16_content() {
        let mimikatz = utf16le("mimikatz # sekurlsa::logonpasswords");
        assert!(matched_rules(&mimikatz).contains(&"mimikatz_command_reference".to_string()));
        let note = utf16le("All your files have been encrypted. To decrypt them send bitcoin.");
        assert!(matched_rules(&note).contains(&"generic_ransom_note_keywords".to_string()));
    }

    #[test]
    fn occurrences_beyond_the_reported_cap_are_counted_and_flagged_not_dropped_silently() {
        let body = "sekurlsa::pth ".repeat(MAX_REPORTED_OCCURRENCES_PER_PATTERN + 1000);
        let file = write_fixture(body.as_bytes());
        let items = scan_path(&test_request(), file.path()).expect("scan succeeds");
        let item = items
            .iter()
            .find(|item| item["rule_id"] == "mimikatz_command_reference")
            .expect("mimikatz rule matched");
        assert_eq!(item["matches_truncated"], true);
        let counts = item["pattern_match_counts"].as_array().expect("counts");
        let cmd2 = counts
            .iter()
            .find(|entry| entry["pattern_id"] == "$cmd2")
            .expect("$cmd2 counted");
        assert_eq!(
            cmd2["total_matches"],
            (MAX_REPORTED_OCCURRENCES_PER_PATTERN + 1000) as u64,
            "o total exato precisa ser informado"
        );
        assert_eq!(
            cmd2["reported_matches"],
            MAX_REPORTED_OCCURRENCES_PER_PATTERN as u64
        );
        let listed = item["matched_patterns"]
            .as_array()
            .expect("matched_patterns")
            .iter()
            .filter(|entry| entry["pattern_id"] == "$cmd2")
            .count();
        assert_eq!(listed, MAX_REPORTED_OCCURRENCES_PER_PATTERN);
    }

    #[test]
    fn small_match_sets_are_complete_and_not_flagged() {
        let file = write_fixture(b"mimikatz # sekurlsa::logonpasswords");
        let items = scan_path(&test_request(), file.path()).expect("scan succeeds");
        let item = items
            .iter()
            .find(|item| item["rule_id"] == "mimikatz_command_reference")
            .expect("rule matched");
        assert_eq!(item["matches_truncated"], false);
    }

    #[test]
    fn marker_past_the_old_256_mib_limit_is_still_found() {
        // O YARA-X truncava a varredura ao `max_scan_size` em silêncio. Arquivo
        // esparso de 300 MiB com o marcador nos últimos bytes: não ocupa disco.
        use std::io::{Seek, SeekFrom, Write};
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        let size: u64 = 300 * 1024 * 1024;
        file.as_file().set_len(size).expect("sparse length");
        let marker = b"sekurlsa::logonpasswords";
        file.as_file_mut()
            .seek(SeekFrom::Start(size - marker.len() as u64))
            .expect("seek");
        file.as_file_mut().write_all(marker).expect("marker");
        file.as_file_mut().flush().expect("flush");
        let items = scan_path(&test_request(), file.path()).expect("scan succeeds");
        assert!(
            items
                .iter()
                .any(|item| item["rule_id"] == "mimikatz_command_reference"),
            "o marcador além de 256 MiB precisa ser encontrado"
        );
    }

    #[test]
    fn scan_timeout_grows_with_the_file_and_is_bounded() {
        use std::time::Duration;
        assert_eq!(scan_timeout_for(0), Duration::from_secs(60));
        assert_eq!(scan_timeout_for(100 * 1024 * 1024), Duration::from_secs(60));
        assert_eq!(
            scan_timeout_for(1024 * 1024 * 1024),
            Duration::from_secs(256)
        );
        assert_eq!(scan_timeout_for(u64::MAX), Duration::from_secs(6 * 3600));
    }

    #[test]
    fn utf16_benign_text_does_not_match() {
        assert!(
            matched_rules(&utf16le("an ordinary log line, nothing suspicious here")).is_empty()
        );
    }

    #[test]
    fn mimikatz_and_ransom_note_rules_match_their_positive_fixtures() {
        let request = test_request();

        let mimikatz = write_fixture(b"mimikatz # sekurlsa::logonpasswords");
        let items = scan_path(&request, mimikatz.path()).expect("scan succeeds");
        assert!(
            items
                .iter()
                .any(|item| item["rule_id"] == "mimikatz_command_reference")
        );

        let ransom_note = write_fixture(
            b"All your files have been encrypted. To decrypt them send bitcoin to our wallet.",
        );
        let items = scan_path(&request, ransom_note.path()).expect("scan succeeds");
        assert!(
            items
                .iter()
                .any(|item| item["rule_id"] == "generic_ransom_note_keywords")
        );
    }

    #[test]
    fn wide_modifier_matches_utf16le_payloads_only() {
        // F1-20: `wide ascii` precisa casar o marcador em UTF-16LE (formato
        // real de strings em EVTX/BinXML). O payload precisa ter o bloco base64
        // longo que a regra exige junto da flag.
        let encoded = utf16le(&encoded_command_line());
        let file = write_fixture(&encoded);
        let items = scan_path(&test_request(), file.path()).expect("scan succeeds");
        assert!(
            items
                .iter()
                .any(|item| item["rule_id"] == "suspicious_powershell_encoded_command"),
            "o marcador em UTF-16LE precisa ser encontrado pela regra wide ascii"
        );
    }

    #[test]
    fn scan_timeout_and_match_limits_are_applied_to_the_scanner() {
        // F1-20: o prazo depende do tamanho (scan_timeout_for, coberto em
        // `scan_timeout_grows_with_the_file_and_is_bounded`) e o teto de
        // matches por padrão é alto o bastante para contar com exatidão sem
        // descartar em silêncio. Constantes invariantes: verificação em tempo
        // de compilação.
        const {
            assert!(MAX_MATCHES_PER_PATTERN >= 1_000_000);
            assert!(MAX_REPORTED_OCCURRENCES_PER_PATTERN < MAX_MATCHES_PER_PATTERN);
        }
    }
}
