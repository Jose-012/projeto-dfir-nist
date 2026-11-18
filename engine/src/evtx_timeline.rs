//! Regras específicas de EVTX → evento de timeline: aceitação por extensão e
//! escolha do timestamp. A conversão para o contrato tipado é
//! `dfir_protocol::NormalizedEvent`.

use serde_json::Value;
use std::{fs::File, io::Read, path::Path};

/// Tamanho de um chunk EVTX (64 KiB). O arquivo é: cabeçalho (normalmente
/// 4096 bytes) + N chunks de 64 KiB.
const EVTX_CHUNK_BYTES: u64 = 65_536;
const EVTX_DEFAULT_HEADER_BYTES: u64 = 4_096;

/// Campos do cabeçalho do arquivo EVTX que permitem detectar um arquivo
/// truncado (AUD-36). **Heurística, não prova**: logs "sujos" (flag de dirty,
/// comuns em logs ao vivo) têm contadores defasados no cabeçalho, então estes
/// sinais só geram AVISO, nunca falha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvtxHeaderInfo {
    pub header_block_size: u64,
    pub chunk_count: u64,
    pub next_record_id: u64,
    pub file_len: u64,
}

/// Lê o cabeçalho; `None` se não for um EVTX legível (o parser reporta o erro).
pub fn read_header_info(path: &Path) -> Option<EvtxHeaderInfo> {
    let mut header = [0_u8; 64];
    File::open(path).ok()?.read_exact(&mut header).ok()?;
    if &header[..8] != b"ElfFile\0" {
        return None;
    }
    let next_record_id = u64::from_le_bytes(header[0x18..0x20].try_into().ok()?);
    let header_block_size = u64::from(u16::from_le_bytes(header[0x28..0x2A].try_into().ok()?));
    let chunk_count = u64::from(u16::from_le_bytes(header[0x2A..0x2C].try_into().ok()?));
    let file_len = std::fs::metadata(path).ok()?.len();
    Some(EvtxHeaderInfo {
        header_block_size: if header_block_size == 0 {
            EVTX_DEFAULT_HEADER_BYTES
        } else {
            header_block_size
        },
        chunk_count,
        next_record_id,
        file_len,
    })
}

/// Aviso quando o arquivo não é um número inteiro de chunks depois do
/// cabeçalho ou tem menos chunks completos do que o cabeçalho declara.
pub fn truncation_warning(info: &EvtxHeaderInfo) -> Option<String> {
    let body = info.file_len.saturating_sub(info.header_block_size);
    let complete = body / EVTX_CHUNK_BYTES;
    let trailing = body % EVTX_CHUNK_BYTES;
    if trailing == 0 && complete >= info.chunk_count {
        return None;
    }
    Some(format!(
        "EVTX file looks truncated: {complete} complete chunk(s) of {declared} declared by the header, {trailing} trailing byte(s) of an incomplete chunk; records in the missing data were not read",
        declared = info.chunk_count
    ))
}

/// Aviso quando o cabeçalho indica registros além do último lido. Cabeçalho
/// defasado só torna o aviso menos sensível (nunca gera falso positivo).
pub fn missing_records_warning(info: &EvtxHeaderInfo, max_parsed: Option<u64>) -> Option<String> {
    let last_expected = info.next_record_id.checked_sub(1)?;
    if last_expected == 0 {
        return None;
    }
    match max_parsed {
        Some(parsed) if parsed >= last_expected => None,
        Some(parsed) => Some(format!(
            "EVTX header indicates records up to #{last_expected} but the last record read was #{parsed}: records may be missing"
        )),
        None => Some(format!(
            "EVTX header indicates records up to #{last_expected} but no record could be read"
        )),
    }
}

pub fn has_evtx_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("evtx"))
}

pub fn event_system_time(data: &Value) -> Option<String> {
    data.pointer("/Event/System/TimeCreated/#attributes/SystemTime")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|value| !value.is_empty())
}

pub fn event_timestamp(record_timestamp: &str, data: &Value) -> String {
    event_system_time(data).unwrap_or_else(|| record_timestamp.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        EvtxHeaderInfo, event_system_time, event_timestamp, has_evtx_extension,
        missing_records_warning, read_header_info, truncation_warning,
    };
    use std::{io::Write, path::Path};

    fn info(file_len: u64, chunk_count: u64, next_record_id: u64) -> EvtxHeaderInfo {
        EvtxHeaderInfo {
            header_block_size: 4096,
            chunk_count,
            next_record_id,
            file_len,
        }
    }

    #[test]
    fn whole_chunks_matching_the_header_are_not_truncated() {
        assert!(truncation_warning(&info(4096 + 3 * 65_536, 3, 10)).is_none());
        // Cabeçalho defasado (menos chunks declarados que os presentes): sem aviso.
        assert!(truncation_warning(&info(4096 + 5 * 65_536, 3, 10)).is_none());
    }

    #[test]
    fn partial_trailing_chunk_or_missing_chunks_are_reported() {
        let partial = truncation_warning(&info(4096 + 2 * 65_536 + 100, 3, 10)).expect("partial");
        assert!(partial.contains("2 complete chunk(s) of 3"), "{partial}");
        assert!(partial.contains("100 trailing byte(s)"), "{partial}");
        assert!(truncation_warning(&info(4096 + 2 * 65_536, 3, 10)).is_some());
        assert!(truncation_warning(&info(100, 3, 10)).is_some());
    }

    #[test]
    fn missing_records_are_reported_only_when_the_header_is_ahead_of_the_parse() {
        let header = info(0, 0, 101); // últimos registros esperados: até #100
        assert!(missing_records_warning(&header, Some(100)).is_none());
        assert!(
            missing_records_warning(&header, Some(150)).is_none(),
            "cabeçalho defasado"
        );
        assert!(missing_records_warning(&header, Some(60)).is_some());
        assert!(missing_records_warning(&header, None).is_some());
        assert!(
            missing_records_warning(&info(0, 0, 1), None).is_none(),
            "log vazio"
        );
    }

    #[test]
    fn header_fields_are_read_from_the_file() {
        let mut bytes = vec![0_u8; 64];
        bytes[..8].copy_from_slice(b"ElfFile\0");
        bytes[0x18..0x20].copy_from_slice(&77_u64.to_le_bytes());
        bytes[0x28..0x2A].copy_from_slice(&4096_u16.to_le_bytes());
        bytes[0x2A..0x2C].copy_from_slice(&26_u16.to_le_bytes());
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        file.write_all(&bytes).expect("write");
        let read = read_header_info(file.path()).expect("header");
        assert_eq!(
            (read.chunk_count, read.next_record_id, read.file_len),
            (26, 77, 64)
        );
        let mut not_evtx = tempfile::NamedTempFile::new().expect("temp file");
        not_evtx.write_all(&[7_u8; 64]).expect("write");
        assert!(read_header_info(not_evtx.path()).is_none());
    }

    #[test]
    fn evtx_detection_is_case_insensitive() {
        assert!(has_evtx_extension(Path::new("security.evtx")));
        assert!(has_evtx_extension(Path::new("security.EVTX")));
        assert!(!has_evtx_extension(Path::new("security.evtx.zip")));
    }

    #[test]
    fn source_event_time_precedes_record_header_timestamp() {
        let data = serde_json::json!({
            "Event": {
                "System": {
                    "TimeCreated": {
                        "#attributes": {
                            "SystemTime": "2019-04-27T19:31:15.1234567Z"
                        }
                    }
                }
            }
        });
        assert_eq!(
            event_system_time(&data).as_deref(),
            Some("2019-04-27T19:31:15.1234567Z")
        );
    }

    #[test]
    fn timestamp_prefers_event_system_time_over_record_header_time() {
        let data = serde_json::json!({
            "Event": {
                "System": {
                    "TimeCreated": {
                        "#attributes": {
                            "SystemTime": "2019-04-27T19:31:15.1234567Z"
                        }
                    }
                }
            }
        });
        assert_eq!(
            event_timestamp("1601-01-01T00:00:00Z", &data),
            "2019-04-27T19:31:15.1234567Z"
        );
    }

    #[test]
    fn timestamp_falls_back_to_record_header_time_when_source_time_is_absent() {
        let data = serde_json::json!({
            "Event": {
                "System": {}
            }
        });
        assert_eq!(
            event_timestamp("2019-04-27T19:31:15Z", &data),
            "2019-04-27T19:31:15Z"
        );
    }

    #[test]
    fn empty_source_event_time_is_ignored() {
        let data = serde_json::json!({
            "Event": {
                "System": {
                    "TimeCreated": {
                        "#attributes": {
                            "SystemTime": ""
                        }
                    }
                }
            }
        });
        assert!(event_system_time(&data).is_none());
    }
}
