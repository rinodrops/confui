use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Table};

// ---------------------------------------------------------------------------
// Error

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(toml_edit::TomlError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Parse(e) => write!(f, "TOML parse error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<toml_edit::TomlError> for Error {
    fn from(e: toml_edit::TomlError) -> Self {
        Error::Parse(e)
    }
}

// ---------------------------------------------------------------------------
// NumberRepr

/// Controls whether whole-number values are written as TOML integers or floats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberRepr {
    WholeAsInteger,
    AlwaysFloat,
}

impl NumberRepr {
    /// Integer `step` → integer literals; fractional `step` → float literals.
    pub fn from_step(step: f64) -> Self {
        if step.fract() == 0.0 {
            NumberRepr::WholeAsInteger
        } else {
            NumberRepr::AlwaysFloat
        }
    }

    /// No `.` in any option string → integer literals; otherwise floats.
    pub fn from_options(options: &[String]) -> Self {
        if options.iter().all(|o| !o.contains('.')) {
            NumberRepr::WholeAsInteger
        } else {
            NumberRepr::AlwaysFloat
        }
    }
}

fn number_item(value: f64, repr: NumberRepr) -> Item {
    match repr {
        NumberRepr::WholeAsInteger if value.fract() == 0.0 && value.is_finite() => {
            toml_edit::value(value as i64)
        }
        _ => toml_edit::value(value),
    }
}

// ---------------------------------------------------------------------------
// ConfigStore

pub struct ConfigStore {
    doc: DocumentMut,
    path: PathBuf,
    /// Set whenever `set_*()` or `remove()` mutates the in-memory state.
    /// Cleared by `save()` and `take_dirty()`.
    dirty: bool,
}

impl ConfigStore {
    /// Load a TOML config file from `path`, preserving all formatting and comments.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let src = fs::read_to_string(path.as_ref())?;
        let doc: DocumentMut = src.parse()?;
        Ok(Self {
            doc,
            path: path.as_ref().to_path_buf(),
            dirty: false,
        })
    }

    /// Write the current state back to disk using an atomic rename.
    /// Original formatting and comments are preserved.
    /// Clears the dirty flag on success.
    pub fn save(&mut self) -> Result<(), Error> {
        let s = self.doc.to_string();
        let tmp = self.path.with_extension("toml.tmp");
        fs::write(&tmp, s.as_bytes())?;
        fs::rename(&tmp, &self.path)?;
        self.dirty = false;
        Ok(())
    }

    /// Returns `true` if unsaved changes exist, then clears the flag.
    #[allow(dead_code)] // retained for callers that peek-and-clear; auto-save uses `is_dirty`.
    pub fn take_dirty(&mut self) -> bool {
        let d = self.dirty;
        self.dirty = false;
        d
    }

    /// Returns `true` if unsaved changes exist, without clearing the flag.
    /// Used to enable/disable the "Apply" button.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    // -----------------------------------------------------------------------
    // Internal navigation

    fn get_item(&self, path: &str) -> Option<&Item> {
        let mut parts = path.split('.');
        let first = parts.next()?;
        let first_item = self.doc.get(first)?;
        parts.try_fold(first_item, |item, key| item.as_table_like()?.get(key))
    }

    /// Recursively descend through `table` along `keys`, creating any missing
    /// intermediate tables. Returns `None` if a non-table node is encountered.
    fn descend_or_create<'a>(table: &'a mut Table, keys: &[&str]) -> Option<&'a mut Table> {
        match keys {
            [] => Some(table),
            [key, rest @ ..] => {
                if !table.contains_key(*key) {
                    table.insert(*key, Item::Table(Table::new()));
                }
                let child = table.get_mut(*key)?.as_table_mut()?;
                Self::descend_or_create(child, rest)
            }
        }
    }

    /// Recursively descend through `table` along `keys` without creating anything.
    fn descend<'a>(table: &'a mut Table, keys: &[&str]) -> Option<&'a mut Table> {
        match keys {
            [] => Some(table),
            [key, rest @ ..] => {
                let child = table.get_mut(*key)?.as_table_mut()?;
                Self::descend(child, rest)
            }
        }
    }

    fn set_item(&mut self, path: &str, new_val: Item) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let Some((last, parents)) = parts.split_last() else {
            return false;
        };
        let root = self.doc.as_table_mut();
        let Some(parent) = Self::descend_or_create(root, parents) else {
            return false;
        };
        parent.insert(last, new_val);
        self.dirty = true;
        true
    }

    // -----------------------------------------------------------------------
    // Public read API

    /// Get a string value by dot-separated path.
    pub fn get_str<'a>(&'a self, path: &str) -> Option<&'a str> {
        self.get_item(path)?.as_str()
    }

    /// Get a boolean value by dot-separated path.
    pub fn get_bool(&self, path: &str) -> Option<bool> {
        self.get_item(path)?.as_bool()
    }

    /// Get a numeric (float or integer) value by dot-separated path.
    pub fn get_number(&self, path: &str) -> Option<f64> {
        let item = self.get_item(path)?;
        item.as_float().or_else(|| item.as_integer().map(|i| i as f64))
    }

    // -----------------------------------------------------------------------
    // Public write API

    /// Set a string value by dot-separated path, creating intermediate tables as needed.
    pub fn set_str(&mut self, path: &str, value: &str) -> bool {
        self.set_item(path, toml_edit::value(value))
    }

    /// Set a boolean value by dot-separated path, creating intermediate tables as needed.
    pub fn set_bool(&mut self, path: &str, value: bool) -> bool {
        self.set_item(path, toml_edit::value(value))
    }

    /// Set a numeric value by dot-separated path, creating intermediate tables as needed.
    /// Writes a float literal unless `repr` requests integers for whole values.
    pub fn set_number(&mut self, path: &str, value: f64, repr: NumberRepr) -> bool {
        self.set_item(path, number_item(value, repr))
    }

    /// Insert an empty table at `path`, creating intermediate tables as needed.
    /// Used when adding a new section.
    pub fn set_table(&mut self, path: &str) -> bool {
        self.set_item(path, Item::Table(Table::new()))
    }

    // -----------------------------------------------------------------------
    // Other public API

    /// Return the filesystem path of the config file.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Discard in-memory state and reload from disk.
    /// Clears the dirty flag on success.
    pub fn reload(&mut self) -> Result<(), Error> {
        let src = std::fs::read_to_string(&self.path)?;
        self.doc = src.parse()?;
        self.dirty = false;
        Ok(())
    }

    /// Return the immediate child keys of the table at `path`.
    /// Used to enumerate sub-sections for `section_map` tabs.
    pub fn section_keys(&self, path: &str) -> Vec<String> {
        self.child_keys(path)
    }

    /// Immediate child keys of the table at `path` (`""` = document root).
    pub fn child_keys(&self, path: &str) -> Vec<String> {
        if path.is_empty() {
            return self
                .doc
                .as_table()
                .iter()
                .map(|(k, _)| k.to_string())
                .collect();
        }
        self.get_item(path)
            .and_then(|item| item.as_table_like())
            .map(|t| t.iter().map(|(k, _)| k.to_string()).collect())
            .unwrap_or_default()
    }

    /// Remove a key by dot-separated path.
    /// Returns `true` if the key existed and was removed.
    pub fn remove(&mut self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let Some((last, parents)) = parts.split_last() else {
            return false;
        };
        let root = self.doc.as_table_mut();
        let Some(parent) = Self::descend(root, parents) else {
            return false;
        };
        let removed = parent.remove(last).is_some();
        if removed {
            self.dirty = true;
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_repr_from_step() {
        assert_eq!(NumberRepr::from_step(1.0), NumberRepr::WholeAsInteger);
        assert_eq!(NumberRepr::from_step(0.5), NumberRepr::AlwaysFloat);
    }

    #[test]
    fn number_repr_from_options() {
        assert_eq!(
            NumberRepr::from_options(&["0".into(), "1".into(), "5".into()]),
            NumberRepr::WholeAsInteger
        );
        assert_eq!(
            NumberRepr::from_options(&["0.5".into(), "1.0".into()]),
            NumberRepr::AlwaysFloat
        );
    }

    #[test]
    fn set_number_writes_integer_literal() {
        let path = std::env::temp_dir().join(format!(
            "settings_number_repr_test_{}.toml",
            std::process::id()
        ));
        std::fs::write(&path, "[section]\n").unwrap();

        let mut store = ConfigStore::load(&path).unwrap();
        store.set_number("section.count", 3.0, NumberRepr::WholeAsInteger);
        store.save().unwrap();

        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.contains("count = 3"));
        assert!(!saved.contains("count = 3.0"));

        let _ = std::fs::remove_file(path);
    }
}
