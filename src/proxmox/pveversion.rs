use std::str::{self, FromStr};

use color_eyre::eyre::{Context, eyre};

/// Represents the version of Proxmox VE (PVE) manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PVEVersion {
    pub major: u8,
}

impl PVEVersion {
    /// Finds the PVE version by executing the `pveversion` command.
    pub fn find() -> color_eyre::Result<Self> {
        let output = std::process::Command::new("pveversion")
            .arg("-v")
            .output()
            .wrap_err("Failed to execute pveversion bin")?;

        if !output.status.success() {
            return Err(eyre!("pveversion command failed"));
        }

        Self::from_str(str::from_utf8(&output.stdout)?)
    }
}

impl FromStr for PVEVersion {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> color_eyre::Result<Self> {
        let mut parts = s.split('/');
        let Some("pve-manager") = parts.next() else {
            return Err(eyre!("Invalid PVE manager title"));
        };
        let Some(version) = parts.next() else {
            return Err(eyre!("Missing PVE manager version"));
        };
        let mut version_parts = version.split(&['-', '.']);
        let Some(major) = version_parts.next() else {
            return Err(eyre!("Invalid PVE manager version major"));
        };

        Ok(PVEVersion { major: major.parse()? })
    }
}

#[test]
fn test_pveversion_parser() -> color_eyre::Result<()> {
    let s = "pve-manager/8.1-2/ab0d52b8 (running kernel: 6.5.11-4-pve)";
    let result = PVEVersion::from_str(s)?;

    assert_eq!(result.major, 8);

    Ok(())
}
