use crate::model_provider::ids::normalize_provider_id;
use crate::model_provider::profile::ProviderProfile;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct ProviderRegistry {
    profiles: BTreeMap<String, ProviderProfile>,
    aliases: BTreeMap<String, String>,
}

impl ProviderRegistry {
    pub fn insert_imported(&mut self, profile: ProviderProfile) {
        self.insert(profile);
    }

    pub fn insert_configured(&mut self, profile: ProviderProfile) {
        self.insert(profile);
    }

    pub fn get(&self, id_or_alias: &str) -> Option<&ProviderProfile> {
        let normalized = normalize_provider_id(id_or_alias);
        let id = self.aliases.get(&normalized).unwrap_or(&normalized);
        self.profiles.get(id)
    }

    pub fn profiles(&self) -> impl Iterator<Item = &ProviderProfile> {
        self.profiles.values()
    }

    fn insert(&mut self, mut profile: ProviderProfile) {
        let id = normalize_provider_id(&profile.id);
        profile.id = id.clone();
        self.aliases.remove(&id);
        for alias in &profile.aliases {
            let alias = normalize_provider_id(alias);
            if alias == id || self.profiles.contains_key(&alias) {
                continue;
            }
            self.aliases.insert(alias, id.clone());
        }
        self.profiles.insert(id, profile);
    }
}
