use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::error::AppError;
use crate::models::common::magic::Magic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellBasics {
    pub fp: i64,
    pub stamina: i64,
    pub req_int: i64,
    pub req_fth: i64,
    pub req_arc: i64,
}

pub fn find_spell_basics(spell_name: &str) -> Result<SpellBasics, AppError> {
    let (category, name) = parse_query(spell_name)?;
    spell_index()?.find(spell_name, category, &name)
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpellCategory {
    Incantation,
    Sorcery,
}

#[derive(Default)]
struct Matches {
    first: Option<SpellBasics>,
    ambiguous: bool,
}

impl Matches {
    fn insert(&mut self, basics: SpellBasics) {
        if self.first.is_some() {
            self.ambiguous = true;
        } else {
            self.first = Some(basics);
        }
    }

    fn resolve(&self, spell_name: &str) -> Result<SpellBasics, AppError> {
        let first = self.first.ok_or_else(|| not_found(spell_name))?;

        if self.ambiguous {
            return Err(AppError::AmbiguousSpell {
                name: spell_name.to_owned(),
                first,
            });
        }

        Ok(first)
    }
}

#[derive(Default)]
struct NameMatches {
    all: Matches,
    incantations: Matches,
    sorceries: Matches,
}

#[derive(Default)]
pub(crate) struct SpellIndex {
    by_name: HashMap<String, NameMatches>,
}

impl SpellIndex {
    pub(crate) fn insert(&mut self, title: &str, basics: SpellBasics) {
        let Some((category, name)) = parse_title(title) else {
            return;
        };

        let name = normalize_name(name);
        if name.is_empty() {
            return;
        }

        let matches = self.by_name.entry(name).or_default();
        matches.all.insert(basics);

        match category {
            Some(SpellCategory::Incantation) => {
                matches.incantations.insert(basics);
            }
            Some(SpellCategory::Sorcery) => {
                matches.sorceries.insert(basics);
            }
            None => {}
        }
    }

    pub(crate) fn find(
        &self,
        spell_name: &str,
        category: Option<SpellCategory>,
        normalized_name: &str,
    ) -> Result<SpellBasics, AppError> {
        let matches = self
            .by_name
            .get(normalized_name)
            .ok_or_else(|| not_found(spell_name))?;

        let matches = match category {
            Some(SpellCategory::Incantation) => &matches.incantations,
            Some(SpellCategory::Sorcery) => &matches.sorceries,
            None => &matches.all,
        };

        matches.resolve(spell_name)
    }
}

fn spell_index() -> Result<&'static SpellIndex, AppError> {
    static INDEX: OnceLock<SpellIndex> = OnceLock::new();
    static INITIALIZATION: Mutex<()> = Mutex::new(());

    if let Some(index) = INDEX.get() {
        return Ok(index);
    }

    let _guard = INITIALIZATION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    // Check if another thread has initialized the index
    if let Some(index) = INDEX.get() {
        return Ok(index);
    }

    let mut reader = csv::Reader::from_reader(include_bytes!("../csv/common/Magic.csv").as_slice());
    let mut index = SpellIndex::default();

    for magic in reader.deserialize::<Magic>() {
        let magic = magic?;

        index.insert(
            &magic.name,
            SpellBasics {
                fp: magic.mp,
                stamina: magic.stamina,
                req_int: magic.requirement_intellect,
                req_fth: magic.requirement_faith,
                req_arc: magic.requirement_luck,
            },
        );
    }

    Ok(INDEX.get_or_init(|| index))
}

fn normalize_name(name: &str) -> String {
    name.replace('_', " ")
        .replace(['‘', '’'], "'")
        .trim()
        .to_lowercase()
}

fn parse_title(title: &str) -> Option<(Option<SpellCategory>, &str)> {
    let title = title.trim();

    let Some(qualified) = title.strip_prefix('[') else {
        return Some((None, title));
    };

    let (category, name) = qualified.split_once(']')?;
    let category = category.trim();

    let category = if category.eq_ignore_ascii_case("incantation") {
        SpellCategory::Incantation
    } else if category.eq_ignore_ascii_case("sorcery") {
        SpellCategory::Sorcery
    } else {
        return None;
    };

    Some((Some(category), name.trim()))
}

pub(crate) fn parse_query(spell_name: &str) -> Result<(Option<SpellCategory>, String), AppError> {
    let (category, name) = parse_title(spell_name).ok_or_else(|| not_found(spell_name))?;
    let name = normalize_name(name);

    if name.is_empty() {
        return Err(AppError::NotFound {
            record_type: "Magic",
            field: "empty spell name".to_owned(),
        });
    }

    Ok((category, name))
}

fn not_found(spell_name: &str) -> AppError {
    AppError::NotFound {
        record_type: "Magic",
        field: format!("spell name = {spell_name}"),
    }
}
