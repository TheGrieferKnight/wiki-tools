mod piece;
mod xml;

use super::{ActiveEffect, ArmorPiece};

pub use piece::{FmgPiece, apply_fmg, parse_fmg_piece};
pub use xml::{FmgMap, load_fmg, parse_fmg_xml};
