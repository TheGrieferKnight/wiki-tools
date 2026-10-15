use std::collections::HashMap;

/// Reads `|key = value` lines of the first template. Nested templates are ignored.
pub(super) fn template_params(text: &str, name: &str) -> Result<HashMap<String, String>, String> {
    let open = format!("{{{{{name}");
    let start = text
        .find(&open)
        .ok_or_else(|| format!("no {{{{{name}}}}} template found"))?;
    let body_start = start + open.len();
    let end = text[body_start..]
        .find("}}")
        .ok_or_else(|| format!("unterminated {name} template"))?
        + body_start;

    Ok(text[body_start..end]
        .lines()
        .filter_map(|line| line.trim().strip_prefix('|'))
        .filter_map(|param| param.split_once('='))
        .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_string()))
        .collect())
}

/// Non-empty, trimmed segments between `<br/>` tags.
pub(super) fn br_segments(s: &str) -> impl Iterator<Item = &str> {
    s.split("<br/>").map(str::trim).filter(|s| !s.is_empty())
}

/// Splits on `sep` but ignores separators inside `[[...]]`.
pub(super) fn split_top_level(s: &str, sep: char) -> Vec<&str> {
    let b = s.as_bytes();
    let sep = sep as u8;
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut last = 0;
    let mut i = 0;
    while i < b.len() {
        match &b[i..(i + 2).min(b.len())] {
            b"[[" => {
                depth += 1;
                i += 2;
                continue;
            }
            b"]]" => {
                depth -= 1;
                i += 2;
                continue;
            }
            _ => {}
        }
        if b[i] == sep && depth == 0 {
            parts.push(&s[last..i]);
            last = i + 1;
        }
        i += 1;
    }
    parts.push(&s[last..]);
    parts
}

/// Returns the file names of all `[[File:...]]` links, in order.
pub(super) fn extract_files(s: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find("[[File:") {
        let after = &rest[start + "[[File:".len()..];
        let end = after.find(['|', ']']).unwrap_or(after.len());
        files.push(after[..end].trim().to_string());
        rest = &after[end..];
    }
    files
}

/// Replaces `[[target|label]]` with `label` (or `target`) and drops
/// `[[File:...]]` links entirely. An unclosed `[[` is kept as is.
fn resolve_links(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find("[[") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let inner = &after[..end];
        if !inner.starts_with("File:") {
            out.push_str(inner.rsplit('|').next().unwrap_or(inner));
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

/// Turns wiki markup into plain text
pub(super) fn clean(s: &str) -> String {
    resolve_links(s)
        .replace("<br/>", "\n")
        .replace("'''", "")
        .replace("''", "")
        .trim()
        .to_string()
}
