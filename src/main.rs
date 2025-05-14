use color_eyre::eyre::Context;
use pupman::{app::App, proxmox::pveversion::PVEVersion};

fn main() -> color_eyre::Result<()> {
    let _ver = PVEVersion::find().wrap_err("failed to get pve version")?;

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
