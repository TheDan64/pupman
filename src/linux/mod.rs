use std::process::Command;
use std::str;
use std::{path::PathBuf, process::Output};

use color_eyre::eyre::{Context, eyre};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LinuxError {
    #[error("Linux command failed with status code and output: {0}, {1:?}, {2:?}")]
    Command(std::process::ExitStatus, String, String),
    #[error("IO failed with error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Failed to convert string to utf-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

impl From<Output> for LinuxError {
    fn from(value: Output) -> Self {
        Self::Command(
            value.status,
            String::from_utf8_lossy(&value.stdout).into_owned(),
            String::from_utf8_lossy(&value.stderr).into_owned(),
        )
    }
}

pub fn username_to_id(username: &str) -> color_eyre::Result<u32> {
    let output = Command::new("id")
        .arg("-u")
        .arg(username)
        .output()
        .wrap_err("Failed to execute id bin")?;

    if !output.status.success() {
        return Err(eyre!("id command failed"));
    }

    let id_str = std::str::from_utf8(&output.stdout).wrap_err("Failed to parse id output")?;
    id_str.trim().parse().wrap_err("Failed to parse user ID")
}

pub fn groupname_to_id(groupname: &str) -> color_eyre::Result<u32> {
    let output = Command::new("id")
        .arg("-g")
        .arg(groupname)
        .output()
        .wrap_err("Failed to execute id bin")?;

    if !output.status.success() {
        return Err(eyre!("id command failed"));
    }

    let id_str = std::str::from_utf8(&output.stdout).wrap_err("Failed to parse id output")?;
    id_str.trim().parse().wrap_err("Failed to parse group ID")
}

pub fn zfs_volume_to_mountpoint(volume: &str) -> Result<Option<PathBuf>, LinuxError> {
    let output = Command::new("zfs").args(&["list", "-o", "mountpoint"]).output()?;

    if !output.status.success() {
        return Err(output.into());
    }

    let stdout = str::from_utf8(&output.stdout)?;

    for line in stdout.lines() {
        if line.trim_end().ends_with(volume) {
            return Ok(Some(PathBuf::from(line.trim_end())));
        }
    }

    Ok(None)
}

#[test]
fn test_username_to_id() {
    assert_eq!(username_to_id("root").unwrap(), 0);
}

#[test]
fn test_groupname_to_id() {
    assert_eq!(groupname_to_id("root").unwrap(), 0);
}
