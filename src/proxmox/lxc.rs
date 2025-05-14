use std::fmt::Display;
use std::str::FromStr;

// const DIR: &str = "/etc/pve/lxc";
// const DIR: &str = "/tmp";

#[derive(Debug)]
pub enum ConfEntry {
    Section(String),
    KeyValue(String, String),
    Comment(String),
    EmptyLine,
}

#[derive(Debug)]
pub struct Config {
    entries: Vec<ConfEntry>,
}

impl Config {
    pub fn sectionless_idmap(&self) -> impl Iterator<Item = &ConfEntry> {
        self.entries
            .iter()
            .take_while(|entry| !matches!(entry, ConfEntry::Section(_)))
            .filter(|entry| matches!(entry, ConfEntry::KeyValue(key, _) if key == "lxc.idmap"))
    }
}

impl FromStr for Config {
    type Err = color_eyre::Report;

    fn from_str(content: &str) -> color_eyre::Result<Self> {
        let lines = content.lines();
        // size_hint() is always (0, None) here; but we keep it in case future optimizations are introduced
        let mut entries = Vec::with_capacity(lines.size_hint().1.unwrap_or(0));

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                entries.push(ConfEntry::EmptyLine);
            } else if trimmed.starts_with('#') || trimmed.starts_with(';') {
                entries.push(ConfEntry::Comment(trimmed.to_string()));
            } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section = trimmed[1..trimmed.len() - 1].to_string();
                entries.push(ConfEntry::Section(section));
            } else if let Some((key, value)) = trimmed.split_once(':') {
                entries.push(ConfEntry::KeyValue(
                    key.trim().to_string(),
                    value.trim().to_string(),
                ));
            } else if let Some((key, value)) = trimmed.split_once('=') {
                entries.push(ConfEntry::KeyValue(
                    key.trim().to_string(),
                    value.trim().to_string(),
                ));
            } else {
                entries.push(ConfEntry::KeyValue(trimmed.to_string(), String::new()));
            }
        }

        Ok(Config { entries })
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in &self.entries {
            match entry {
                ConfEntry::Section(section) => writeln!(f, "[{section}]\n")?,
                ConfEntry::KeyValue(key, value) => writeln!(f, "{key}: {value}\n")?,
                ConfEntry::Comment(comment) => writeln!(f, "{comment}\n")?,
                ConfEntry::EmptyLine => writeln!(f, "\n")?,
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
    assert!(
        matches!(&config.entries[0], ConfEntry::KeyValue(key, value) if key == "arch" && value == "amd64")
    );
    assert!(
        matches!(&config.entries[1], ConfEntry::KeyValue(key, value) if key == "cores" && value == "1")
    );
    assert!(
        matches!(&config.entries[2], ConfEntry::KeyValue(key, value) if key == "features" && value == "nesting=1")
    );
    assert!(
        matches!(&config.entries[3], ConfEntry::KeyValue(key, value) if key == "hostname" && value == "trash-pandas")
    );
    assert!(
        matches!(&config.entries[4], ConfEntry::KeyValue(key, value) if key == "memory" && value == "1024")
    );
    assert!(
        matches!(&config.entries[5], ConfEntry::KeyValue(key, value) if key == "net0" && value == "name=eth0,bridge=vmbr0,firewall=1,gw=192.168.1.1,hwaddr=AD:24:14:45:A8:38,ip=192.168.1.42/24,type=veth")
    );
    assert!(
        matches!(&config.entries[6], ConfEntry::KeyValue(key, value) if key == "ostype" && value == "debian")
    );
    assert!(
        matches!(&config.entries[7], ConfEntry::KeyValue(key, value) if key == "parent" && value == "pre-setup")
    );
    assert!(
        matches!(&config.entries[8], ConfEntry::KeyValue(key, value) if key == "rootfs" && value == "local-zfs:subvol-100-disk-0,size=4G")
    );
    assert!(
        matches!(&config.entries[9], ConfEntry::KeyValue(key, value) if key == "swap" && value == "512")
    );
    assert!(
        matches!(&config.entries[10], ConfEntry::KeyValue(key, value) if key == "tags" && value == "unprivileged")
    );
    assert!(
        matches!(&config.entries[11], ConfEntry::KeyValue(key, value) if key == "unprivileged" && value == "1")
    );
    assert!(
        matches!(&config.entries[12], ConfEntry::KeyValue(key, value) if key == "lxc.idmap" && value == "u 0 6653600 65536")
    );
    assert!(
        matches!(&config.entries[13], ConfEntry::KeyValue(key, value) if key == "lxc.idmap" && value == "g 0 6653600 65536")
    );
    assert!(matches!(&config.entries[14], ConfEntry::EmptyLine));
    assert!(matches!(&config.entries[15], ConfEntry::Section(section) if section == "pre-setup"));
    assert!(
        matches!(&config.entries[16], ConfEntry::KeyValue(key, value) if key == "arch" && value == "amd64")
    );
    assert!(
        matches!(&config.entries[17], ConfEntry::KeyValue(key, value) if key == "cores" && value == "1")
    );
    assert!(
        matches!(&config.entries[18], ConfEntry::KeyValue(key, value) if key == "features" && value == "nesting=1")
    );
    assert!(
        matches!(&config.entries[19], ConfEntry::KeyValue(key, value) if key == "hostname" && value == "trash-pandas")
    );
    assert!(
        matches!(&config.entries[20], ConfEntry::KeyValue(key, value) if key == "memory" && value == "1024")
    );
    assert!(
        matches!(&config.entries[21], ConfEntry::KeyValue(key, value) if key == "net0" && value == "name=eth0,bridge=vmbr0,firewall=1,gw=192.168.1.1,hwaddr=AD:24:14:45:A8:38,ip=192.168.1.42/24,type=veth")
    );
    assert!(
        matches!(&config.entries[22], ConfEntry::KeyValue(key, value) if key == "ostype" && value == "debian")
    );
    assert!(
        matches!(&config.entries[23], ConfEntry::KeyValue(key, value) if key == "rootfs" && value == "local-zfs:subvol-100-disk-0,size=4G")
    );
    assert!(
        matches!(&config.entries[24], ConfEntry::KeyValue(key, value) if key == "snaptime" && value == "1764532648")
    );
    assert!(
        matches!(&config.entries[25], ConfEntry::KeyValue(key, value) if key == "swap" && value == "512")
    );
    assert!(
        matches!(&config.entries[26], ConfEntry::KeyValue(key, value) if key == "unprivileged" && value == "1")
    );
    assert!(
        matches!(&config.entries[27], ConfEntry::KeyValue(key, value) if key == "lxc.idmap" && value == "u 0 1000 3000")
    );
    assert!(
        matches!(&config.entries[28], ConfEntry::KeyValue(key, value) if key == "lxc.idmap" && value == "g 0 1000 3000")
    );

    let idmaps = config.sectionless_idmap().collect::<Vec<_>>();

    assert_eq!(idmaps.len(), 2);
    assert!(
        matches!(idmaps[0], ConfEntry::KeyValue(key, value) if key == "lxc.idmap" && value == "u 0 6653600 65536")
    );
    assert!(
        matches!(idmaps[1], ConfEntry::KeyValue(key, value) if key == "lxc.idmap" && value == "g 0 6653600 65536")
    );

    // assert_eq!(config.to_string(), content);

    Ok(())
}
