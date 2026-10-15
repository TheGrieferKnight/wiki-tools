use std::{collections::HashMap, fs};

use super::{ProtectorLookup, general};
use crate::AppError;

#[derive(Debug)]
pub struct ArmorSetIDs {
    pub set_name: String,
    pub helm_id: i64,
    pub helm_name: String,
    pub armor_id: i64,
    pub armor_name: String,
    pub gauntlet_id: i64,
    pub gauntlet_name: String,
    pub greaves_id: i64,
    pub greaves_name: String,
    pub helm_altered_id: Option<i64>,
    pub armor_altered_id: Option<i64>,
    pub gauntlet_altered_id: Option<i64>,
    pub greaves_altered_id: Option<i64>,
}

/// In wikitext / sensible order
impl ArmorSetIDs {
    pub fn piece_ids(&self) -> [i64; 4] {
        [
            self.helm_id,
            self.armor_id,
            self.gauntlet_id,
            self.greaves_id,
        ]
    }

    pub fn piece_names(&self) -> [&str; 4] {
        [
            &self.helm_name,
            &self.armor_name,
            &self.gauntlet_name,
            &self.greaves_name,
        ]
    }
}

/// Builds armor sets by scanning EquipParamProtector ids.
/// Sets live at multiples of 10000, pieces at +0/+100/+200/+300, and altered
/// variants at +1000 on top of that.
#[allow(dead_code)]
pub fn discover_armor_set_ids(id_lookup: &ProtectorLookup) -> Vec<ArmorSetIDs> {
    let mut sets = Vec::with_capacity(200);

    for id in (1..999).map(|i| i * 10000) {
        let [Some(helm), Some(armor), Some(gauntlet), Some(greaves)] =
            [0, 100, 200, 300].map(|offset| id_lookup.get(&(id + offset)))
        else {
            continue;
        };
        let altered = |offset| {
            let alt = id + 1000 + offset;
            id_lookup.contains_key(&alt).then_some(alt)
        };

        sets.push(ArmorSetIDs {
            set_name: String::new(),
            helm_id: helm.id,
            helm_name: helm.name.clone(),
            armor_id: armor.id,
            armor_name: armor.name.clone(),
            gauntlet_id: gauntlet.id,
            gauntlet_name: gauntlet.name.clone(),
            greaves_id: greaves.id,
            greaves_name: greaves.name.clone(),
            helm_altered_id: altered(0),
            armor_altered_id: altered(100),
            gauntlet_altered_id: altered(200),
            greaves_altered_id: altered(300),
        });
    }

    sets
}

/// Loads armor sets from a `{:#?}` dump of `Vec<ArmorSetIDs>`.
pub fn load_armor_set_ids(path: &str) -> Result<Vec<ArmorSetIDs>, AppError> {
    let text = fs::read_to_string(path).map_err(|e| general(format!("read {path}: {e}")))?;

    let mut sets = Vec::new();
    let mut fields: Option<HashMap<String, String>> = None;
    let mut lines = text.lines().map(str::trim);

    while let Some(line) = lines.next() {
        if line == "ArmorSetIDs {" {
            fields = Some(HashMap::new());
        } else if line == "}," {
            if let Some(f) = fields.take() {
                let set = parse_fields(&f).map_err(|e| general(format!("{path}: {e}")))?;
                sets.push(set);
            }
        } else if let Some(f) = fields.as_mut()
            && let Some((key, value)) = line.trim_end_matches(',').split_once(": ")
        {
            let mut value = value.to_owned();
            if value.ends_with('(') {
                // Pretty-printed `Some` spans lines: `Some(`, `61000,`, `),`
                for inner in lines.by_ref() {
                    if inner == ")," {
                        break;
                    }
                    value.push_str(inner.trim_end_matches(','));
                }
                value.push(')');
            }
            f.insert(key.to_owned(), value);
        }
    }

    Ok(sets)
}

fn parse_fields(fields: &HashMap<String, String>) -> Result<ArmorSetIDs, String> {
    let get = |key: &str| {
        fields
            .get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("missing field `{key}`"))
    };

    let id = |key: &str| -> Result<i64, String> {
        get(key)?.parse().map_err(|e| format!("bad `{key}`: {e}"))
    };

    // Reverses the escaping done by `{:?}` on strings (quotes and backslashes).
    let name = |key: &str| -> Result<String, String> {
        let raw = get(key)?;
        raw.strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .map(|s| s.replace("\\\"", "\"").replace("\\\\", "\\"))
            .ok_or_else(|| format!("bad `{key}`: {raw}"))
    };

    let opt_id = |key: &str| -> Result<Option<i64>, String> {
        match get(key)? {
            "None" => Ok(None),
            raw => raw
                .strip_prefix("Some(")
                .and_then(|s| s.strip_suffix(')'))
                .and_then(|s| s.parse().ok())
                .map(Some)
                .ok_or_else(|| format!("bad `{key}`: {raw}")),
        }
    };

    Ok(ArmorSetIDs {
        set_name: name("set_name")?,
        helm_id: id("helm_id")?,
        helm_name: name("helm_name")?,
        armor_id: id("armor_id")?,
        armor_name: name("armor_name")?,
        gauntlet_id: id("gauntlet_id")?,
        gauntlet_name: name("gauntlet_name")?,
        greaves_id: id("greaves_id")?,
        greaves_name: name("greaves_name")?,
        helm_altered_id: opt_id("helm_altered_id")?,
        armor_altered_id: opt_id("armor_altered_id")?,
        gauntlet_altered_id: opt_id("gauntlet_altered_id")?,
        greaves_altered_id: opt_id("greaves_altered_id")?,
    })
}
