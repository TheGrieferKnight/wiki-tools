use std::collections::HashSet;
use std::fmt;
use std::fmt::Write;

use crate::csv_io::{find_atk_param_pc, find_bullet, find_magic};
use crate::error::AppError;
use crate::graph_output::{escape_dot, write_bullet_text, write_dot_edge};
use crate::models::common::{atk_param_pc::AtkParamPc, bullet::Bullet, magic::Magic};
use crate::records::SearchField;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulletRelation {
    MagicReference { reference_index: usize },
    IntervalEmitter,
    HitBullet,
}

#[derive(Debug, Clone)]
pub struct AtkParamNode {
    pub id: i64,
    pub value: AtkParamPc,
}

#[derive(Debug, Clone)]
pub struct BulletNode {
    pub id: i64,
    pub value: Option<Bullet>,
    pub atk_param: Option<AtkParamNode>,
    pub cons_type: Option<i64>,
    pub children: Vec<BulletEdge>,
    pub is_cycle: bool,
}

#[derive(Debug, Clone)]
pub struct BulletEdge {
    pub relation: BulletRelation,
    pub child: BulletNode,
}

#[derive(Debug, Clone)]
pub struct MagicGraph {
    pub magic: Magic,
    pub bullets: Vec<BulletEdge>,
}

impl BulletNode {
    fn cycle(id: i64) -> Self {
        Self {
            id,
            value: None,
            atk_param: None,
            cons_type: None,
            children: Vec::new(),
            is_cycle: true,
        }
    }
}

impl BulletRelation {
    pub fn label(self) -> String {
        match self {
            Self::MagicReference { reference_index } => {
                format!("magic ref_id{reference_index}")
            }
            Self::IntervalEmitter => "emits".to_owned(),
            Self::HitBullet => "on hit creates".to_owned(),
        }
    }
}

impl MagicGraph {
    pub fn build(magic_id: i64) -> Result<Self, AppError> {
        let magic = find_magic(&SearchField::ID(magic_id))?;

        let mut bullets = Vec::new();

        for (reference_index, bullet_id, consumption_type) in magic_bullet_references(&magic) {
            let mut path = HashSet::new();

            let bullet = build_bullet(bullet_id, Some(consumption_type), &mut path)?;

            bullets.push(BulletEdge {
                relation: BulletRelation::MagicReference { reference_index },
                child: bullet,
            });
        }

        Ok(Self { magic, bullets })
    }

    pub fn to_dot(&self) -> String {
        let mut output = String::new();

        writeln!(&mut output, "digraph magic_graph {{").unwrap();
        writeln!(&mut output, "    rankdir=LR;").unwrap();
        writeln!(&mut output, "    node [shape=box, fontname=\"monospace\"];").unwrap();

        let magic_node_id = format!("magic_{}", self.magic.id);
        let magic = &self.magic;

        let mut magic_label = format!(
            "Magic {}\\n{}\\n\
             FP Default: {}\\n\
             Stamina Default: {}\\n\
             FP Charged: {}\\n\
             Stamina Charged: {}\\n\
             ARC: {}\\n\
             INT: {}\\n\
             FTH: {}",
            magic.id,
            escape_dot(&magic.name),
            magic.mp,
            magic.stamina,
            magic.mp_charge,
            magic.stamina_charge,
            magic.requirement_luck,
            magic.requirement_intellect,
            magic.requirement_faith,
        );

        if magic.consume_loop_mp_for_menu != -1 {
            write!(
                magic_label,
                "\\nFP Loop Menu: {}",
                magic.consume_loop_mp_for_menu,
            )
            .unwrap();
        }

        writeln!(
            &mut output,
            "    {magic_node_id} \
             [label=\"{magic_label}\", shape=oval];"
        )
        .unwrap();

        let mut node_counter = 0;

        for edge in &self.bullets {
            write_dot_edge(&mut output, &magic_node_id, edge, &mut node_counter);
        }

        writeln!(&mut output, "}}").unwrap();

        output
    }
}

impl fmt::Display for MagicGraph {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let magic = &self.magic;

        writeln!(formatter, "Magic {} ({})", magic.id, magic.name,)?;

        writeln!(formatter, "  FP Consumption - Default: {}", magic.mp,)?;

        writeln!(
            formatter,
            "  Stamina Consumption - Default: {}",
            magic.stamina,
        )?;

        writeln!(formatter, "  FP Consumption - Charged: {}", magic.mp_charge,)?;

        writeln!(
            formatter,
            "  Stamina Consumption - Charged: {}",
            magic.stamina_charge,
        )?;

        if magic.consume_loop_mp_for_menu != -1 {
            writeln!(
                formatter,
                "  FP Consumption Loop - For Menu Display: {}",
                magic.consume_loop_mp_for_menu,
            )?;
        }

        writeln!(formatter, "  Requirement: ARC ({})", magic.requirement_luck,)?;

        writeln!(
            formatter,
            "  Requirement: INT ({})",
            magic.requirement_intellect,
        )?;

        writeln!(
            formatter,
            "  Requirement: FTH ({})",
            magic.requirement_faith,
        )?;

        for edge in &self.bullets {
            write_bullet_text(formatter, edge, 1)?;
        }

        Ok(())
    }
}

fn magic_bullet_references(magic: &Magic) -> Vec<(usize, i64, i64)> {
    let categories = [
        magic.ref_category1,
        magic.ref_category2,
        magic.ref_category3,
        magic.ref_category4,
        magic.ref_category5,
        magic.ref_category6,
        magic.ref_category7,
        magic.ref_category8,
        magic.ref_category9,
        magic.ref_category10,
    ];

    let ids = [
        magic.ref_id1,
        magic.ref_id2,
        magic.ref_id3,
        magic.ref_id4,
        magic.ref_id5,
        magic.ref_id6,
        magic.ref_id7,
        magic.ref_id8,
        magic.ref_id9,
        magic.ref_id10,
    ];

    let consumption_types = [
        magic.consume_type1,
        magic.consume_type2,
        magic.consume_type3,
        magic.consume_type4,
        magic.consume_type5,
        magic.consume_type6,
        magic.consume_type7,
        magic.consume_type8,
        magic.consume_type9,
        magic.consume_type10,
    ];

    categories
        .into_iter()
        .zip(ids)
        .zip(consumption_types)
        .enumerate()
        .filter_map(|(index, ((category, id), consumption_type))| {
            (category == 1).then_some((index + 1, id, consumption_type))
        })
        .collect()
}

fn build_bullet(
    bullet_id: i64,
    consumption_type: Option<i64>,
    path: &mut HashSet<i64>,
) -> Result<BulletNode, AppError> {
    if !path.insert(bullet_id) {
        return Ok(BulletNode::cycle(bullet_id));
    }

    let result = build_bullet_inner(bullet_id, consumption_type, path);

    path.remove(&bullet_id);

    result
}

fn build_bullet_inner(
    bullet_id: i64,
    consumption_type: Option<i64>,
    path: &mut HashSet<i64>,
) -> Result<BulletNode, AppError> {
    let bullet = find_bullet(&SearchField::ID(bullet_id))?;

    let atk_param = if bullet.atk_id_bullet <= 1 {
        None
    } else {
        let atk_param = find_atk_param_pc(&SearchField::ID(bullet.atk_id_bullet))?;

        Some(AtkParamNode {
            id: atk_param.id,
            value: atk_param,
        })
    };

    let mut node = BulletNode {
        id: bullet.id,
        value: Some(bullet.clone()),
        atk_param,
        cons_type: consumption_type,
        children: Vec::new(),
        is_cycle: false,
    };

    let interval_create_bullet_id = bullet.interval_create_bullet_id;
    let hit_bullet_id = bullet.hit_bullet_id;

    drop(bullet); // Bullet is dropped here, before recursion as to save a lot of memory

    if interval_create_bullet_id >= 0 {
        let child = build_bullet(interval_create_bullet_id, None, path)?;

        node.children.push(BulletEdge {
            relation: BulletRelation::IntervalEmitter,
            child,
        });
    }

    if
    /*bullet.launch_condition_type == 0 &&*/
    hit_bullet_id >= 0 {
        let child = build_bullet(hit_bullet_id, None, path)?;

        node.children.push(BulletEdge {
            relation: BulletRelation::HitBullet,
            child,
        });
    }

    Ok(node)
}
