//! Filtro estruturado da timeline (F1-18/F1-26).
//!
//! O mesmo campo de texto do filtro aceita, além de palavras livres, termos
//! `campo:valor` que consultam as colunas estruturais do evento:
//!
//! | termo                | casa com                                           |
//! |----------------------|----------------------------------------------------|
//! | `ref:record:5,record:9` | `raw_reference` exato (vários valores = OU)    |
//! | `event:4624,4625`    | `event_id` exato (vários valores = OU)             |
//! | `user:fsir`          | conta contém o texto                               |
//! | `host:DC01`          | máquina contém o texto                             |
//! | `ip:10.0.0.5`        | IP de origem **ou** de destino, exato              |
//! | `src:` / `dst:`      | só origem / só destino, exato                      |
//! | `proc:powershell`    | processo contém o texto                            |
//! | `action:logon`       | ação normalizada contém o texto                    |
//! | `outcome:failure`    | resultado exato (`success` ou `failure`)           |
//! | `provider:` `channel:` | provedor / canal contém o texto                  |
//! | `from:` `to:`        | intervalo em UTC (`2016-07-08` ou RFC 3339)        |
//!
//! Termos diferentes se combinam por **E**; valores separados por vírgula num
//! mesmo termo, por **OU**. O que não for `campo:valor` conhecido é texto
//! livre (título, descrição, provedor, canal), como sempre foi. Aspas agrupam
//! (`user:"maria silva"`). Um `from:`/`to:` inválido é **erro**, não texto: o
//! analista não pode achar que filtrou por data quando não filtrou.

use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TimelineFilter {
    pub text: Option<String>,
    /// Referências brutas dos eventos de origem (`record:N`), por igualdade exata.
    pub references: Vec<String>,
    pub event_ids: Vec<String>,
    pub users: Vec<String>,
    pub hosts: Vec<String>,
    /// Origem **ou** destino.
    pub ips: Vec<String>,
    pub src_ips: Vec<String>,
    pub dst_ips: Vec<String>,
    pub processes: Vec<String>,
    pub actions: Vec<String>,
    pub outcomes: Vec<String>,
    pub providers: Vec<String>,
    pub channels: Vec<String>,
    pub from_micros: Option<i64>,
    /// Exclusivo.
    pub to_micros: Option<i64>,
}

impl TimelineFilter {
    /// `true` quando nenhum termo restringe a consulta.
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

/// Divide em termos respeitando aspas duplas.
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut started = false;
    for ch in input.chars() {
        match ch {
            '"' => {
                quoted = !quoted;
                started = true;
            }
            ch if ch.is_whitespace() && !quoted => {
                if started {
                    tokens.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            ch => {
                current.push(ch);
                started = true;
            }
        }
    }
    if started {
        tokens.push(current);
    }
    tokens
}

fn values(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_instant(raw: &str, end_of_range: bool) -> Result<i64, String> {
    let text = raw.trim();
    if let Ok(date) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        let day = if end_of_range {
            date.succ_opt().unwrap_or(date)
        } else {
            date
        };
        let start = day.and_hms_opt(0, 0, 0).expect("midnight is valid");
        return Ok(start.and_utc().timestamp_micros());
    }
    let instant = DateTime::parse_from_rfc3339(text)
        .map_err(|_| format!("invalid date {text:?}: use YYYY-MM-DD or RFC 3339"))?
        .with_timezone(&Utc)
        .timestamp_micros();
    // `to:` com horário é inclusivo; o limite superior da consulta é exclusivo.
    Ok(if end_of_range { instant + 1 } else { instant })
}

/// Interpreta o texto do filtro. `Err` leva uma mensagem para o analista.
pub fn parse(input: &str) -> Result<TimelineFilter, String> {
    let mut filter = TimelineFilter::default();
    let mut free: Vec<String> = Vec::new();
    for token in tokenize(input) {
        let Some((key, value)) = token.split_once(':') else {
            free.push(token);
            continue;
        };
        let key_lower = key.to_ascii_lowercase();
        let known = matches!(
            key_lower.as_str(),
            "ref"
                | "event"
                | "id"
                | "eventid"
                | "user"
                | "host"
                | "computer"
                | "ip"
                | "src"
                | "dst"
                | "proc"
                | "process"
                | "action"
                | "outcome"
                | "result"
                | "provider"
                | "channel"
                | "from"
                | "to"
        );
        if !known || value.is_empty() {
            // `http://x`, `C:\\x`, `campo:` vazio: texto livre.
            free.push(token);
            continue;
        }
        match key_lower.as_str() {
            "ref" => filter.references.extend(values(value)),
            "event" | "id" | "eventid" => filter.event_ids.extend(values(value)),
            "user" => filter.users.extend(values(value)),
            "host" | "computer" => filter.hosts.extend(values(value)),
            "ip" => filter.ips.extend(values(value)),
            "src" => filter.src_ips.extend(values(value)),
            "dst" => filter.dst_ips.extend(values(value)),
            "proc" | "process" => filter.processes.extend(values(value)),
            "action" => filter.actions.extend(values(value)),
            "outcome" | "result" => filter
                .outcomes
                .extend(values(value).into_iter().map(|v| v.to_ascii_lowercase())),
            "provider" => filter.providers.extend(values(value)),
            "channel" => filter.channels.extend(values(value)),
            "from" => filter.from_micros = Some(parse_instant(value, false)?),
            "to" => filter.to_micros = Some(parse_instant(value, true)?),
            _ => unreachable!("filtered by `known`"),
        }
    }
    if let (Some(from), Some(to)) = (filter.from_micros, filter.to_micros)
        && from >= to
    {
        return Err("the filter range is empty: from must be earlier than to".into());
    }
    if !free.is_empty() {
        filter.text = Some(free.join(" "));
    }
    Ok(filter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_words_stay_free_text_exactly_as_before() {
        let filter = parse("logon falhou").unwrap();
        assert_eq!(filter.text.as_deref(), Some("logon falhou"));
        assert!(filter.event_ids.is_empty() && filter.users.is_empty());
        assert!(parse("   ").unwrap().is_empty());
        assert!(parse("").unwrap().is_empty());
    }

    #[test]
    fn candidate_reference_term_is_not_free_text() {
        let filter = parse("ref:record:5,record:9 event:4624").unwrap();
        assert_eq!(
            filter.text, None,
            "ref:record:5,record:9 não pode ser tratado como texto livre"
        );
        assert_eq!(filter.references, vec!["record:5", "record:9"]);
        assert_eq!(filter.event_ids, vec!["4624"]);
    }

    #[test]
    fn structured_terms_and_free_text_combine() {
        let filter = parse("event:4624,4625 user:fsir ip:10.0.0.5 senha").unwrap();
        assert_eq!(filter.event_ids, vec!["4624", "4625"]);
        assert_eq!(filter.users, vec!["fsir"]);
        assert_eq!(filter.ips, vec!["10.0.0.5"]);
        assert_eq!(filter.text.as_deref(), Some("senha"));
    }

    #[test]
    fn keys_are_case_insensitive_and_aliases_work() {
        let filter = parse("EVENT:1 Computer:DC01 proc:cmd result:FAILURE").unwrap();
        assert_eq!(filter.event_ids, vec!["1"]);
        assert_eq!(filter.hosts, vec!["DC01"]);
        assert_eq!(filter.processes, vec!["cmd"]);
        assert_eq!(
            filter.outcomes,
            vec!["failure"],
            "resultado é normalizado em minúsculas"
        );
    }

    #[test]
    fn quotes_group_values_and_phrases() {
        let filter = parse(r#"user:"maria silva" "falha de logon""#).unwrap();
        assert_eq!(filter.users, vec!["maria silva"]);
        assert_eq!(filter.text.as_deref(), Some("falha de logon"));
    }

    #[test]
    fn unknown_keys_urls_and_windows_paths_are_free_text() {
        let filter = parse(r"http://evil.example C:\Windows\x.exe foo:bar user:").unwrap();
        assert!(filter.users.is_empty());
        assert_eq!(
            filter.text.as_deref(),
            Some(r"http://evil.example C:\Windows\x.exe foo:bar user:")
        );
    }

    #[test]
    fn dates_accept_a_day_or_an_rfc3339_instant_and_to_is_exclusive() {
        let filter = parse("from:2016-07-08 to:2016-07-08").unwrap();
        let (from, to) = (filter.from_micros.unwrap(), filter.to_micros.unwrap());
        assert_eq!(to - from, 86_400 * 1_000_000, "o dia inteiro");
        let precise = parse("from:2016-07-08T10:00:00Z to:2016-07-08T10:00:00Z").unwrap();
        assert_eq!(
            precise.to_micros.unwrap() - precise.from_micros.unwrap(),
            1,
            "instante inclusivo"
        );
        let zoned = parse("from:2016-07-08T10:00:00-03:00").unwrap();
        let utc = parse("from:2016-07-08T13:00:00Z").unwrap();
        assert_eq!(zoned.from_micros, utc.from_micros);
    }

    #[test]
    fn a_bad_date_or_an_empty_range_is_an_error_not_free_text() {
        assert!(parse("from:ontem").unwrap_err().contains("invalid date"));
        assert!(parse("to:2016-13-45").is_err());
        assert!(
            parse("from:2016-07-09 to:2016-07-01")
                .unwrap_err()
                .contains("empty")
        );
    }

    #[test]
    fn an_unterminated_quote_does_not_panic_and_keeps_the_rest() {
        let filter = parse(r#"user:"maria silva"#).unwrap();
        assert_eq!(filter.users, vec!["maria silva"]);
    }
}
