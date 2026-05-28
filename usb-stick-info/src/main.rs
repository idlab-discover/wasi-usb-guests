use anyhow::anyhow;
use env_logger::{Builder, Env};

fn format_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;

    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn main() -> anyhow::Result<()> {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let fs = usb_storage::mount()?;

    println!("Filesystem type : {:?}", fs.fat_type());
    println!("Volume label    : {}", fs.volume_label());

    let stats = fs
        .stats()
        .map_err(|e| anyhow!("Failed to read FS stats: {}", e))?;
    let cluster_size = stats.cluster_size();
    let total = stats.total_clusters() as u64 * cluster_size as u64;
    let free = stats.free_clusters() as u64 * cluster_size as u64;

    println!("Cluster size    : {} bytes", cluster_size);
    println!("Total clusters  : {}", stats.total_clusters());
    println!("Free clusters   : {}", stats.free_clusters());
    println!("Total size      : {}", format_size(total));
    println!("Free space      : {}", format_size(free));

    Ok(())
}
