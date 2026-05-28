use anyhow::anyhow;
use env_logger::{Builder, Env};
use std::io::{Read, Seek, Write};
use usb_storage::fatfs;

fn main() -> anyhow::Result<()> {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let fs = usb_storage::mount()?;
    let root = fs.root_dir();

    println!("/");

    let mut counts = (0usize, 0usize); // (dirs, files)
    print_tree(&root, "", &mut counts)?;

    println!("\n{} directories, {} files", counts.0, counts.1);

    Ok(())
}

/// Recursive file tree traversal
fn print_tree<IO: Read + Write + Seek>(
    dir: &fatfs::Dir<IO>,
    prefix: &str,
    counts: &mut (usize, usize),
) -> anyhow::Result<()> {
    let entries: Vec<_> = dir
        .iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            name != "." && name != ".."
        })
        .collect();

    let total = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i + 1 == total;
        let connector = if is_last { "└── " } else { "├── " };
        let name = entry.file_name();

        if entry.is_dir() {
            println!("{}{}{}/", prefix, connector, name);
            counts.0 += 1;

            let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            let child_dir = dir
                .open_dir(&name)
                .map_err(|e| anyhow!("Failed to open subdirectory '{}': {}", name, e))?;

            print_tree(&child_dir, &child_prefix, counts)?;
        } else {
            println!("{}{}{}", prefix, connector, name);
            counts.1 += 1;
        }
    }

    Ok(())
}
