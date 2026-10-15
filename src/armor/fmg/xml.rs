use std::{collections::HashMap, fs};

/// Text entries keyed by FMG id.
pub type FmgMap = HashMap<u32, String>;

const OPEN: &str = "<text id=\"";
const CLOSE: &str = "</text>";

pub fn load_fmg(path: &str) -> Result<FmgMap, String> {
    let xml = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    parse_fmg_xml(&xml)
}

/// Minimal scanner for `<text id="N">...</text>` entries. FMG exports are flat
/// and machine-generated, so a full XML parser isn't needed. Anything outside
/// `<text>` elements is skipped.
pub fn parse_fmg_xml(xml: &str) -> Result<FmgMap, String> {
    let mut entries = HashMap::new();
    let mut rest = xml;

    while let Some(start) = rest.find(OPEN) {
        let after = &rest[start + OPEN.len()..];

        let id_end = after.find('"').ok_or("unterminated id attribute")?;
        let raw_id = &after[..id_end];
        let id: u32 = raw_id
            .parse()
            .map_err(|e| format!("bad id `{raw_id}`: {e}"))?;

        let tag_end = after.find('>').ok_or("unterminated <text> tag")?;

        // `<text id="N" />` marks an entry that exists but has no text.
        if after[..tag_end].ends_with('/') {
            entries.insert(id, String::new());
            rest = &after[tag_end + 1..];
            continue;
        }

        let body = &after[tag_end + 1..];
        let body_end = body
            .find(CLOSE)
            .ok_or_else(|| format!("missing {CLOSE} for id {id}"))?;

        // Normalized after decoding, so an encoded `&#13;` before a newline
        // is turned into a plain `\n` too.
        let text = decode_entities(&body[..body_end]).replace("\r\n", "\n");
        entries.insert(id, text);

        rest = &body[body_end + CLOSE.len()..];
    }

    Ok(entries)
}

/// Decodes the five predefined XML entities plus numeric character
/// references. Anything else that is unknown or malformed stays as it is.
fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;

    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];

        let Some(semi) = rest.find(';') else { break };

        match decode_entity(&rest[1..semi]) {
            Some(c) => {
                out.push(c);
                rest = &rest[semi + 1..];
            }
            // Emit the `&` on its own and keep scanning right after it, so a
            // stray `&` doesn't swallow a valid entity that follows it.
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }

    out.push_str(rest);
    out
}

/// `name` is the part between `&` and `;`.
fn decode_entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let code = if let Some(hex) = name.strip_prefix("#x") {
                u32::from_str_radix(hex, 16).ok()?
            } else {
                name.strip_prefix('#')?.parse().ok()?
            };
            char::from_u32(code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_entries() {
        let xml = r#"<fmg><text id="1">a &amp; b</text><text id="2" /></fmg>"#;
        let map = parse_fmg_xml(xml).unwrap();
        assert_eq!(map[&1], "a & b");
        assert_eq!(map[&2], "");
    }

    #[test]
    fn decodes_entities() {
        assert_eq!(decode_entities("&lt;&#65;&#x42;&gt;"), "<AB>");
        assert_eq!(decode_entities("a & b &amp; c"), "a & b & c");
        assert_eq!(decode_entities("&bogus; &"), "&bogus; &");
    }
}
