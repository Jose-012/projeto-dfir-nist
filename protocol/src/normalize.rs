//! Normalização de um registro EVTX em campos estruturais (F1-26).
//!
//! O parser entrega cada registro como JSON (`Event/System/...`,
//! `Event/EventData/...`). Este módulo extrai o que a investigação consulta —
//! identificador do evento, provedor, canal, máquina, conta, IPs, processo,
//! linha de comando, ação e resultado — **sem interpretar nada além do que o
//! registro diz**: campo ausente continua ausente (`None`), nunca é inferido.
//!
//! Vive no crate de protocolo porque o engine (que escreve os campos nos
//! resultados) e o app (que os relê de resultados antigos, sem os campos)
//! precisam extrair **exatamente** o mesmo. As regras de detecção do engine
//! usam os mesmos extratores: regra e timeline leem o evento do mesmo jeito.

use crate::EventFields;
use serde_json::Value;

/// Maior tamanho guardado para um campo de texto comum.
const MAX_FIELD_CHARS: usize = 512;
/// Maior tamanho guardado para uma linha de comando ou bloco de script.
const MAX_COMMAND_CHARS: usize = 4096;

/// Valor JSON escalar como texto limpo: sem espaços nas pontas, sem NUL, e
/// com os "vazios" do Windows (`-`, `(NULL)`, `` ) tratados como ausentes.
fn scalar_text(value: &Value, max_chars: usize) -> Option<String> {
    let raw = match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        // Alguns campos chegam como `{"#text": "..."}`.
        Value::Object(map) => {
            return map
                .get("#text")
                .and_then(|inner| scalar_text(inner, max_chars));
        }
        _ => return None,
    };
    let cleaned: String = raw.chars().filter(|ch| *ch != '\0').collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() || trimmed == "-" || trimmed.eq_ignore_ascii_case("(NULL)") {
        return None;
    }
    Some(trimmed.chars().take(max_chars).collect())
}

/// `Event` do registro (a raiz que o parser produz).
pub fn event_root(data: &Value) -> &Value {
    data.get("Event").unwrap_or(data)
}

/// `Event/System/EventID` (número ou `{"#text": n}`).
pub fn event_id(data: &Value) -> Option<u32> {
    let value = event_root(data).get("System")?.get("EventID")?;
    let number = match value {
        Value::Number(number) => number.as_u64()?,
        Value::String(text) => text.trim().parse().ok()?,
        Value::Object(map) => match map.get("#text")? {
            Value::Number(number) => number.as_u64()?,
            Value::String(text) => text.trim().parse().ok()?,
            _ => return None,
        },
        _ => return None,
    };
    u32::try_from(number).ok()
}

fn system_text(data: &Value, key: &str) -> Option<String> {
    scalar_text(event_root(data).get("System")?.get(key)?, MAX_FIELD_CHARS)
}

pub fn provider(data: &Value) -> Option<String> {
    let provider = event_root(data).get("System")?.get("Provider")?;
    scalar_text(provider.get("#attributes")?.get("Name")?, MAX_FIELD_CHARS)
}

pub fn channel(data: &Value) -> Option<String> {
    system_text(data, "Channel")
}

pub fn computer(data: &Value) -> Option<String> {
    system_text(data, "Computer")
}

/// Campo de `EventData` (ou, na falta, de `UserData`, onde alguns provedores
/// como o `Microsoft-Windows-Eventlog` colocam os dados).
pub fn data_text(data: &Value, key: &str) -> Option<String> {
    data_text_limited(data, key, MAX_FIELD_CHARS)
}

pub fn data_command(data: &Value, key: &str) -> Option<String> {
    data_text_limited(data, key, MAX_COMMAND_CHARS)
}

fn data_text_limited(data: &Value, key: &str, max_chars: usize) -> Option<String> {
    let root = event_root(data);
    if let Some(value) = root.get("EventData").and_then(|section| section.get(key)) {
        return scalar_text(value, max_chars);
    }
    // `UserData/<Elemento>/<campo>`: um nível de elemento nomeado.
    let user_data = root.get("UserData")?.as_object()?;
    for child in user_data.values() {
        if let Some(value) = child.get(key) {
            return scalar_text(value, max_chars);
        }
    }
    user_data
        .get(key)
        .and_then(|value| scalar_text(value, max_chars))
}

/// Primeiro campo presente entre `keys`.
fn first_of(data: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| data_text(data, key))
}

/// Resultado da auditoria pelo bit de palavra-chave do `Security`:
/// `0x0020…` = sucesso, `0x0010…` = falha. Só vale para o log de auditoria.
fn outcome_from_keywords(data: &Value) -> Option<String> {
    let keywords = system_text(data, "Keywords")?;
    let hex = keywords.trim_start_matches("0x").trim_start_matches("0X");
    let bits = u64::from_str_radix(hex, 16).ok()?;
    if bits & 0x0020_0000_0000_0000 != 0 {
        Some("success".into())
    } else if bits & 0x0010_0000_0000_0000 != 0 {
        Some("failure".into())
    } else {
        None
    }
}

/// Extrai os campos estruturais de um registro.
pub fn event_fields(data: &Value) -> EventFields {
    let id = event_id(data);
    let provider = provider(data);
    let mut fields = EventFields {
        event_id: id.map(|value| value.to_string()),
        provider: provider.clone(),
        channel: channel(data),
        host: computer(data),
        source_type: Some("EVTX".into()),
        ..EventFields::default()
    };
    let is_security_audit = provider
        .as_deref()
        .is_some_and(|name| name.eq_ignore_ascii_case("Microsoft-Windows-Security-Auditing"));
    if is_security_audit {
        fields.outcome = outcome_from_keywords(data);
    }

    let sysmon = provider
        .as_deref()
        .is_some_and(|name| name.eq_ignore_ascii_case("Microsoft-Windows-Sysmon"));
    let powershell = provider
        .as_deref()
        .is_some_and(|name| name.eq_ignore_ascii_case("Microsoft-Windows-PowerShell"));

    if sysmon {
        match id {
            Some(1) => {
                fields.user = data_text(data, "User");
                fields.process = data_text(data, "Image");
                fields.command_line = data_command(data, "CommandLine");
                fields.action = Some("process_created".into());
            }
            Some(3) => {
                fields.user = data_text(data, "User");
                fields.process = data_text(data, "Image");
                fields.src_ip = data_text(data, "SourceIp");
                fields.dst_ip = data_text(data, "DestinationIp");
                fields.action = Some("network_connection".into());
            }
            Some(10) => {
                fields.process = data_text(data, "SourceImage");
                fields.action = Some("process_access".into());
            }
            _ => {}
        }
        return fields;
    }
    if powershell {
        if matches!(id, Some(4103 | 4104)) {
            fields.command_line =
                data_command(data, "ScriptBlockText").or_else(|| data_command(data, "Payload"));
            fields.action = Some("powershell_script_block".into());
            fields.user = data_text(data, "UserId");
        }
        return fields;
    }

    match id {
        // Logon.
        Some(4624) => {
            fields.user = data_text(data, "TargetUserName");
            fields.src_ip = data_text(data, "IpAddress");
            fields.process = data_text(data, "ProcessName");
            fields.action = Some("logon".into());
            fields.outcome = Some("success".into());
        }
        Some(4625) => {
            fields.user = data_text(data, "TargetUserName");
            fields.src_ip = data_text(data, "IpAddress");
            fields.process = data_text(data, "ProcessName");
            fields.action = Some("logon_failed".into());
            fields.outcome = Some("failure".into());
        }
        Some(4634 | 4647) => {
            fields.user = data_text(data, "TargetUserName");
            fields.action = Some("logoff".into());
        }
        Some(4648) => {
            fields.user = data_text(data, "TargetUserName");
            fields.src_ip = data_text(data, "IpAddress");
            fields.process = data_text(data, "ProcessName");
            fields.action = Some("logon_explicit_credentials".into());
        }
        Some(4672) => {
            fields.user = data_text(data, "SubjectUserName");
            fields.action = Some("special_privileges_assigned".into());
        }
        // Processos.
        Some(4688) => {
            fields.user = data_text(data, "SubjectUserName");
            fields.process = data_text(data, "NewProcessName");
            fields.command_line = data_command(data, "CommandLine");
            fields.action = Some("process_created".into());
        }
        // Serviços e tarefas.
        Some(4697) => {
            fields.user = data_text(data, "SubjectUserName");
            fields.process = data_text(data, "ServiceFileName");
            fields.action = Some("service_installed".into());
        }
        Some(7045) => {
            fields.user = data_text(data, "AccountName");
            fields.process = data_text(data, "ImagePath");
            fields.action = Some("service_installed".into());
        }
        Some(4698) => {
            fields.user = data_text(data, "SubjectUserName");
            fields.action = Some("scheduled_task_created".into());
        }
        // Contas e grupos.
        Some(4720) => {
            fields.user = data_text(data, "TargetUserName");
            fields.action = Some("account_created".into());
        }
        Some(4722 | 4724 | 4725 | 4726 | 4738 | 4740 | 4781) => {
            fields.user = data_text(data, "TargetUserName");
            fields.action = Some(
                match id {
                    Some(4722) => "account_enabled",
                    Some(4724) => "password_reset",
                    Some(4725) => "account_disabled",
                    Some(4726) => "account_deleted",
                    Some(4738) => "account_changed",
                    Some(4740) => "account_locked_out",
                    _ => "account_renamed",
                }
                .into(),
            );
        }
        Some(4728 | 4732 | 4756) => {
            fields.user = first_of(data, &["MemberName", "MemberSid"]);
            fields.action = Some("group_member_added".into());
        }
        // Compartilhamentos e rede.
        Some(5140 | 5145) => {
            fields.user = data_text(data, "SubjectUserName");
            fields.src_ip = data_text(data, "IpAddress");
            fields.process = data_text(data, "ShareName");
            fields.action = Some("share_access".into());
        }
        Some(5156 | 5157) => {
            fields.process = data_text(data, "Application");
            fields.src_ip = data_text(data, "SourceAddress");
            fields.dst_ip = data_text(data, "DestAddress");
            fields.action = Some("network_connection".into());
            fields.outcome = Some(
                if id == Some(5156) {
                    "success"
                } else {
                    "failure"
                }
                .into(),
            );
        }
        // Log apagado.
        Some(1102 | 104) => {
            fields.user = data_text(data, "SubjectUserName");
            fields.action = Some("log_cleared".into());
        }
        Some(4719) => {
            fields.user = data_text(data, "SubjectUserName");
            fields.action = Some("audit_policy_changed".into());
        }
        _ => {
            // Eventos sem mapeamento: só a conta, se o registro a traz.
            fields.user = first_of(
                data,
                &[
                    "TargetUserName",
                    "SubjectUserName",
                    "UserName",
                    "AccountName",
                ],
            );
            fields.src_ip = data_text(data, "IpAddress");
        }
    }
    fields
}

/// `YYYY-MM-DDTHH:MM:SS[.ffffff]Z` (ou com fuso `±HH:MM`) em microssegundos
/// desde a época Unix. Sem dependência de calendário: o engine só precisa
/// **ordenar e medir janelas**, não formatar datas.
pub fn epoch_micros(timestamp: &str) -> Option<i64> {
    let text = timestamp.trim();
    let (date, rest) = text.split_once(['T', ' '])?;
    let mut date_parts = date.split('-');
    let year: i64 = date_parts.next()?.parse().ok()?;
    let month: i64 = date_parts.next()?.parse().ok()?;
    let day: i64 = date_parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // Separa a hora do fuso.
    let (clock, offset_seconds) = if let Some(clock) = rest.strip_suffix(['Z', 'z']) {
        (clock, 0_i64)
    } else if let Some(index) = rest.rfind(['+', '-']).filter(|index| *index > 2) {
        let (clock, zone) = rest.split_at(index);
        let sign = if zone.starts_with('-') { -1 } else { 1 };
        let zone = &zone[1..];
        let (hours, minutes) = zone.split_once(':').unwrap_or((zone, "0"));
        let seconds = hours.parse::<i64>().ok()? * 3600 + minutes.parse::<i64>().ok()? * 60;
        (clock, sign * seconds)
    } else {
        (rest, 0)
    };
    let (whole, fraction) = clock.split_once('.').unwrap_or((clock, ""));
    let mut clock_parts = whole.split(':');
    let hour: i64 = clock_parts.next()?.parse().ok()?;
    let minute: i64 = clock_parts.next()?.parse().ok()?;
    let second: i64 = clock_parts.next().unwrap_or("0").parse().ok()?;
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let mut micros: i64 = 0;
    if !fraction.is_empty() {
        let digits: String = fraction.chars().take(6).collect();
        if !digits.chars().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
        micros = format!("{digits:0<6}").parse().ok()?;
    }
    // Dias desde 1970-01-01 (algoritmo de calendário civil, proléptico gregoriano).
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    let seconds = days * 86_400 + hour * 3600 + minute * 60 + second - offset_seconds;
    seconds.checked_mul(1_000_000)?.checked_add(micros)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn logon(id: u64, extra: Value) -> Value {
        let mut event_data = json!({
            "TargetUserName": "fsir", "IpAddress": "10.0.0.5", "LogonType": 3,
            "ProcessName": "C:\\Windows\\System32\\winlogon.exe"
        });
        if let (Some(base), Some(more)) = (event_data.as_object_mut(), extra.as_object()) {
            for (key, value) in more {
                base.insert(key.clone(), value.clone());
            }
        }
        json!({"Event": {
            "System": {
                "Provider": {"#attributes": {"Name": "Microsoft-Windows-Security-Auditing"}},
                "EventID": id, "Channel": "Security", "Computer": "DC01",
                "Keywords": "0x8020000000000000"
            },
            "EventData": event_data
        }})
    }

    #[test]
    fn a_successful_logon_gets_structured_fields() {
        let fields = event_fields(&logon(4624, json!({})));
        assert_eq!(fields.event_id.as_deref(), Some("4624"));
        assert_eq!(
            fields.provider.as_deref(),
            Some("Microsoft-Windows-Security-Auditing")
        );
        assert_eq!(fields.channel.as_deref(), Some("Security"));
        assert_eq!(fields.host.as_deref(), Some("DC01"));
        assert_eq!(fields.user.as_deref(), Some("fsir"));
        assert_eq!(fields.src_ip.as_deref(), Some("10.0.0.5"));
        assert_eq!(fields.action.as_deref(), Some("logon"));
        assert_eq!(fields.outcome.as_deref(), Some("success"));
        assert_eq!(fields.source_type.as_deref(), Some("EVTX"));
        assert!(fields.dst_ip.is_none() && fields.command_line.is_none());
    }

    #[test]
    fn windows_placeholders_for_empty_values_are_absent_not_dashes() {
        let fields = event_fields(&logon(4624, json!({"IpAddress": "-", "ProcessName": ""})));
        assert!(fields.src_ip.is_none());
        assert!(fields.process.is_none());
        let fields = event_fields(&logon(4624, json!({"IpAddress": "(NULL)"})));
        assert!(fields.src_ip.is_none());
    }

    #[test]
    fn a_failed_logon_is_a_failure() {
        let fields = event_fields(&logon(4625, json!({"IpAddress": "185.220.101.5"})));
        assert_eq!(fields.action.as_deref(), Some("logon_failed"));
        assert_eq!(fields.outcome.as_deref(), Some("failure"));
        assert_eq!(fields.src_ip.as_deref(), Some("185.220.101.5"));
    }

    #[test]
    fn the_event_id_accepts_a_number_a_string_and_a_text_object() {
        for value in [
            json!(4624),
            json!("4624"),
            json!({"#text": 4624, "#attributes": {"Qualifiers": 0}}),
        ] {
            let data = json!({"Event": {"System": {"EventID": value}}});
            assert_eq!(event_id(&data), Some(4624));
        }
        assert_eq!(event_id(&json!({"Event": {"System": {}}})), None);
        assert_eq!(
            event_id(&json!({"Event": {"System": {"EventID": -1}}})),
            None
        );
    }

    #[test]
    fn process_creation_carries_the_command_line() {
        let data = json!({"Event": {
            "System": {"Provider": {"#attributes": {"Name": "Microsoft-Windows-Security-Auditing"}}, "EventID": 4688, "Computer": "WS01"},
            "EventData": {"SubjectUserName": "bob", "NewProcessName": "C:\\Windows\\System32\\cmd.exe", "CommandLine": "cmd /c whoami"}
        }});
        let fields = event_fields(&data);
        assert_eq!(
            fields.process.as_deref(),
            Some("C:\\Windows\\System32\\cmd.exe")
        );
        assert_eq!(fields.command_line.as_deref(), Some("cmd /c whoami"));
        assert_eq!(fields.action.as_deref(), Some("process_created"));
    }

    #[test]
    fn user_data_providers_are_read_too() {
        // Microsoft-Windows-Eventlog 104: dados em UserData/LogFileCleared.
        let data = json!({"Event": {
            "System": {"Provider": {"#attributes": {"Name": "Microsoft-Windows-Eventlog"}}, "EventID": 104},
            "UserData": {"LogFileCleared": {"SubjectUserName": "mallory", "Channel": "System"}}
        }});
        let fields = event_fields(&data);
        assert_eq!(fields.user.as_deref(), Some("mallory"));
        assert_eq!(fields.action.as_deref(), Some("log_cleared"));
    }

    #[test]
    fn a_record_without_event_data_still_yields_the_system_fields() {
        let data = json!({"Event": {
            "System": {"Provider": {"#attributes": {"Name": "Microsoft-Windows-Eventlog"}}, "EventID": 1100, "Channel": "Security", "Computer": "X"},
            "EventData": null
        }});
        let fields = event_fields(&data);
        assert_eq!(fields.event_id.as_deref(), Some("1100"));
        assert_eq!(fields.host.as_deref(), Some("X"));
        assert!(fields.user.is_none() && fields.action.is_none());
    }

    #[test]
    fn hostile_values_are_bounded_and_nul_free() {
        let long = "A".repeat(100_000);
        let data = logon(4624, json!({"TargetUserName": format!("a\0b{long}")}));
        let fields = event_fields(&data);
        let user = fields.user.expect("user");
        assert!(user.chars().count() <= MAX_FIELD_CHARS);
        assert!(!user.contains('\0'));
        // Nada de campo estrutural em registro que não é objeto.
        let fields = event_fields(&json!("não é um evento"));
        assert!(fields.event_id.is_none() && fields.user.is_none());
    }

    #[test]
    fn epoch_micros_matches_known_instants() {
        assert_eq!(epoch_micros("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(epoch_micros("1970-01-01T00:00:01.5Z"), Some(1_500_000));
        assert_eq!(
            epoch_micros("2000-03-01T00:00:00Z"),
            Some(951_868_800_000_000)
        );
        assert_eq!(
            epoch_micros("2016-07-08T18:12:51.681640Z"),
            Some(1_468_001_571_681_640)
        );
        // Fuso: 03:00 em -03:00 é 06:00 UTC.
        assert_eq!(
            epoch_micros("2026-01-01T03:00:00-03:00"),
            epoch_micros("2026-01-01T06:00:00Z")
        );
        // Ano bissexto e virada de século.
        assert_eq!(
            epoch_micros("2024-03-01T00:00:00Z").unwrap()
                - epoch_micros("2024-02-28T00:00:00Z").unwrap(),
            2 * 86_400 * 1_000_000
        );
        assert_eq!(
            epoch_micros("1900-03-01T00:00:00Z").unwrap()
                - epoch_micros("1900-02-28T00:00:00Z").unwrap(),
            86_400 * 1_000_000
        );
    }

    #[test]
    fn epoch_micros_rejects_garbage() {
        for bad in [
            "",
            "ontem",
            "2026-13-01T00:00:00Z",
            "2026-01-32T00:00:00Z",
            "2026-01-01T25:00:00Z",
            "2026-01-01",
        ] {
            assert_eq!(epoch_micros(bad), None, "{bad:?}");
        }
    }
}
