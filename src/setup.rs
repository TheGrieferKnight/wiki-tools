use std::path::Path;

use futures::{StreamExt, TryStreamExt, stream};
use tokio::fs;

use crate::constants::{COMMON, GENERAL, GRAPHICS, SOUND};

async fn move_category(
    source_dir: &Path,
    destination_root: &Path,
    folder: &str,
    files: &[&str],
) -> std::io::Result<()> {
    let destination_dir = destination_root.join(folder);

    fs::create_dir_all(&destination_dir).await?;

    let moves = files.iter().map(|&filename| {
        let source = source_dir.join(filename);
        let destination = destination_dir.join(filename);

        async move {
            let source_string = fs::read_to_string(&source).await?;

            let mut lines = source_string.lines();

            let header = lines.next().unwrap_or("");
            let header = header.strip_suffix(',').unwrap_or(header);

            let cleaned = std::iter::once(header)
                .chain(lines)
                .collect::<Vec<_>>()
                .join("\n");

            match fs::write(&destination, cleaned).await {
                Ok(()) => {
                    fs::remove_file(source).await?;
                    Ok(())
                }
                Err(error) => {
                    eprintln!(
                        "Failed to write '{}' to '{}': {}",
                        source.display(),
                        destination.display(),
                        error
                    );

                    Err(error)
                }
            }
        }
    });

    stream::iter(moves)
        .buffer_unordered(8)
        .try_collect::<Vec<_>>()
        .await?;

    Ok(())
}

async fn directory_is_empty(path: &Path) -> std::io::Result<bool> {
    let mut entries = fs::read_dir(path).await?;
    Ok(entries.next_entry().await?.is_none())
}

pub async fn setup_csv() -> std::io::Result<()> {
    let source_dir = Path::new("./exported-csv");
    let destination_root = Path::new("./csv");

    if !directory_is_empty(source_dir).await? {
        move_category(source_dir, destination_root, "common", COMMON).await?;
        move_category(source_dir, destination_root, "general", GENERAL).await?;
        move_category(source_dir, destination_root, "graphics", GRAPHICS).await?;
        move_category(source_dir, destination_root, "sound", SOUND).await?;
        println!("Completed setup successfully.");
    } else {
        println!(
            "Skipping setup, as there are no files in {}",
            source_dir.to_string_lossy()
        );
    }
    Ok(())
}
