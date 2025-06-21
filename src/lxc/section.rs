use compact_str::CompactString;

use crate::lxc::config::Config;

#[derive(Debug)]
pub struct SectionView<'s, 'c> {
    pub(super) config: &'c Config,
    pub(super) section: Option<&'s str>,
}

impl<'s, 'c> SectionView<'s, 'c> {
    pub fn get(&self, key: &str) -> Option<&'c str> {
        let section = self.section.map(CompactString::new);
        let key = CompactString::new(key);

        self.config
            .index
            .get(&(section, key))
            .and_then(|vals| vals.first().map(|s| s.as_str()))
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

    pub fn has_key(&self, key: &str) -> bool {
        let section = self.section.map(CompactString::new);
        let key = CompactString::new(key);

        self.config.index.contains_key(&(section, key))
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
