use std::path::{Path, PathBuf};

use color_eyre::eyre::eyre;

const PVE_CONF_DIR: &str = "/etc/pve/lxc";

#[derive(Clone, Debug)]
pub struct Metadata {
    pub lxc_config_dir: PathBuf,
    pub is_pve: bool,
}

impl Metadata {
    pub fn collect(lxc_config_dir: Option<PathBuf>) -> color_eyre::Result<Self> {
        let is_pve = Path::new(PVE_CONF_DIR).exists();
        let lxc_config_dir = if let Some(lxc_config_dir) = lxc_config_dir {
            lxc_config_dir
        } else if is_pve {
            PathBuf::from(PVE_CONF_DIR)
        } else {
            return Err(eyre!(
                "LXC configuration directory not found. Please specify a custom directory with the -c option."
            ));
        };

        Ok(Metadata { lxc_config_dir, is_pve })
    }
}
