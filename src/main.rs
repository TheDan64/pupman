use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::{Context, eyre};
use lxcidman::app::App;
use lxcidman::proxmox::lxc;
use lxcidman::proxmox::pveversion::PVEVersion;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom lxc config directory
    #[arg(short = 'c', long, value_name = "DIR", default_value_os = lxc::CONF_DIR)]
    lxc_config: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let ver = PVEVersion::find().wrap_err("Failed to determine pve version")?;

    if ver.major != 8 {
        return Err(eyre!("Unsupported PVE version: {}", ver.major));
    }

    let terminal = ratatui::init();
    let result = App::new(&cli.lxc_config).run(terminal);
    ratatui::restore();
    result
}
