use std::path::{Path, PathBuf};

use color_eyre::eyre::eyre;

const PVE_CONF_DIR: &str = "/etc/pve/lxc";

#[derive(Clone, Debug, Default)]
pub struct Metadata {
    pub lxc_config_dir: PathBuf,
}

impl Metadata {
    pub fn collect(lxc_config_dir: Option<PathBuf>) -> color_eyre::Result<Self> {
        let lxc_config_dir = if let Some(lxc_config_dir) = lxc_config_dir {
            lxc_config_dir
        } else if Path::new(PVE_CONF_DIR).exists() {
            PathBuf::from(PVE_CONF_DIR)
        } else {
            return Err(eyre!(
                "LXC configuration directory not found. Please specify a custom directory with the -c option."
            ));
        };

        Ok(Metadata { lxc_config_dir })
    }
}
