//! Neutralização de caracteres invisíveis e de direção de texto (bidi) em
//! texto que vem de evidência.
//!
//! O conteúdo de um log é entrada hostil. Além de caracteres de controle
//! (que o terminal e o Markdown já tratam), o Unicode tem caracteres de
//! formatação que **não aparecem** mas mudam o que o analista lê: os de
//! direção de texto (`U+202E` inverte a ordem de exibição, "Trojan Source"),
//! os de largura zero e os de etiqueta. Um nome de arquivo
//! `relatorio\u{202E}txt.exe` aparece como `relatorioexe.txt`.
//!
//! A regra aqui é **mostrar, não apagar**: cada caractere vira um marcador
//! visível `⟦U+202E⟧`. O analista vê que existe algo ali, e o texto original
//! continua intacto no banco e na evidência.

use std::borrow::Cow;

/// `true` para caracteres de formatação que não têm glifo visível ou que
/// alteram a direção da exibição.
pub fn is_invisible_or_bidi(ch: char) -> bool {
    matches!(
        ch,
        '\u{00AD}'                // hífen condicional
        | '\u{061C}'              // marca de letra árabe
        | '\u{180E}'              // separador de vogal mongol
        | '\u{200B}'..='\u{200F}' // largura zero, ZWNJ, ZWJ, LRM, RLM
        | '\u{202A}'..='\u{202E}' // LRE, RLE, PDF, LRO, RLO
        | '\u{2060}'..='\u{2064}' // junção de palavra e operadores invisíveis
        | '\u{2066}'..='\u{2069}' // isolamentos LRI, RLI, FSI, PDI
        | '\u{FEFF}'              // BOM / ZWNBSP
        | '\u{FFF9}'..='\u{FFFB}' // anotação interlinear
        | '\u{E0000}'..='\u{E007F}' // caracteres de etiqueta
    )
}

/// Marcador visível de um caractere neutralizado.
fn marker(ch: char) -> String {
    format!("⟦U+{:04X}⟧", u32::from(ch))
}

/// Troca cada caractere invisível ou de direção por um marcador visível.
/// Não aloca quando não há nada a trocar.
pub fn neutralize(raw: &str) -> Cow<'_, str> {
    if !raw.chars().any(is_invisible_or_bidi) {
        return Cow::Borrowed(raw);
    }
    let mut out = String::with_capacity(raw.len() + 16);
    for ch in raw.chars() {
        if is_invisible_or_bidi(ch) {
            out.push_str(&marker(ch));
        } else {
            out.push(ch);
        }
    }
    Cow::Owned(out)
}

/// Conveniência: devolve `String` já neutralizada.
pub fn neutralized(raw: &str) -> String {
    neutralize(raw).into_owned()
}

#[cfg(test)]
mod tests {
    use super::{is_invisible_or_bidi, neutralize, neutralized};
    use std::borrow::Cow;

    #[test]
    fn plain_text_is_returned_without_allocation() {
        assert!(matches!(
            neutralize("logon 4624 de 10.0.0.5"),
            Cow::Borrowed(_)
        ));
        assert_eq!(neutralized("ação — çãõ 日本語"), "ação — çãõ 日本語");
    }

    #[test]
    fn a_right_to_left_override_becomes_a_visible_marker() {
        // "relatorio" + RLO + "txt.exe": aparece como "relatorioexe.txt".
        let hostile = "relatorio\u{202E}txt.exe";
        let shown = neutralized(hostile);
        assert_eq!(shown, "relatorio⟦U+202E⟧txt.exe");
        assert!(!shown.contains('\u{202E}'));
    }

    #[test]
    fn every_directional_isolate_and_embedding_is_covered() {
        for code in (0x202A..=0x202E)
            .chain(0x2066..=0x2069)
            .chain([0x200E, 0x200F, 0x061C])
        {
            let ch = char::from_u32(code).expect("valid scalar");
            assert!(
                is_invisible_or_bidi(ch),
                "U+{code:04X} deve ser neutralizado"
            );
            assert!(neutralized(&ch.to_string()).contains(&format!("U+{code:04X}")));
        }
    }

    #[test]
    fn zero_width_bom_soft_hyphen_and_tag_characters_are_covered() {
        for ch in [
            '\u{200B}',
            '\u{200C}',
            '\u{200D}',
            '\u{2060}',
            '\u{FEFF}',
            '\u{00AD}',
            '\u{E0041}',
        ] {
            assert!(is_invisible_or_bidi(ch), "{:?}", ch);
        }
        assert_eq!(neutralized("a\u{200B}b"), "a⟦U+200B⟧b");
        assert_eq!(neutralized("\u{E0041}"), "⟦U+E0041⟧");
    }

    #[test]
    fn ordinary_punctuation_and_spaces_are_not_touched() {
        for ch in [' ', '\t', '-', '–', '—', '\u{00A0}', '·', '⟦', '⟧'] {
            assert!(!is_invisible_or_bidi(ch), "{:?}", ch);
        }
    }

    #[test]
    fn neutralizing_twice_changes_nothing_more() {
        let once = neutralized("a\u{202E}b");
        assert_eq!(neutralized(&once), once);
    }
}
