use super::model::ArmorSet;
use crate::models::common::equip_param_protector::EquipParamProtector;

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Per-kg values use the unrounded total weight; weight is rounded last.
pub fn apply_stats(set: &mut ArmorSet, pieces: &[&EquipParamProtector]) {
    let weight: f64 = pieces.iter().map(|p| p.weight).sum();

    // Absorption scales multiplicatively
    let absorption = |get: fn(&EquipParamProtector) -> f64| {
        let pct = (1.0 - pieces.iter().map(|p| get(p)).product::<f64>()) * 100.0;
        (round2(pct), round2(pct / weight))
    };

    let sum = |get: fn(&EquipParamProtector) -> i64| pieces.iter().map(|p| get(p)).sum();

    let poise = pieces.iter().map(|p| p.toughness_correct_rate).sum::<f64>() * 1000.0;
    (
        set.toughness_correct_rate,
        set.toughness_correct_rate_per_kg,
    ) = (round2(poise), round2(poise / weight));

    (
        set.neutral_damage_cut_rate,
        set.neutral_damage_cut_rate_per_kg,
    ) = absorption(|p| p.neutral_damage_cut_rate);

    (set.slash_damage_cut_rate, set.slash_damage_cut_rate_per_kg) =
        absorption(|p| p.slash_damage_cut_rate);

    (set.blow_damage_cut_rate, set.blow_damage_cut_rate_per_kg) =
        absorption(|p| p.blow_damage_cut_rate);

    (
        set.thrust_damage_cut_rate,
        set.thrust_damage_cut_rate_per_kg,
    ) = absorption(|p| p.thrust_damage_cut_rate);

    (set.magic_damage_cut_rate, set.magic_damage_cut_rate_per_kg) =
        absorption(|p| p.magic_damage_cut_rate);

    (set.fire_damage_cut_rate, set.fire_damage_cut_rate_per_kg) =
        absorption(|p| p.fire_damage_cut_rate);

    (
        set.thunder_damage_cut_rate,
        set.thunder_damage_cut_rate_per_kg,
    ) = absorption(|p| p.thunder_damage_cut_rate);

    (set.dark_damage_cut_rate, set.dark_damage_cut_rate_per_kg) =
        absorption(|p| p.dark_damage_cut_rate);

    set.resist_sleep = sum(|p| p.resist_sleep);
    set.resist_madness = sum(|p| p.resist_madness);
    set.resist_freeze = sum(|p| p.resist_freeze);
    set.resist_poison = sum(|p| p.resist_poison);
    set.resist_disease = sum(|p| p.resist_disease);
    set.resist_blood = sum(|p| p.resist_blood);
    set.resist_curse = sum(|p| p.resist_curse);

    set.weight = round2(weight);
}
