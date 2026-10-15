use super::markup::split_top_level;

/// Splits a wikitable into rows of cell contents. A cell with `rowspan` is
/// repeated in the rows it covers, so each column lines up across rows.
/// Header (`!`) and caption (`|+`) lines are skipped.
pub(super) fn wikitable_rows(table: &str) -> Vec<Vec<String>> {
    let raw_rows = raw_cells(table);

    let mut rows = Vec::new();
    // Per column: content still spanning down, and how many rows it has left.
    let mut carried: Vec<Option<(String, usize)>> = Vec::new();

    for raw_row in raw_rows.into_iter().filter(|r| !r.is_empty()) {
        let mut row = Vec::new();
        let mut cells = raw_row.into_iter();
        for col in 0.. {
            if carried.len() <= col {
                carried.push(None);
            }
            if let Some((content, remaining)) = carried[col].take() {
                if remaining > 1 {
                    carried[col] = Some((content.clone(), remaining - 1));
                }
                row.push(content);
            } else if let Some((content, rowspan)) = cells.next() {
                if rowspan > 1 {
                    carried[col] = Some((content.clone(), rowspan - 1));
                }
                row.push(content);
            } else {
                break;
            }
        }
        rows.push(row);
    }
    rows
}

/// Collects `(content, rowspan)` per cell, grouped by `|-` row separators.
fn raw_cells(table: &str) -> Vec<Vec<(String, usize)>> {
    let mut rows: Vec<Vec<(String, usize)>> = Vec::new();
    for line in table.lines() {
        if line.starts_with("|-") {
            rows.push(Vec::new());
            continue;
        }
        if line.starts_with("|+") {
            continue;
        }
        let Some(rest) = line.strip_prefix('|') else {
            continue;
        };
        if rows.is_empty() {
            rows.push(Vec::new());
        }
        let row = rows.last_mut().expect("row was just pushed");

        for raw in rest.split("||") {
            // `attrs | content` — a single top-level pipe separates them.
            let parts = split_top_level(raw, '|');
            let rowspan = match parts.as_slice() {
                [attrs, _content] => rowspan_of(attrs),
                _ => 1,
            };
            let content = parts.last().unwrap_or(&"").trim().to_string();
            row.push((content, rowspan));
        }
    }
    rows
}

fn rowspan_of(attrs: &str) -> usize {
    attrs
        .split_whitespace()
        .find_map(|attr| attr.strip_prefix("rowspan="))
        .and_then(|v| v.trim_matches('"').parse().ok())
        .unwrap_or(1)
}
