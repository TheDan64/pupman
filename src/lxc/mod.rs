pub mod config;
pub mod section;
pub mod section_mut;

use crate::linux::zfs_volume_to_mountpoint;

use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::eyre;

use std::path::PathBuf;

#[cfg(test)]
const SAMPLE_CONFIG: &'static str = r#"arch: amd64
cores: 1
features: nesting=1
hostname: trash-pandas
memory: 1024
net0: name=eth0,bridge=vmbr0,firewall=1,gw=192.168.1.1,hwaddr=AD:24:14:45:A8:38,ip=192.168.1.42/24,type=veth
ostype: debian
parent: pre-setup
rootfs: local-zfs:subvol-100-disk-0,size=4G
swap: 512
tags: unprivileged
unprivileged: 1
lxc.idmap: u 0 6653600 65536
lxc.idmap: g 0 6653600 65536

[pre-setup]
arch: amd64
cores: 1
features: nesting=1
hostname: trash-pandas
memory: 1024
net0: name=eth0,bridge=vmbr0,firewall=1,gw=192.168.1.1,hwaddr=AD:24:14:45:A8:38,ip=192.168.1.42/24,type=veth
ostype: debian
rootfs: local-zfs:subvol-100-disk-0,size=4G
snaptime: 1764532648
swap: 512
unprivileged: 1
lxc.idmap: u 0 1000 3000
lxc.idmap: g 0 1000 3000"#;

pub fn rootfs_value_to_path(value: &str) -> color_eyre::Result<PathBuf> {
    let (storage_id, volume_id) = parse_rootfs_value(value).wrap_err("invalid rootfs value")?;

    match storage_id {
        "local-zfs" => {
            let Some(path) = zfs_volume_to_mountpoint(volume_id)? else {
                return Err(eyre!("failed to find zfs mountpoint for {volume_id}"));
            };
            Ok(path)
        },
        _ => Err(eyre!("unsupported storage id {storage_id}")),
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
