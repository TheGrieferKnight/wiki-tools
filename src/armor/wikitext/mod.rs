mod markup;
mod pieces;
mod table;

use super::{ActiveEffect, ArmorPiece, ArmorSet, HistoryEntry};
use markup::{clean, split_top_level, template_params};
use std::collections::HashMap;

pub async fn parse_armor_set(wikitext: &str) -> Result<ArmorSet, String> {
    let infobox = template_params(wikitext, "ArmorSetInfo")?;

    Ok(ArmorSet {
        name: field(&infobox, "name")?,
        image: field(&infobox, "image")?,
        // `class` and `tier` are the only optional infobox fields.
        class: infobox.get("class").map(|v| clean(v)).unwrap_or_default(),
        tier: infobox.get("tier").map(|v| clean(v)).unwrap_or_default(),
        description: parse_description(wikitext),
        pieces: pieces::parse_pieces(wikitext)?,
        history: parse_history(wikitext),
        ..Default::default()
    })
}

fn field(map: &HashMap<String, String>, key: &str) -> Result<String, String> {
    map.get(key)
        .map(|v| clean(v))
        .ok_or_else(|| format!("missing infobox field `{key}`"))
}

/// The description is everything between the first level-1 heading line and
/// the "Armor Pieces" section (or the end of the page).
fn parse_description(text: &str) -> String {
    let Some(start) = text.find("\n= ") else {
        return String::new();
    };
    let body = text[start + 1..]
        .split_once('\n')
        .map_or("", |(_, rest)| rest);
    let end = body.find("== Armor Pieces ==").unwrap_or(body.len());
    clean(&body[..end])
}

/// Parses lines of HistoryItem template `{{HistoryItem|<version>|<text...>}}`. Pipes
/// after the version belong to the text.
fn parse_history(text: &str) -> Vec<HistoryEntry> {
    text.lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("{{HistoryItem|")
                .map(|r| r.trim_end_matches("}}"))
        })
        .map(|inner| {
            let parts = split_top_level(inner, '|');
            HistoryEntry {
                version: clean(parts[0]),
                text: clean(&parts[1..].join("|")),
            }
        })
        .collect()
}
