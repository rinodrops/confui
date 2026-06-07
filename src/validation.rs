//! Bridges [`ConfigStore`] to [`settings_schema::runtime::ConfigView`].

use crate::config::ConfigStore;
use settings_schema::runtime::ConfigView;

impl ConfigView for ConfigStore {
    fn get_str(&self, path: &str) -> Option<&str> {
        ConfigStore::get_str(self, path)
    }

    fn get_bool(&self, path: &str) -> Option<bool> {
        ConfigStore::get_bool(self, path)
    }

    fn get_number(&self, path: &str) -> Option<f64> {
        ConfigStore::get_number(self, path)
    }

    fn child_keys(&self, path: &str) -> Vec<String> {
        ConfigStore::child_keys(self, path)
    }
}
