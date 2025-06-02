use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Context;
use log::{LevelFilter, info};
use lxcidman::app::App;
use lxcidman::metadata::Metadata;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom lxc config directory
    #[arg(short = 'c', long, value_name = "DIR")]
    lxc_config: Option<PathBuf>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tui_logger::init_logger(LevelFilter::Trace)?;
    tui_logger::set_default_level(LevelFilter::Trace);

    info!("Starting lxcidman...");

    let cli = Cli::parse();

    info!("Collecting system metadata...");

    let md = Metadata::collect(cli.lxc_config).wrap_err("Failed to collect system metadata")?;
    let terminal = ratatui::init();
    let result = App::new(md).run(terminal);
    ratatui::restore();
    result
}
