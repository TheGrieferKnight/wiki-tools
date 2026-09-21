use std::fmt;
use std::fmt::Write;

use crate::graph::BulletEdge;

pub fn write_bullet_text(
    formatter: &mut fmt::Formatter<'_>,
    edge: &BulletEdge,
    depth: usize,
) -> fmt::Result {
    let indent = "  ".repeat(depth);
    let bullet = &edge.child;

    writeln!(
        formatter,
        "{indent}└─ {} → Bullet {}",
        edge.relation.label(),
        bullet.id,
    )?;

    if bullet.is_cycle {
        writeln!(formatter, "{indent}   [cycle detected]")?;
        return Ok(());
    }

    let value = bullet
        .value
        .as_ref()
        .expect("non-cycle bullet must have a value");

    let property_indent = "  ".repeat(depth + 1);

    writeln!(formatter, "{property_indent}Name: {}", value.name,)?;

    writeln!(
        formatter,
        "{property_indent}atkId_Bullet: {}",
        value.atk_id_bullet,
    )?;

    writeln!(
        formatter,
        "{property_indent}Consumption Type: {:?}",
        bullet.cons_type,
    )?;

    writeln!(
        formatter,
        "{property_indent}Damage Hit Durationb: {:?}",
        value.dmg_hit_record_life_time,
    )?;

    if value.interval_create_bullet_id != -1 {
        writeln!(
            formatter,
            "{property_indent}Interval Min: {:?}",
            value.interval_create_time_min,
        )?;

        writeln!(
            formatter,
            "{property_indent}Interval Max: {:?}",
            value.interval_create_time_max,
        )?;
    }



    if let Some(atk_param) = &bullet.atk_param {
        let atk = &atk_param.value;
        let atk_indent = "  ".repeat(depth + 1);
        let damage_indent = "  ".repeat(depth + 2);

        writeln!(
            formatter,
            "{atk_indent}└─ AtkParamPc {} ({})",
            atk.id, atk.name,
        )?;

        writeln!(
            formatter,
            "{damage_indent}Damage: Physical ({})",
            atk.atk_phys,
        )?;

        writeln!(formatter, "{damage_indent}Damage: Magic ({})", atk.atk_mag,)?;

        writeln!(formatter, "{damage_indent}Damage: Fire ({})", atk.atk_fire,)?;

        writeln!(
            formatter,
            "{damage_indent}Damage: Lightning ({})",
            atk.atk_thun,
        )?;

        writeln!(formatter, "{damage_indent}Damage: Holy ({})", atk.atk_dark,)?;

        writeln!(
            formatter,
            "{damage_indent}Damage: Stamina ({})",
            atk.atk_stam,
        )?;

        writeln!(
            formatter,
            "{damage_indent}Damage: Poise ({})",
            atk.atk_super_armor,
        )?;
    }

    writeln!(
        formatter,
        "{property_indent}launchConditionType: {}",
        value.launch_condition_type,
    )?;

    for child in &bullet.children {
        write_bullet_text(formatter, child, depth + 1)?;
    }

    Ok(())
}

pub fn write_dot_edge(
    output: &mut String,
    parent_id: &str,
    edge: &BulletEdge,
    node_counter: &mut usize,
) {
    let child = &edge.child;
    let current_id = format!("bullet_{}_{}", child.id, *node_counter);

    *node_counter += 1;

    let bullet_label = if child.is_cycle {
        format!("Bullet {}\\ncycle", child.id)
    } else {
        let value = child
            .value
            .as_ref()
            .expect("non-cycle bullet must have a value");

        if value.interval_create_bullet_id != -1 {
            format!(
                "Bullet {}\\n{}\\natkId_Bullet: {}\\nConsumption Type: {:?} \\nInterval Min: {} \\nInterval Max: {}",
                child.id,
                escape_dot(&value.name),
                value.atk_id_bullet,
                child.cons_type,
                value.interval_create_time_min,
                value.interval_create_time_max
            )
        } else {
            format!(
                "Bullet {}\\n{}\\natkId_Bullet: {}\\nConsumption Type: {:?}",
                child.id,
                escape_dot(&value.name),
                value.atk_id_bullet,
                child.cons_type
            )
        }
    };

    writeln!(
        output,
        "    {current_id} \
         [label=\"{bullet_label}\", shape=box];"
    )
    .unwrap();

    writeln!(
        output,
        "    {parent_id} -> {current_id} \
         [label=\"{}\"];",
        escape_dot(&edge.relation.label()),
    )
    .unwrap();

    if child.is_cycle {
        return;
    }

    if let Some(atk_param) = &child.atk_param {
        let atk = &atk_param.value;
        let atk_node_id = format!("{current_id}_atk");

        let atk_label = format!("AtkParamPc {}\\n{}", atk.id, escape_dot(&atk.name),);

        writeln!(
            output,
            "    {atk_node_id} \
             [label=\"{atk_label}\", shape=component];"
        )
        .unwrap();

        writeln!(
            output,
            "    {current_id} -> {atk_node_id} \
             [label=\"attack params\"];"
        )
        .unwrap();

        let damage = [
            ("Physical", atk.atk_phys.to_string()),
            ("Magic", atk.atk_mag.to_string()),
            ("Fire", atk.atk_fire.to_string()),
            ("Lightning", atk.atk_thun.to_string()),
            ("Holy", atk.atk_dark.to_string()),
            ("Stamina", atk.atk_stam.to_string()),
            ("Poise", atk.atk_super_armor.to_string()),
        ];

        let mut damage_lines = String::new();

        for (index, (damage_type, amount)) in damage.iter().enumerate() {
            if index > 0 {
                damage_lines.push_str("\\n");
            }

            write!(damage_lines, "{}: {}", damage_type, escape_dot(amount),).unwrap();
        }

        let damage_node_id = format!("{atk_node_id}_damage");

        writeln!(
            output,
            "    {damage_node_id} \
             [label=\"{damage_lines}\", shape=note];"
        )
        .unwrap();

        writeln!(
            output,
            "    {atk_node_id} -> {damage_node_id} \
             [label=\"damage\"];"
        )
        .unwrap();
    }

    for nested_edge in &child.children {
        write_dot_edge(output, &current_id, nested_edge, node_counter);
    }
}

pub fn escape_dot(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
