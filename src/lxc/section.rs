use compact_str::CompactString;

use crate::lxc::config::Config;

#[derive(Clone, Copy, Debug)]
pub struct SectionView<'s, 'c> {
    pub(super) config: &'c Config,
    pub(super) section: Option<&'s str>,
}

impl<'c> SectionView<'_, 'c> {
    pub fn get(&self, key: &str) -> Option<&'c str> {
        let section = self.section.map(CompactString::new);
        let key = CompactString::new(key);

        self.config
            .index
            .get(&(section, key))
            .and_then(|vals| vals.first().map(|s| s.as_str()))
    }

    #[inline]
    pub fn get_rootfs(&self) -> Option<&'c str> {
        self.get("rootfs")
    }

    #[inline]
    pub fn get_unprivileged(&self) -> Option<&'c str> {
        self.get("unprivileged")
    }

    pub fn get_all(&self, key: &str) -> impl Iterator<Item = &'c str> {
        let section = self.section.map(CompactString::new);
        let key = CompactString::new(key);

        self.config
            .index
            .get(&(section, key))
            .into_iter()
            .flatten()
            .map(|s| s.as_str())
    }

    #[inline]
    pub fn get_lxc_idmaps(&self) -> impl Iterator<Item = &'c str> {
        self.get_all("lxc.idmap")
    }

    pub fn has_key(&self, key: &str) -> bool {
        let section = self.section.map(CompactString::new);
        let key = CompactString::new(key);

        self.config.index.contains_key(&(section, key))
    }

    #[inline]
    pub fn has_lxc_idmap(&self) -> bool {
        self.has_key("lxc.idmap")
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.config.index.keys().filter_map(move |(section, key)| {
            if section.as_deref() == self.section {
                Some(key.as_str())
            } else {
                None
            }
        })
    }
}

#[test]
fn test_section_section_view() -> color_eyre::Result<()> {
    use crate::lxc::SAMPLE_CONFIG;
    use std::str::FromStr;

    let config = Config::from_str(SAMPLE_CONFIG)?;
    let section = config.section(None);

    assert!(section.has_lxc_idmap());
    assert_eq!(section.get("tags"), Some("unprivileged"));
    assert_eq!(section.get_rootfs(), Some("local-zfs:subvol-100-disk-0,size=4G"));
    assert_eq!(section.get_unprivileged(), Some("1"));
    assert_eq!(section.get_lxc_idmaps().count(), 2);

    let keys: Vec<_> = section.keys().collect();

    assert_eq!(keys.len(), 13);
    assert!(keys.contains(&"arch"));
    assert!(keys.contains(&"cores"));
    assert!(keys.contains(&"features"));
    assert!(keys.contains(&"hostname"));
    assert!(keys.contains(&"memory"));
    assert!(keys.contains(&"net0"));
    assert!(keys.contains(&"ostype"));
    assert!(keys.contains(&"parent"));
    assert!(keys.contains(&"rootfs"));
    assert!(keys.contains(&"swap"));
    assert!(keys.contains(&"tags"));
    assert!(keys.contains(&"unprivileged"));
    assert!(keys.contains(&"lxc.idmap"));

    let pre_setup = config.section("pre-setup");

    assert_eq!(pre_setup.get("snaptime"), Some("1764532648"));

    Ok(())
}
