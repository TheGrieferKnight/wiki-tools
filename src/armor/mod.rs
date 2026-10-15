use std::{collections::HashMap, fs, path::Path};

use futures::{TryStreamExt, stream};

use crate::{AppError, models::common::equip_param_protector::EquipParamProtector};

pub mod fmg;
mod icons;
mod ids;
mod model;
mod stats;
mod template;
mod wiki_api;
pub mod wikitext;

pub use model::{ActiveEffect, ArmorPiece, ArmorSet, HistoryEntry};

use fmg::{apply_fmg, load_fmg};
use ids::{ArmorSetIDs, load_armor_set_ids};
use wiki_api::Wiki;
use wikitext::parse_armor_set;

const API_URL: &str = "https://wiki.convergencemod.com/api.php";
const PROTECTOR_CSV: &str =
    "/home/thegrieferknight/Projects/wiki-tools/csv/common/EquipParamProtector.csv";
const ARMOR_SET_IDS_PATH: &str = "output.txt";
const FMG_PATH: &str = "/home/thegrieferknight/Projects/wiki-tools/ProtectorCaption.fmg.xml";
/// Loaded first so base-game entries in FMG_PATH override DLC ones.
const DLC_FMG_PATH: &str =
    "/home/thegrieferknight/Projects/wiki-tools/ProtectorCaption_dlc01.fmg.xml";
const BACKUP_DIR: &str = "/home/thegrieferknight/Projects/wiki-tools/backups";
const OUTPUT_DIR: &str = "/home/thegrieferknight/Projects/wiki-tools/new-pages";

const MAX_CONCURRENT_SETS: usize = 8;

type ProtectorLookup = HashMap<i64, EquipParamProtector>;

pub(crate) fn general(message: impl Into<String>) -> AppError {
    AppError::General {
        message: message.into(),
    }
}

fn env_var(key: &str) -> Result<String, AppError> {
    std::env::var(key).map_err(|e| general(format!("{key}: {e}")))
}

pub async fn cache_csv() -> Result<(), AppError> {
    let mut reader = csv::Reader::from_path(PROTECTOR_CSV)?;
    println!("{reader:?}");

    let mut id_lookup = ProtectorLookup::new();
    for row in reader.deserialize::<EquipParamProtector>() {
        let row = row?;
        id_lookup.insert(row.id, row);
    }

    let armor_set_ids = load_armor_set_ids(ARMOR_SET_IDS_PATH)?;
    println!("{armor_set_ids:?}");

    let wiki = Wiki::login(
        API_URL,
        env_var("WIKI_BOT_USER")?,
        env_var("WIKI_BOT_PASS")?,
    )
    .await?;

    let (id_lookup, wiki) = (&id_lookup, &wiki);
    stream::iter(armor_set_ids.into_iter().map(Ok))
        .try_for_each_concurrent(MAX_CONCURRENT_SETS, |ids| {
            process_armor_set(wiki, id_lookup, ids)
        })
        .await
}

/// Backs up the current page, rebuilds it from params + FMG, and writes the
/// result to OUTPUT_DIR. Nothing is pushed back to the wiki except icons.
async fn process_armor_set(
    wiki: &Wiki,
    id_lookup: &ProtectorLookup,
    ids: ArmorSetIDs,
) -> Result<(), AppError> {
    let current = wiki
        .read_page(&ids.set_name.replace(' ', "_"))
        .await
        .map_err(|e| general(format!("{}: {e}", ids.set_name)))?;

    fs::create_dir_all(BACKUP_DIR)?;
    let backup_path = format!("{BACKUP_DIR}/{}.txt", ids.set_name);
    fs::write(&backup_path, current)?;

    let set = get_armor_set(wiki, id_lookup, &ids, &backup_path, FMG_PATH).await?;

    fs::create_dir_all(OUTPUT_DIR)?;
    fs::write(
        format!("{OUTPUT_DIR}/{}.txt", ids.set_name),
        template::render(&ids.set_name, &set),
    )?;

    Ok(())
}

pub async fn get_armor_set(
    wiki: &Wiki,
    id_lookup: &ProtectorLookup,
    ids: &ArmorSetIDs,
    wiki_path: &str,
    xml_path: &str,
) -> Result<ArmorSet, AppError> {
    let with_set = |e: String| general(format!("{}: {e}", ids.set_name));

    let pieces = lookup_pieces(id_lookup, ids).map_err(with_set)?;
    let mut set = build_armor_set(wiki_path, xml_path, ids)
        .await
        .map_err(with_set)?;

    for (piece, param) in set.pieces.iter_mut().zip(&pieces) {
        if piece.icons.is_empty() {
            let icon = icons::ensure_icon(wiki, param).await?;
            if !icon.is_empty() {
                piece.icons.push(icon);
            }
        }
    }

    stats::apply_stats(&mut set, &pieces);
    Ok(set)
}

/// Parses the wikitext, then overwrites the description, passive, and active
/// effect of each piece with the FMG entry for that piece's id.
pub async fn build_armor_set(
    wiki_path: &str,
    xml_path: &str,
    ids: &ArmorSetIDs,
) -> Result<ArmorSet, String> {
    let wiki = fs::read_to_string(wiki_path).map_err(|e| format!("read {wiki_path}: {e}"))?;
    let mut set = if wiki.trim().is_empty() {
        ArmorSet::default()
    } else {
        parse_armor_set(&wiki)
            .await
            .map_err(|e| format!("parse {wiki_path}: {e}"))?
    };

    let mut fmg = HashMap::new();
    for path in [DLC_FMG_PATH, xml_path] {
        if Path::new(path).exists() {
            fmg.extend(load_fmg(path)?);
        }
    }
    if fmg.is_empty() {
        return Err(format!(
            "no FMG file found, tried {xml_path} and {DLC_FMG_PATH}"
        ));
    }

    let piece_ids = ids.piece_ids();
    if set.pieces.len() > piece_ids.len() {
        return Err(format!(
            "expected at most {} pieces in wikitext ({wiki_path}), found {}",
            piece_ids.len(),
            set.pieces.len()
        ));
    }
    set.pieces.resize_with(piece_ids.len(), ArmorPiece::default);

    for ((piece, id), name) in set.pieces.iter_mut().zip(piece_ids).zip(ids.piece_names()) {
        let id = u32::try_from(id).map_err(|e| format!("bad piece id {id}: {e}"))?;
        apply_fmg(piece, &fmg, id)?;
        piece.name = name.to_owned();
    }

    Ok(set)
}

fn lookup_pieces<'a>(
    id_lookup: &'a ProtectorLookup,
    ids: &ArmorSetIDs,
) -> Result<Vec<&'a EquipParamProtector>, String> {
    ids.piece_ids()
        .into_iter()
        .map(|id| {
            id_lookup
                .get(&id)
                .ok_or_else(|| format!("no EquipParamProtector with id {id}"))
        })
        .collect()
}
