//! F1-32 (RF-055, RB-006) — máscara de dados pessoais nos entregáveis.
//!
//! O que é mascarado, **por padrão de texto**: endereços IPv4 e IPv6,
//! e-mails e contas no formato `DOMÍNIO\usuário`. Além disso, o campo
//! "fornecida por" da evidência e o responsável de cada evento de custódia
//! (exceto o ator padrão do aplicativo) são mascarados por inteiro.
//!
//! O que **não** é detectado: nomes de pessoas em texto livre, números de
//! documento, endereços. A máscara é por padrão e é aviso, não garantia de
//! anonimização: por isso o documento diz o que foi mascarado e o operador
//! escolhe explicitamente exportar sem ela.
//!
//! ## Marcadores
//!
//! Cada valor distinto vira um marcador estável dentro de uma geração:
//! `{IP-1}`, `{EMAIL-2}`, `{CONTA-1}`, `{PESSOA-3}`. O mesmo valor recebe o
//! mesmo marcador em todos os documentos (dá para correlacionar), mas o
//! mapeamento **não é gravado em lugar nenhum**. A numeração segue a ordem
//! alfabética dos valores, não a ordem em que aparecem: a saída não muda se
//! os dados chegarem em outra ordem (renderização determinística).
//!
//! Dois passes: o primeiro coleta todos os valores; o segundo substitui.

use super::{CustodyEntry, DeliverableData};
use crate::service::{AnalyticalResultResponse, EvidenceResponse};
use serde::Serialize;
use std::collections::BTreeSet;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ops::Range;
use std::str::FromStr;

/// Ator que o aplicativo grava quando é ele que age; não é uma pessoa.
const APP_ACTOR: &str = "local-analyst";

/// Contagens do que foi mascarado, sem os valores.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RedactionSummary {
    pub ips: usize,
    pub emails: usize,
    pub accounts: usize,
    pub persons: usize,
}

#[derive(Clone, Copy)]
enum Kind {
    Email,
    Account,
    Ipv6,
    Ipv4,
}

// ---------------------------------------------------------------------
// Detecção
// ---------------------------------------------------------------------

fn is_email_local(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-')
}

fn is_email_domain(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '-')
}

fn find_emails(text: &str) -> Vec<Range<usize>> {
    let mut found: Vec<Range<usize>> = Vec::new();
    for (at, _) in text.char_indices().filter(|(_, c)| *c == '@') {
        let mut start = at;
        for (index, c) in text[..at].char_indices().rev() {
            if is_email_local(c) {
                start = index;
            } else {
                break;
            }
        }
        let mut end = at + 1;
        for (index, c) in text[at + 1..].char_indices() {
            if is_email_domain(c) {
                end = at + 1 + index + c.len_utf8();
            } else {
                break;
            }
        }
        let domain = text[at + 1..end].trim_end_matches(['.', '-']);
        end = at + 1 + domain.len();
        let overlaps = found.last().is_some_and(|last| start < last.end);
        if start < at && domain.contains('.') && !domain.starts_with('.') && !overlaps {
            found.push(start..end);
        }
    }
    found
}

fn is_domain_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')
}

fn is_user_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$' | '-')
}

/// `DOMÍNIO\usuário`. O domínio tem de parecer um domínio (NetBIOS em
/// maiúsculas ou nome com ponto) e não pode vir colado a um caminho, para
/// não confundir `C:\Windows\System32` com uma conta.
fn find_accounts(text: &str) -> Vec<Range<usize>> {
    let mut found: Vec<Range<usize>> = Vec::new();
    for (slash, _) in text.char_indices().filter(|(_, c)| *c == '\\') {
        let mut start = slash;
        for (index, c) in text[..slash].char_indices().rev() {
            if is_domain_char(c) {
                start = index;
            } else {
                break;
            }
        }
        let mut end = slash + 1;
        for (index, c) in text[slash + 1..].char_indices() {
            if is_user_char(c) {
                end = slash + 1 + index + c.len_utf8();
            } else {
                break;
            }
        }
        let domain = &text[start..slash];
        let user = text[slash + 1..end].trim_end_matches('.');
        end = slash + 1 + user.len();
        let boundary_ok = text[..start].chars().next_back().is_none_or(|c| {
            c.is_whitespace()
                || matches!(
                    c,
                    '(' | '"' | '\'' | ',' | ';' | '=' | '[' | '<' | '{' | '|'
                )
        });
        let looks_like_domain = domain.len() >= 2
            && (domain.contains('.') || !domain.chars().any(|c| c.is_ascii_lowercase()));
        let overlaps = found.last().is_some_and(|last| start < last.end);
        if boundary_ok && looks_like_domain && !user.is_empty() && !overlaps {
            found.push(start..end);
        }
    }
    found
}

/// Percorre as sequências de caracteres de `allowed` e devolve as que
/// `accept` aceita (já sem pontos nas pontas).
fn scan_runs(
    text: &str,
    allowed: impl Fn(char) -> bool,
    accept: impl Fn(&str, Option<char>, Option<char>) -> bool,
) -> Vec<Range<usize>> {
    let mut found = Vec::new();
    let mut run_start: Option<usize> = None;
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    for position in 0..=chars.len() {
        let in_run = chars.get(position).is_some_and(|(_, c)| allowed(*c));
        match (in_run, run_start) {
            (true, None) => run_start = Some(chars[position].0),
            (false, Some(start)) => {
                let end = chars.get(position).map_or(text.len(), |(index, _)| *index);
                let token = text[start..end].trim_matches('.');
                let trimmed_start = start
                    + (text[start..end].len() - text[start..end].trim_start_matches('.').len());
                let trimmed_end = trimmed_start + token.len();
                let before = text[..trimmed_start].chars().next_back();
                let after = text[trimmed_end..].chars().next();
                if !token.is_empty() && accept(token, before, after) {
                    found.push(trimmed_start..trimmed_end);
                }
                run_start = None;
            }
            _ => {}
        }
    }
    found
}

fn find_ipv6(text: &str) -> Vec<Range<usize>> {
    scan_runs(
        text,
        |c| c.is_ascii_hexdigit() || matches!(c, ':' | '.'),
        |token, _, _| token.contains(':') && Ipv6Addr::from_str(token).is_ok(),
    )
}

fn find_ipv4(text: &str) -> Vec<Range<usize>> {
    scan_runs(
        text,
        |c| c.is_ascii_digit() || c == '.',
        |token, before, after| {
            Ipv4Addr::from_str(token).is_ok()
                && !before.is_some_and(|c| c.is_ascii_alphanumeric())
                && !after.is_some_and(|c| c.is_ascii_alphanumeric())
        },
    )
}

fn ranges_of(kind: Kind, text: &str) -> Vec<Range<usize>> {
    match kind {
        Kind::Email => find_emails(text),
        Kind::Account => find_accounts(text),
        Kind::Ipv6 => find_ipv6(text),
        Kind::Ipv4 => find_ipv4(text),
    }
}

/// Aplica as quatro detecções em sequência, trocando cada achado pelo que
/// `replace` devolver. Os marcadores não casam com nenhum padrão, então a
/// coleta e a substituição enxergam exatamente os mesmos achados.
fn transform(text: &str, replace: &mut dyn FnMut(Kind, &str) -> String) -> String {
    let mut current = text.to_string();
    for kind in [Kind::Email, Kind::Account, Kind::Ipv6, Kind::Ipv4] {
        let ranges = ranges_of(kind, &current);
        if ranges.is_empty() {
            continue;
        }
        let mut out = String::with_capacity(current.len());
        let mut last = 0;
        for range in ranges {
            out.push_str(&current[last..range.start]);
            out.push_str(&replace(kind, &current[range.clone()]));
            last = range.end;
        }
        out.push_str(&current[last..]);
        current = out;
    }
    current
}

// ---------------------------------------------------------------------
// Substituição
// ---------------------------------------------------------------------

#[derive(Default)]
struct Redactor {
    emails: BTreeSet<String>,
    accounts: BTreeSet<String>,
    ips: BTreeSet<String>,
    persons: BTreeSet<String>,
}

/// Chave de normalização: o mesmo valor com outra caixa ou grafia de IP
/// recebe o mesmo marcador.
fn key_of(kind: Kind, matched: &str) -> String {
    match kind {
        Kind::Email | Kind::Account => matched.to_lowercase(),
        Kind::Ipv4 => {
            Ipv4Addr::from_str(matched).map_or_else(|_| matched.to_string(), |ip| ip.to_string())
        }
        Kind::Ipv6 => {
            Ipv6Addr::from_str(matched).map_or_else(|_| matched.to_string(), |ip| ip.to_string())
        }
    }
}

fn token(set: &BTreeSet<String>, key: &str, label: &str) -> String {
    let index = set.iter().position(|value| value == key).unwrap_or(0) + 1;
    format!("{{{label}-{index}}}")
}

impl Redactor {
    fn collect_text(&mut self, text: &str) {
        transform(text, &mut |kind, matched| {
            let key = key_of(kind, matched);
            match kind {
                Kind::Email => self.emails.insert(key),
                Kind::Account => self.accounts.insert(key),
                Kind::Ipv4 | Kind::Ipv6 => self.ips.insert(key),
            };
            "{}".to_string()
        });
    }

    fn collect_person(&mut self, name: &str) {
        let name = name.trim();
        if !name.is_empty() {
            self.persons.insert(name.to_string());
        }
    }

    fn replace_text(&self, text: &str) -> String {
        transform(text, &mut |kind, matched| {
            let key = key_of(kind, matched);
            match kind {
                Kind::Email => token(&self.emails, &key, "EMAIL"),
                Kind::Account => token(&self.accounts, &key, "CONTA"),
                Kind::Ipv4 | Kind::Ipv6 => token(&self.ips, &key, "IP"),
            }
        })
    }

    fn replace_person(&self, name: &str) -> String {
        let name = name.trim();
        if name.is_empty() {
            String::new()
        } else {
            token(&self.persons, name, "PESSOA")
        }
    }

    fn summary(&self) -> RedactionSummary {
        RedactionSummary {
            ips: self.ips.len(),
            emails: self.emails.len(),
            accounts: self.accounts.len(),
            persons: self.persons.len(),
        }
    }
}

// ---------------------------------------------------------------------
// Travessia dos dados
// ---------------------------------------------------------------------

/// Visitante: recebe cada campo de texto e diz se é um nome de pessoa
/// (mascarado por inteiro) ou texto livre (mascarado por padrão).
trait Visitor {
    fn text(&mut self, value: &mut String);
    fn person(&mut self, value: &mut String);
}

fn visit_json(visitor: &mut dyn Visitor, value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(text) => visitor.text(text),
        serde_json::Value::Array(items) => {
            for item in items {
                visit_json(visitor, item);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, item) in map.iter_mut() {
                visit_json(visitor, item);
            }
        }
        _ => {}
    }
}

fn visit_results(visitor: &mut dyn Visitor, results: &mut [AnalyticalResultResponse]) {
    for result in results {
        visit_json(visitor, &mut result.payload);
    }
}

fn visit_optional(visitor: &mut dyn Visitor, value: &mut Option<String>, person: bool) {
    if let Some(text) = value {
        if person {
            visitor.person(text);
        } else {
            visitor.text(text);
        }
    }
}

fn visit_evidence(visitor: &mut dyn Visitor, evidence: &mut EvidenceResponse) {
    visitor.text(&mut evidence.original_filename);
    visitor.text(&mut evidence.source);
    visit_optional(visitor, &mut evidence.provided_by, true);
    visitor.text(&mut evidence.collection_method);
    visit_optional(visitor, &mut evidence.collection_tool, false);
    visit_optional(visitor, &mut evidence.collection_tool_version, false);
    visit_optional(visitor, &mut evidence.authorization_reference, false);
}

fn visit_custody(visitor: &mut dyn Visitor, entry: &mut CustodyEntry) {
    if entry.actor != APP_ACTOR {
        visitor.person(&mut entry.actor);
    }
    visitor.text(&mut entry.reason);
}

fn visit(visitor: &mut dyn Visitor, data: &mut DeliverableData) {
    visitor.text(&mut data.incident.name);
    for section in &mut data.evidence {
        visit_evidence(visitor, &mut section.evidence);
        for entry in &mut section.custody {
            visit_custody(visitor, entry);
        }
    }
    for evidence in &mut data.other_evidence {
        visit_evidence(visitor, evidence);
    }
    visit_results(visitor, &mut data.attack_mappings);
    visit_results(visitor, &mut data.critical_path.stages);
    if let Some(impact) = &mut data.impact {
        visit_json(visitor, &mut impact.payload);
    }
    visit_results(visitor, &mut data.iocs);
    visit_results(visitor, &mut data.recommendations);
    visit_results(visitor, &mut data.findings.findings);
    visit_results(visitor, &mut data.findings.root_causes);
    visit_results(visitor, &mut data.findings.attack_mappings);
    visit_results(visitor, &mut data.plan.items);
    visit_results(visitor, &mut data.plan.findings);
    visit_results(visitor, &mut data.gaps);
    visit_results(visitor, &mut data.source_gaps);
    for profile in [
        &mut data.nist.current_profile,
        &mut data.nist.target_profile,
    ]
    .into_iter()
    .flatten()
    {
        visitor.text(&mut profile.version_label);
        visit_optional(visitor, &mut profile.tier_justification, false);
    }
}

struct Collect<'a>(&'a mut Redactor);

impl Visitor for Collect<'_> {
    fn text(&mut self, value: &mut String) {
        self.0.collect_text(value);
    }

    fn person(&mut self, value: &mut String) {
        self.0.collect_person(value);
    }
}

struct Replace<'a>(&'a Redactor);

impl Visitor for Replace<'_> {
    fn text(&mut self, value: &mut String) {
        *value = self.0.replace_text(value);
    }

    fn person(&mut self, value: &mut String) {
        *value = self.0.replace_person(value);
    }
}

/// Cópia dos dados com os dados pessoais mascarados, e o resumo do que foi
/// mascarado. Não altera `data`.
pub fn redact(data: &DeliverableData) -> (DeliverableData, RedactionSummary) {
    let mut redactor = Redactor::default();
    let mut scratch = data.clone();
    visit(&mut Collect(&mut redactor), &mut scratch);

    let mut masked = data.clone();
    visit(&mut Replace(&redactor), &mut masked);
    let summary = redactor.summary();
    masked.redaction = Some(summary.clone());
    (masked, summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replaced(text: &str) -> String {
        let mut redactor = Redactor::default();
        redactor.collect_text(text);
        redactor.replace_text(text)
    }

    #[test]
    fn emails_ips_and_domain_accounts_are_found_in_free_text() {
        let text = "Logon de joao.silva@empresa.com.br em 203.0.113.7:3389 pela conta CORP\\joao e pela corp.local\\maria; v6 2001:db8::7 e ::ffff:198.51.100.9.";
        let masked = replaced(text);
        for original in [
            "joao.silva@empresa.com.br",
            "203.0.113.7",
            "CORP\\joao",
            "corp.local\\maria",
            "2001:db8::7",
            "198.51.100.9",
        ] {
            assert!(!masked.contains(original), "{original} em {masked}");
        }
        assert!(masked.contains("{EMAIL-1}"));
        // 198.51.100.9 (mapeado em IPv6) < 2001:db8::7 < 203.0.113.7, em ordem alfabética.
        assert!(masked.contains("{IP-2}:3389"), "a porta fica: {masked}");
        assert!(masked.contains("{CONTA-1}") && masked.contains("{CONTA-2}"));
    }

    #[test]
    fn what_is_not_personal_data_is_left_alone() {
        for text in [
            "2026-09-30T22:01:00Z",
            "22:01:00",
            "versão 10.0.22621 e 10.0.22621.1",
            "C:\\Windows\\System32\\config\\SAM",
            "\\\\servidor\\compartilhamento",
            "Windows\\System32",
            "hash 5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a",
            "id 123e4567-e89b-12d3-a456-426614174000",
            "a@b",
            "e-mail@",
            "T1133 e CWE-308",
            "x1.2.3.4y",
        ] {
            assert_eq!(replaced(text), text, "alterou: {text}");
        }
    }

    #[test]
    fn the_same_value_gets_the_same_marker_with_numbers_by_sorted_value() {
        let mut redactor = Redactor::default();
        let text = "b@x.com depois a@x.com e de novo B@X.COM; 10.0.0.2 e 10.0.0.1 e 010.0.0.1x";
        redactor.collect_text(text);
        let masked = redactor.replace_text(text);
        // a@x.com < b@x.com: EMAIL-1 é "a", EMAIL-2 é "b" (e B@X.COM é o mesmo "b").
        assert!(
            masked.starts_with("{EMAIL-2} depois {EMAIL-1} e de novo {EMAIL-2};"),
            "{masked}"
        );
        // 10.0.0.1 < 10.0.0.2.
        assert!(masked.contains("{IP-2} e {IP-1}"), "{masked}");
        assert_eq!(redactor.summary().emails, 2);
        assert_eq!(redactor.summary().ips, 2);
    }
}
