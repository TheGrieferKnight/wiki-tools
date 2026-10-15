use super::markup::{br_segments, clean, extract_files, template_params};
use super::table::wikitable_rows;
use super::{ActiveEffect, ArmorPiece};

/// Pages describe their pieces either with `{{ArmorSetTable}}` or with a
/// plain wikitable.
pub(super) fn parse_pieces(text: &str) -> Result<Vec<ArmorPiece>, String> {
    if text.contains("{{ArmorSetTable") {
        parse_pieces_template(text)
    } else {
        parse_pieces_wikitable(text)
    }
}

// parse_pieces_template is for the ArmorSetTable template Joshy made later.
// parse_pieces_wikitable is for the earlier Armor Set pages.

/// Pieces are numbered params (`piece1_name`, `piece2_name`, ...) and all
/// share one `location`.
fn parse_pieces_template(text: &str) -> Result<Vec<ArmorPiece>, String> {
    let map = template_params(text, "ArmorSetTable")?;
    let get = |key: &str| map.get(key).map_or("", String::as_str);

    let location = get("location");
    let obtain_images = extract_files(location);
    let obtain_location = clean(location);

    let mut pieces = Vec::new();
    for n in 1.. {
        let Some(name) = map.get(&format!("piece{n}_name")) else {
            break;
        };
        let icons = ["img", "img2"]
            .iter()
            .filter_map(|key| map.get(&format!("piece{n}_{key}")))
            .filter(|s| !s.is_empty())
            .cloned()
            .collect();

        let mut piece = ArmorPiece {
            name: clean(name),
            icons,
            description: clean(get(&format!("piece{n}_desc"))),
            obtain_images: obtain_images.clone(),
            obtain_location: obtain_location.clone(),
            ..Default::default()
        };
        for seg in br_segments(get(&format!("piece{n}_effect"))) {
            parse_effect(&mut piece, seg);
        }
        pieces.push(piece);
    }
    Ok(pieces)
}

/// The header row decides the layout:
/// - 3 columns: name (with icons) | description | obtain location
/// - 4 columns: image | name | description | passive
fn parse_pieces_wikitable(text: &str) -> Result<Vec<ArmorPiece>, String> {
    let table_start = text.find("{|").ok_or("no wikitable found")?;
    let table_end = text[table_start..]
        .find("\n|}")
        .ok_or("unterminated wikitable")?
        + table_start;
    let table = &text[table_start..table_end];

    let columns = table
        .lines()
        .find(|line| line.starts_with('!'))
        .map_or(3, |header| header.matches("!!").count() + 1);

    wikitable_rows(table)
        .iter()
        .enumerate()
        .map(|(i, row)| {
            if row.len() != columns {
                return Err(format!(
                    "row {} expected {columns} cells, got {}",
                    i + 1,
                    row.len()
                ));
            }
            match columns {
                3 => Ok(parse_three_column_row(row)),
                4 => Ok(parse_four_column_row(row)),
                _ => Err(format!("unsupported wikitable with {columns} columns")),
            }
        })
        .collect()
}

fn parse_three_column_row(cells: &[String]) -> ArmorPiece {
    let [name_cell, desc_cell, obtain_cell] = cells else {
        unreachable!("column count is checked by the caller");
    };

    // The name cell holds the icon or icons first, with the name after the last <br/>.
    let mut piece = ArmorPiece {
        name: clean(name_cell.rsplit("<br/>").next().unwrap_or("")),
        icons: extract_files(name_cell),
        obtain_images: extract_files(obtain_cell),
        obtain_location: clean(obtain_cell),
        ..Default::default()
    };
    parse_description_cell(&mut piece, desc_cell);
    piece
}

fn parse_four_column_row(cells: &[String]) -> ArmorPiece {
    let [image_cell, name_cell, desc_cell, passive_cell] = cells else {
        unreachable!("column count is checked by the caller");
    };

    let mut piece = ArmorPiece {
        name: clean(name_cell),
        icons: extract_files(image_cell),
        ..Default::default()
    };
    parse_description_cell(&mut piece, desc_cell);

    // Text without a "Passive:" or "Active" prefix is part of the passive.
    for seg in br_segments(passive_cell) {
        if !parse_effect(&mut piece, seg) {
            piece.passive = Some(clean(seg));
        }
    }
    piece
}

/// A description cell can have effects, italic flavor text and plain
/// description lines. Only the first italic segment counts as flavor text.
fn parse_description_cell(piece: &mut ArmorPiece, cell: &str) {
    for seg in br_segments(cell) {
        if parse_effect(piece, seg) {
            continue;
        }
        if piece.flavor.is_empty() && seg.starts_with("''") && seg.ends_with("''") {
            piece.flavor = clean(seg);
        } else {
            if !piece.description.is_empty() {
                piece.description.push('\n');
            }
            piece.description.push_str(&clean(seg));
        }
    }
}

/// Sets the passive or active effect from a segment. Returns false if the
/// segment is neither.
fn parse_effect(piece: &mut ArmorPiece, seg: &str) -> bool {
    if let Some(rest) = seg.strip_prefix("'''Passive:'''") {
        piece.passive = Some(clean(rest));
        true
    } else if let Some(rest) = seg.strip_prefix("'''Active (") {
        // A malformed active line is still consumed, just not stored.
        if let Some((name, effect)) = rest.split_once("):'''") {
            piece.active = Some(ActiveEffect {
                name: clean(name),
                effect: clean(effect),
            });
        }
        true
    } else {
        false
    }
}
