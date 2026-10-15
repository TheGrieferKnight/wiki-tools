use std::fmt::{Display, Write as _};

use super::model::{ArmorPiece, ArmorSet};

/// Parameter prefixes used by {{ArmorSet}}, in piece order.
const PIECE_PREFIXES: [&str; 4] = ["helm", "armor", "gauntlets", "greaves"];

struct Page(String);

impl Page {
    fn field(&mut self, key: &str, value: impl Display) {
        writeln!(self.0, "    |{key} = {value}").unwrap();
    }

    fn blank(&mut self) {
        self.0.push('\n');
    }
}

pub fn render(set_name: &str, set: &ArmorSet) -> String {
    let mut p = Page(String::from("{{ArmorSet\n"));

    p.field("name", set_name);
    p.field("image", &set.image);
    p.field("class", &set.class);
    p.field("tier", &set.tier);
    p.blank();
    p.field("weight", set.weight);
    p.blank();

    let rates = [
        (
            "toughness_correct_rate",
            set.toughness_correct_rate,
            set.toughness_correct_rate_per_kg,
        ),
        (
            "neutral_damage_cut_rate",
            set.neutral_damage_cut_rate,
            set.neutral_damage_cut_rate_per_kg,
        ),
        (
            "slash_damage_cut_rate",
            set.slash_damage_cut_rate,
            set.slash_damage_cut_rate_per_kg,
        ),
        (
            "blow_damage_cut_rate",
            set.blow_damage_cut_rate,
            set.blow_damage_cut_rate_per_kg,
        ),
        (
            "thrust_damage_cut_rate",
            set.thrust_damage_cut_rate,
            set.thrust_damage_cut_rate_per_kg,
        ),
        (
            "magic_damage_cut_rate",
            set.magic_damage_cut_rate,
            set.magic_damage_cut_rate_per_kg,
        ),
        (
            "fire_damage_cut_rate",
            set.fire_damage_cut_rate,
            set.fire_damage_cut_rate_per_kg,
        ),
        (
            "thunder_damage_cut_rate",
            set.thunder_damage_cut_rate,
            set.thunder_damage_cut_rate_per_kg,
        ),
        (
            "dark_damage_cut_rate",
            set.dark_damage_cut_rate,
            set.dark_damage_cut_rate_per_kg,
        ),
    ];
    for (key, total, _) in rates {
        p.field(key, total);
    }
    p.blank();
    for (key, _, per_kg) in rates {
        p.field(&format!("{key}_per_kg"), per_kg);
    }
    p.blank();

    p.field("resist_sleep", set.resist_sleep);
    p.field("resist_madness", set.resist_madness);
    p.field("resist_freeze", set.resist_freeze);
    p.field("resist_poison", set.resist_poison);
    p.field("resist_disease", set.resist_disease);
    p.field("resist_blood", set.resist_blood);
    p.field("resist_curse", set.resist_curse);
    p.blank();

    p.field("introduction", &set.description);
    p.blank();

    for (prefix, piece) in PIECE_PREFIXES.into_iter().zip(&set.pieces) {
        render_piece(&mut p, prefix, piece);
        p.blank();
    }

    p.field(
        "history",
        set.history.first().map_or("", |h| h.text.as_str()),
    );
    p.0.push_str("    }}");
    p.0
}

fn render_piece(p: &mut Page, prefix: &str, piece: &ArmorPiece) {
    let nth = |v: &[String], i: usize| v.get(i).cloned().unwrap_or_default();
    let active = piece.active.clone().unwrap_or_default();

    p.field(&format!("{prefix}_name"), &piece.name);
    p.field(&format!("{prefix}_image"), nth(&piece.icons, 0));
    p.field(&format!("{prefix}_alt_image"), nth(&piece.icons, 1));
    p.field(&format!("{prefix}_flavor"), &piece.flavor);
    p.field(&format!("{prefix}_description"), &piece.description);
    p.field(
        &format!("{prefix}_passive"),
        piece.passive.as_deref().unwrap_or_default(),
    );
    p.field(&format!("{prefix}_active_name"), &active.name);
    p.field(&format!("{prefix}_active"), &active.effect);
    p.field(&format!("{prefix}_obtain"), &piece.obtain_location);
    p.field(
        &format!("{prefix}_location_images"),
        format!("[[File:{}]]", nth(&piece.obtain_images, 0)),
    );
}
