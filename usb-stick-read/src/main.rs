use anyhow::Context;
use env_logger::{Builder, Env};
use std::io::Read;

fn main() -> anyhow::Result<()> {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let path = "policy.toml";

    let fs = usb_storage::mount()?;
    let root = fs.root_dir();

    let mut file = root
        .open_file(path)
        .map_err(|e| anyhow::anyhow!("Failed to open '{}': {}", path, e))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .context("Failed to read file")?;

    print!("{}", contents);

    Ok(())
}
