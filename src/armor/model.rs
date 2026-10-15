use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArmorSet {
    #[serde(rename = "Name")]
    pub name: String,
    pub image: String,
    pub class: String,
    pub tier: String,
    pub description: String,
    pub pieces: Vec<ArmorPiece>,
    pub history: Vec<HistoryEntry>,

    pub weight: f64,

    /// Summed poise of all pieces, scaled by 1000.
    pub toughness_correct_rate: f64,
    /// Absorption in percent: (1 - product of piece rates) * 100.
    pub neutral_damage_cut_rate: f64,
    pub slash_damage_cut_rate: f64,
    pub blow_damage_cut_rate: f64,
    pub thrust_damage_cut_rate: f64,
    pub magic_damage_cut_rate: f64,
    pub fire_damage_cut_rate: f64,
    pub thunder_damage_cut_rate: f64,
    pub dark_damage_cut_rate: f64,

    pub toughness_correct_rate_per_kg: f64,
    pub neutral_damage_cut_rate_per_kg: f64,
    pub slash_damage_cut_rate_per_kg: f64,
    pub blow_damage_cut_rate_per_kg: f64,
    pub thrust_damage_cut_rate_per_kg: f64,
    pub magic_damage_cut_rate_per_kg: f64,
    pub fire_damage_cut_rate_per_kg: f64,
    pub thunder_damage_cut_rate_per_kg: f64,
    pub dark_damage_cut_rate_per_kg: f64,

    pub resist_sleep: i64,
    pub resist_madness: i64,
    pub resist_freeze: i64,
    pub resist_poison: i64,
    pub resist_disease: i64,
    pub resist_blood: i64,
    pub resist_curse: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArmorPiece {
    pub name: String,
    /// `[normal, altered]`; either may be missing.
    pub icons: Vec<String>,
    pub flavor: String,
    pub description: String,
    pub passive: Option<String>,
    pub active: Option<ActiveEffect>,
    pub obtain_location: String,
    pub obtain_images: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub name: String,
    pub effect: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub version: String,
    pub text: String,
}
