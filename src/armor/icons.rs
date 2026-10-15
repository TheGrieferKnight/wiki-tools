use std::path::Path;

use tokio::process::Command;

use super::{general, wiki_api::Wiki};
use crate::{AppError, models::common::equip_param_protector::EquipParamProtector};

const WITCHY_BND: &str = "/home/thegrieferknight/Downloads/WitchyBND/WitchyBND";
const TPF_DIR: &str = "/home/thegrieferknight/Downloads/WitchyBND/00_solo-tpfbdt";

/// Icons that already exist on the wiki and are shared by several pieces, so
/// re-uploading them causes errors.
const PREEXISTING_ICONS: [u32; 5] = [15717, 13090, 45930, 13603, 15413];

/// Extracts the piece`s menu icon, uploads it, and returns the wiki filename
pub async fn ensure_icon(wiki: &Wiki, param: &EquipParamProtector) -> Result<String, AppError> {
    let icon_id = param.icon_id_m;
    let filename = format!("MENU_Knowledge_{icon_id}.png");

    if PREEXISTING_ICONS
        .iter()
        .any(|&id| id as i64 == icon_id as i64)
    {
        return Ok(filename);
    }

    let tpf_path = format!("{TPF_DIR}/MENU_Knowledge_{icon_id}.tpf.dcx");
    // WitchyBND unpacks into same directory, `<name>-tpf-dcx/`
    let dds_path =
        format!("{TPF_DIR}/MENU_Knowledge_{icon_id}-tpf-dcx/MENU_Knowledge_{icon_id}.dds");
    println!("{tpf_path}");
    println!("{dds_path}");

    let output = Command::new(WITCHY_BND)
        .arg(&tpf_path)
        .output()
        .await
        .map_err(|e| {
            general(format!(
                "failed to run WitchyBND for {}: {e}; kind={:?}; os_error={:?}; cwd={:?}",
                param.name,
                e.kind(),
                e.raw_os_error(),
                std::env::current_dir(),
            ))
        })?;

    // WitchyBND exit code doesn't work properly, check output instead
    if !Path::new(&dds_path).exists() {
        return Err(general(format!(
            "{}: expected {dds_path} after WitchyBND (status {}): {}",
            param.name,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let png_path = crate::dds::dds_to_png(dds_path).map_err(|e| general(e.to_string()))?;

    wiki.upload_file(
        &filename,
        format!("Icon upload for current armor piece: {}", param.name),
        png_path,
    )
    .await?;

    Ok(filename)
}
