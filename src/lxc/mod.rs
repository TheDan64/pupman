pub mod config;
pub mod section;
pub mod section_mut;

use crate::linux::zfs_volume_to_mountpoint;

use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::eyre;

use std::path::PathBuf;

pub fn rootfs_value_to_path(value: &str) -> color_eyre::Result<PathBuf> {
    let (storage_id, volume_id) = parse_rootfs_value(value).wrap_err("invalid rootfs value")?;

    match storage_id {
        "local-zfs" => {
            let Some(path) = zfs_volume_to_mountpoint(volume_id)? else {
                return Err(eyre!("failed to find zfs mountpoint for {volume_id}"));
            };
            Ok(path)
        },
        _ => {
            return Err(eyre!("unsupported storage id {storage_id}"));
        },
    }
}

fn parse_rootfs_value(value: &str) -> Option<(&str, &str)> {
    let mut iter = value.split(':');
    let storage_id = iter.next()?;
    let rest = iter.next()?;
    let volume_id = rest.split(',').next()?;

    Some((storage_id, volume_id))
}

#[test]
fn test_parse_rootfs_value() {
    assert_eq!(
        parse_rootfs_value("local-zfs:subvol-100-disk-0,size=4G"),
        Some(("local-zfs", "subvol-100-disk-0"))
    );
    assert_eq!(
        parse_rootfs_value("local-zfs:subvol-100-disk-0"),
        Some(("local-zfs", "subvol-100-disk-0"))
    );
    assert_eq!(parse_rootfs_value("local-zfs"), None);
}
