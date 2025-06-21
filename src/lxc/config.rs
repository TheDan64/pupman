//! A Proxmox LXC container configuration file parser, writer, and validator.
//!
//! A config should have near constant time lookups on methods since data is constantly read and
//! displayed to the user. Writes can be slower as they are infrequent operations.

use std::fmt::{Display, Write};
use std::str::FromStr;

use ahash::HashMap;
use compact_str::{CompactString, ToCompactString};

use super::section::SectionView;
use super::section_mut::SectionViewMut;

#[derive(Clone, Debug)]
pub enum ConfEntry {
    Section(CompactString),
    KeyValue(CompactString, CompactString),
    Comment(String),
    EmptyLine,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub(super) entries: Vec<ConfEntry>,
    pub(super) index: HashMap<(Option<CompactString>, CompactString), Vec<CompactString>>,
}

impl Config {
    pub fn section<'s, S>(&self, section: S) -> SectionView<'s, '_>
    where
        S: Into<Option<&'s str>>,
    {
        SectionView {
            config: self,
            section: section.into(),
        }
    }

    pub fn section_mut<'s, S>(&mut self, section: S) -> SectionViewMut<'s, '_>
    where
        S: Into<Option<&'s str>>,
    {
        SectionViewMut {
            config: self,
            section: section.into(),
        }
    }
}

impl FromStr for Config {
    type Err = color_eyre::Report;

    fn from_str(content: &str) -> color_eyre::Result<Self> {
        let lines = content.lines();
        // size_hint() is always (0, None) here; but we keep it in case future optimizations are introduced
        let mut entries = Vec::with_capacity(lines.size_hint().1.unwrap_or(0));
        let mut index: HashMap<_, Vec<_>> = HashMap::default();
        let mut current_section: Option<CompactString> = None;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                entries.push(ConfEntry::EmptyLine);
            } else if trimmed.starts_with('#') || trimmed.starts_with(';') {
                entries.push(ConfEntry::Comment(trimmed.to_string()));
            } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section = (&trimmed[1..trimmed.len() - 1]).to_compact_string();

                entries.push(ConfEntry::Section(section.clone()));
                current_section = Some(section);
            } else if let Some((key, value)) = trimmed.split_once(':').or_else(|| trimmed.split_once('=')) {
                let key = key.trim().to_compact_string();
                let value = value.trim().to_compact_string();

                entries.push(ConfEntry::KeyValue(key.clone(), value.clone()));
                index.entry((current_section.clone(), key)).or_default().push(value);
            } else {
                let key = trimmed.to_compact_string();

                entries.push(ConfEntry::KeyValue(key.clone(), CompactString::new("")));
                index
                    .entry((current_section.clone(), key))
                    .or_default()
                    .push(CompactString::new(""));
            }
        }

        Ok(Config { entries, index })
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, entry) in self.entries.iter().enumerate() {
            if i != 0 {
                f.write_char('\n')?;
            }

            match entry {
                ConfEntry::Section(section) => write!(f, "[{section}]")?,
                ConfEntry::KeyValue(key, value) => write!(f, "{key}: {value}")?,
                ConfEntry::Comment(comment) => write!(f, "{comment}")?,
                ConfEntry::EmptyLine => {},
            }
        }

        Ok(())
    }
}

#[test]
fn test_config_to_from_str() -> color_eyre::Result<()> {
    let content = r#"arch: amd64
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

    let config = Config::from_str(content)?;

    assert_eq!(config.entries.len(), 29);
    assert!(matches!(&config.entries[0], ConfEntry::KeyValue(key, value) if key == "arch" && value == "amd64"));
    assert!(matches!(&config.entries[1], ConfEntry::KeyValue(key, value) if key == "cores" && value == "1"));
    assert!(matches!(&config.entries[2], ConfEntry::KeyValue(key, value) if key == "features" && value == "nesting=1"));
    assert!(
        matches!(&config.entries[3], ConfEntry::KeyValue(key, value) if key == "hostname" && value == "trash-pandas")
    );
    assert!(matches!(&config.entries[4], ConfEntry::KeyValue(key, value) if key == "memory" && value == "1024"));
    assert!(
        matches!(&config.entries[5], ConfEntry::KeyValue(key, value) if key == "net0" && value == "name=eth0,bridge=vmbr0,firewall=1,gw=192.168.1.1,hwaddr=AD:24:14:45:A8:38,ip=192.168.1.42/24,type=veth")
    );
    assert!(matches!(&config.entries[6], ConfEntry::KeyValue(key, value) if key == "ostype" && value == "debian"));
    assert!(matches!(&config.entries[7], ConfEntry::KeyValue(key, value) if key == "parent" && value == "pre-setup"));
    assert!(
        matches!(&config.entries[8], ConfEntry::KeyValue(key, value) if key == "rootfs" && value == "local-zfs:subvol-100-disk-0,size=4G")
    );
    assert!(matches!(&config.entries[9], ConfEntry::KeyValue(key, value) if key == "swap" && value == "512"));
    assert!(matches!(&config.entries[10], ConfEntry::KeyValue(key, value) if key == "tags" && value == "unprivileged"));
    assert!(matches!(&config.entries[11], ConfEntry::KeyValue(key, value) if key == "unprivileged" && value == "1"));
    assert!(
        matches!(&config.entries[12], ConfEntry::KeyValue(key, value) if key == "lxc.idmap" && value == "u 0 6653600 65536")
    );
    assert!(
        matches!(&config.entries[13], ConfEntry::KeyValue(key, value) if key == "lxc.idmap" && value == "g 0 6653600 65536")
    );
    assert!(matches!(&config.entries[14], ConfEntry::EmptyLine));
    assert!(matches!(&config.entries[15], ConfEntry::Section(section) if section == "pre-setup"));
    assert!(matches!(&config.entries[16], ConfEntry::KeyValue(key, value) if key == "arch" && value == "amd64"));
    assert!(matches!(&config.entries[17], ConfEntry::KeyValue(key, value) if key == "cores" && value == "1"));
    assert!(
        matches!(&config.entries[18], ConfEntry::KeyValue(key, value) if key == "features" && value == "nesting=1")
    );
    assert!(
        matches!(&config.entries[19], ConfEntry::KeyValue(key, value) if key == "hostname" && value == "trash-pandas")
    );
    assert!(matches!(&config.entries[20], ConfEntry::KeyValue(key, value) if key == "memory" && value == "1024"));
    assert!(
        matches!(&config.entries[21], ConfEntry::KeyValue(key, value) if key == "net0" && value == "name=eth0,bridge=vmbr0,firewall=1,gw=192.168.1.1,hwaddr=AD:24:14:45:A8:38,ip=192.168.1.42/24,type=veth")
    );
    assert!(matches!(&config.entries[22], ConfEntry::KeyValue(key, value) if key == "ostype" && value == "debian"));
    assert!(
        matches!(&config.entries[23], ConfEntry::KeyValue(key, value) if key == "rootfs" && value == "local-zfs:subvol-100-disk-0,size=4G")
    );
    assert!(
        matches!(&config.entries[24], ConfEntry::KeyValue(key, value) if key == "snaptime" && value == "1764532648")
    );
    assert!(matches!(&config.entries[25], ConfEntry::KeyValue(key, value) if key == "swap" && value == "512"));
    assert!(matches!(&config.entries[26], ConfEntry::KeyValue(key, value) if key == "unprivileged" && value == "1"));
    assert!(
        matches!(&config.entries[27], ConfEntry::KeyValue(key, value) if key == "lxc.idmap" && value == "u 0 1000 3000")
    );
    assert!(
        matches!(&config.entries[28], ConfEntry::KeyValue(key, value) if key == "lxc.idmap" && value == "g 0 1000 3000")
    );

    let section = config.section(None);
    let idmaps = section.get_all("lxc.idmap").collect::<Vec<_>>();

    assert_eq!(idmaps.len(), 2);
    assert_eq!(idmaps[0], "u 0 6653600 65536");
    assert_eq!(idmaps[1], "g 0 6653600 65536");

    assert_eq!(config.to_string(), content);

    Ok(())
}
