use color_eyre::eyre::{Context, eyre};
use pupman::app::App;
use pupman::proxmox::pveversion::PVEVersion;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let ver = PVEVersion::find().wrap_err("Failed to determine pve version")?;

    if ver.major != 8 {
        return Err(eyre!("Unsupported PVE version: {}", ver.major));
    }

    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
