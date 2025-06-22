use compact_str::CompactString;

use crate::lxc::config::{ConfEntry, Config};

#[derive(Debug)]
pub struct SectionViewMut<'s, 'c> {
    pub(super) config: &'c mut Config,
    pub(super) section: Option<&'s str>,
}

impl<'s, 'c> SectionViewMut<'s, 'c> {
    pub fn set(&mut self, key: &str, value: &str) {
        self.remove_all(key);
        self.append(key, value);
    }

    pub fn append(&mut self, key: &str, value: &str) {
        let key = CompactString::new(key);
        let value = CompactString::new(value);
        let section_key = (self.section.map(CompactString::new), key.clone());

        // Update index
        self.config.index.entry(section_key).or_default().push(value.clone());

        // Insert into entries
        let insert_index = self.find_append_point();

        self.config
            .entries
            .insert(insert_index, ConfEntry::KeyValue(key, value));
    }

    pub fn remove_all(&mut self, key: &str) {
        let section_key = (self.section.map(CompactString::new), CompactString::new(key));

        self.config.index.remove(&section_key);

        let mut in_section = self.section.is_none();
        let section = self.section;

        self.config.entries.retain(move |entry| match entry {
            ConfEntry::Section(sec) => {
                in_section = section == Some(sec.as_str());
                true
            },
            ConfEntry::KeyValue(k, _) if in_section && k == key => false,
            _ => true,
        });
    }

    fn find_append_point(&self) -> usize {
        let mut in_section = self.section.is_none();
        let mut last_match_index = None;

        for (i, entry) in self.config.entries.iter().enumerate() {
            match entry {
                ConfEntry::Section(sec) => {
                    in_section = self.section == Some(sec.as_str());
                },
                ConfEntry::KeyValue(_, _) if in_section => {
                    last_match_index = Some(i);
                },
                _ => {},
            }
        }

        match last_match_index {
            Some(i) => i + 1,
            None => self.config.entries.len(),
        }
    }
}
