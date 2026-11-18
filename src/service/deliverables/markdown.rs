//! Construtor de Markdown dos entregáveis (F1-07).
//!
//! Todo texto que vem de dados — nome do arquivo de evidência, texto do
//! analista, valores de IOC, campos de log — passa por [`esc`] ou [`code`]
//! antes de entrar no documento. Isso fecha a injeção de Markdown e de HTML
//! (um valor de IOC com `<script>`, um título com `|` ou quebra de linha que
//! desalinha uma tabela, um `[x](javascript:...)`, uma sequência ANSI que
//! mexe no terminal de quem lê o `.md` cru).

use std::collections::BTreeSet;

/// Escapa texto de dados para uso em linha: colapsa espaços, quebras de
/// linha e caracteres de controle (inclusive `ESC`) num único espaço e
/// protege os caracteres que o Markdown interpreta.
pub fn esc(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut pending_space = false;
    // Caracteres invisíveis e de direção de texto viram marcadores visíveis
    // antes de qualquer outra coisa (o marcador não contém nada que o
    // Markdown interprete).
    let raw = crate::text_safety::neutralize(raw);
    for ch in raw.chars() {
        if ch.is_whitespace() || ch.is_control() {
            pending_space = !out.is_empty();
            continue;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        match ch {
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '|' | '#' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Texto de dados como código em linha. Crases são trocadas por apóstrofos
/// (não há como escapá-las dentro de um trecho de código) e `|` é protegido
/// para não quebrar tabelas.
pub fn code(raw: &str) -> String {
    let mut cleaned = String::with_capacity(raw.len());
    let mut pending_space = false;
    let raw = crate::text_safety::neutralize(raw);
    for ch in raw.chars() {
        if ch.is_whitespace() || ch.is_control() {
            pending_space = !cleaned.is_empty();
            continue;
        }
        if pending_space {
            cleaned.push(' ');
            pending_space = false;
        }
        match ch {
            '`' => cleaned.push('\''),
            '|' => cleaned.push_str("\\|"),
            _ => cleaned.push(ch),
        }
    }
    if cleaned.is_empty() {
        "—".to_string()
    } else {
        format!("`{cleaned}`")
    }
}

/// Os 8 primeiros caracteres de um id, para citar a origem sem poluir o texto
/// (o id completo vai para o manifest).
pub fn short(id: &str) -> String {
    id.chars().take(8).collect()
}

fn one_line(markdown: &str) -> String {
    markdown.replace(['\r', '\n'], " ")
}

/// Documento em construção: acumula o texto e os ids de resultado citados.
#[derive(Default)]
pub struct Doc {
    out: String,
    sources: BTreeSet<String>,
}

impl Doc {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&mut self, raw: &str) {
        self.out.push_str(&format!("# {}\n\n", esc(raw)));
    }

    pub fn h2(&mut self, raw: &str) {
        self.out.push_str(&format!("## {}\n\n", esc(raw)));
    }

    pub fn h3(&mut self, raw: &str) {
        self.out.push_str(&format!("### {}\n\n", esc(raw)));
    }

    /// Parágrafo. `markdown` já vem seguro: peças de dados passaram por
    /// [`esc`] ou [`code`] quem chama.
    pub fn para(&mut self, markdown: &str) {
        self.out.push_str(&one_line(markdown));
        self.out.push_str("\n\n");
    }

    pub fn quote(&mut self, markdown: &str) {
        self.out.push_str(&format!("> {}\n\n", one_line(markdown)));
    }

    pub fn bullets(&mut self, items: &[String]) {
        for item in items {
            self.out.push_str(&format!("- {}\n", one_line(item)));
        }
        self.out.push('\n');
    }

    /// Seção sem dados: diz que não há registro, em vez de omitir.
    pub fn none(&mut self, what: &str) {
        self.out
            .push_str(&format!("*Sem registro: {}.*\n\n", esc(what)));
    }

    /// Tabela. Os cabeçalhos são literais do programa; as células já vêm
    /// seguras. Linha com número errado de células é completada com `—`.
    pub fn table(&mut self, headers: &[&str], rows: &[Vec<String>]) {
        let width = headers.len();
        let header: Vec<String> = headers.iter().map(|h| esc(h)).collect();
        self.out.push_str(&format!("| {} |\n", header.join(" | ")));
        self.out
            .push_str(&format!("| {} |\n", vec!["---"; width].join(" | ")));
        for row in rows {
            let mut cells: Vec<String> = row
                .iter()
                .take(width)
                .map(|cell| {
                    let line = one_line(cell);
                    if line.trim().is_empty() {
                        "—".to_string()
                    } else {
                        line
                    }
                })
                .collect();
            cells.resize(width, "—".to_string());
            self.out.push_str(&format!("| {} |\n", cells.join(" | ")));
        }
        self.out.push('\n');
    }

    /// Registra um resultado como fonte do documento e devolve a citação
    /// curta, pronta para entrar numa célula ou frase.
    pub fn cite(&mut self, id: &str) -> String {
        self.sources.insert(id.to_string());
        code(&short(id))
    }

    pub fn finish(self) -> (String, Vec<String>) {
        let mut text = self.out.trim_end().to_string();
        text.push('\n');
        (text, self.sources.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Barras verticais que dividem colunas (as não escapadas).
    fn unescaped_pipes(line: &str) -> usize {
        let mut count = 0;
        let mut escaped = false;
        for ch in line.chars() {
            match ch {
                '\\' if !escaped => {
                    escaped = true;
                    continue;
                }
                '|' if !escaped => count += 1,
                _ => {}
            }
            escaped = false;
        }
        count
    }

    #[test]
    fn esc_neutralises_markdown_html_links_and_control_characters() {
        let hostile = "a|b\nc\r\n<script>alert(1)</script> [x](javascript:alert(1)) `rm -rf` *b* _i_ # h\u{1b}[31mred";
        let escaped = esc(hostile);
        assert!(!escaped.contains('\n') && !escaped.contains('\r'));
        assert!(!escaped.contains('\u{1b}'));
        assert!(!escaped.contains("<script>"), "{escaped}");
        // Cada caractere especial aparece precedido de barra invertida.
        assert!(escaped.contains("\\<script\\>"));
        assert!(
            escaped.contains("\\[x\\]"),
            "sem colchete cru, não vira link: {escaped}"
        );
        assert!(escaped.contains("a\\|b"));
        assert!(escaped.contains("\\`rm -rf\\`"));
        assert!(escaped.contains("\\# h"));
        assert_eq!(esc("   "), "");
        assert_eq!(esc("a\n\n   b"), "a b");
    }

    #[test]
    fn bidi_and_invisible_characters_become_visible_markers_in_text_and_code() {
        let hostile = "relatorio\u{202E}txt.exe\u{200B}";
        let escaped = esc(hostile);
        assert!(!escaped.contains('\u{202E}') && !escaped.contains('\u{200B}'));
        assert!(
            escaped.contains("⟦U+202E⟧") && escaped.contains("⟦U+200B⟧"),
            "{escaped}"
        );
        let coded = code(hostile);
        assert!(!coded.contains('\u{202E}'), "{coded}");
        assert!(coded.contains("⟦U+202E⟧"), "{coded}");
    }

    #[test]
    fn code_never_contains_a_raw_backtick_or_pipe_and_empty_becomes_a_dash() {
        assert_eq!(code("a`b|c\nd"), "`a'b\\|c d`");
        assert_eq!(code(""), "—");
        assert_eq!(code("  \n "), "—");
    }

    #[test]
    fn table_keeps_every_row_the_same_width_even_with_hostile_cells() {
        let mut doc = Doc::new();
        let rows = vec![
            vec![esc("x|y\nz"), code("a|b"), esc("<b>")],
            vec![esc("só uma")],
            vec![esc("a"), esc("b"), esc("c"), esc("excedente")],
            vec![String::new(), "  ".into(), "ok".into()],
        ];
        doc.table(&["A", "B", "C"], &rows);
        let (text, _) = doc.finish();
        let table_lines: Vec<&str> = text.lines().filter(|l| l.starts_with('|')).collect();
        assert_eq!(table_lines.len(), 2 + rows.len());
        for line in &table_lines {
            assert_eq!(unescaped_pipes(line), 4, "{line}");
        }
        assert!(!text.contains("<b>"));
    }

    #[test]
    fn cite_registers_the_full_id_and_prints_the_short_one() {
        let mut doc = Doc::new();
        let shown = doc.cite("0123456789abcdef");
        assert_eq!(shown, "`01234567`");
        doc.cite("0123456789abcdef");
        doc.cite("aaaaaaaa-1");
        let (text, sources) = doc.finish();
        assert_eq!(text, "\n");
        assert_eq!(sources, vec!["0123456789abcdef", "aaaaaaaa-1"]);
    }

    #[test]
    fn none_says_there_is_no_record_instead_of_omitting_the_section() {
        let mut doc = Doc::new();
        doc.none("nenhum achado");
        let (text, _) = doc.finish();
        assert_eq!(text, "*Sem registro: nenhum achado.*\n");
    }
}
