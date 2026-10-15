use super::{ActiveEffect, ArmorPiece, FmgMap};

const CONDITION_PREFIX: &str = "If you do not have";

/// Armor fields parsed from one FMG description entry.
#[derive(Debug, Default, PartialEq)]
pub struct FmgPiece {
    pub description: String,
    pub passive: Option<String>,
    pub condition: Option<String>,
    /// `(name, effect)`
    pub active: Option<(String, String)>,
}

/// Expected layout of an entry:
///
/// 1. Everything up to the first blank line is the description.
/// 2. Following paragraph holds the effects, one per line:
///    - line starts with "If you do not have" means it`s the active condition
///    - line containing " - " or " – " means it`s the active effect (`name - effect`)
///    - any other line is passive text (multiple lines are joined with using newlines)
/// 3. Everything after that is ignored.
///
/// If a paragraph has more than one condition or active line, choose the last one
pub fn parse_fmg_piece(text: &str) -> FmgPiece {
    let mut lines = text.lines().map(str::trim);

    let description = lines
        .by_ref()
        .take_while(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    let mut piece = FmgPiece {
        description,
        ..Default::default()
    };

    for line in lines.take_while(|l| !l.is_empty()) {
        if line.starts_with(CONDITION_PREFIX) {
            piece.condition = Some(clean_condition(line));
        } else if let Some((name, effect)) = split_active(line) {
            piece.active = Some((name.trim().to_owned(), effect.trim().to_owned()));
        } else {
            match &mut piece.passive {
                Some(p) => {
                    p.push('\n');
                    p.push_str(line);
                }
                None => piece.passive = Some(line.to_owned()),
            }
        }
    }

    piece
}

/// Overwrites the description, passive and active fields of `piece` with the
/// FMG entry `id`. If there is a condition, it goes in front of the active
/// effect text.
pub fn apply_fmg(piece: &mut ArmorPiece, fmg: &FmgMap, id: u32) -> Result<(), String> {
    let raw = fmg
        .get(&id)
        .ok_or_else(|| format!("no FMG entry with id {id}"))?;
    let parsed = parse_fmg_piece(raw);

    piece.description = parsed.description;
    piece.passive = parsed.passive;
    piece.active = parsed.active.map(|(name, effect)| {
        let effect = match parsed.condition {
            Some(cond) => format!("{cond}, {}", lower_first(&effect)),
            None => effect,
        };
        ActiveEffect { name, effect }
    });

    Ok(())
}

/// FMG uses both a hyphen and en dash separator.
fn split_active(line: &str) -> Option<(&str, &str)> {
    line.split_once(" - ").or_else(|| line.split_once(" – "))
}

/// "If ..., this item has the following effect:" -> "If ..."
fn clean_condition(line: &str) -> String {
    line.trim_end_matches(':')
        .trim_end_matches("this item has the following effect")
        .trim_end_matches(", ")
        .trim()
        .to_owned()
}

fn lower_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_layout() {
        let text = "Line one\nLine two\n\n\
                    Passive A\n\
                    If you do not have X, this item has the following effect:\n\
                    Blast - Deals damage\n\
                    Passive B\n\n\
                    ignored";
        let p = parse_fmg_piece(text);
        assert_eq!(p.description, "Line one\nLine two");
        assert_eq!(p.passive.as_deref(), Some("Passive A\nPassive B"));
        assert_eq!(p.condition.as_deref(), Some("If you do not have X"));
        assert_eq!(p.active, Some(("Blast".into(), "Deals damage".into())));
    }

    #[test]
    fn lowercases_first_char() {
        assert_eq!(lower_first("Deals"), "deals");
        assert_eq!(lower_first(""), "");
    }
}
