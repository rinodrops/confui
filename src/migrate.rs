//! Config schema migration engine.
//!
//! Parent applications declare schema changes in `schema.toml` (see
//! [`settings_schema::Migration`]). At startup Settings compares the config
//! file's recorded version against the schema's target `schema_version` and
//! applies any pending migrations in ascending order, then writes the file back
//! preserving comments and formatting.
//!
//! Migrations are recorded via a top-level version key (default
//! `"schema_version"`, overridable by `migration_version_key`) so each step
//! runs exactly once. Every operation is idempotent, so re-running is harmless.

use settings_schema::{Migration, Schema, TransformKind};

use crate::config::{ConfigStore, Error};

/// Apply any pending migrations to `config`.
///
/// Returns `Ok(true)` when the config file was migrated and saved, `Ok(false)`
/// when nothing needed to change (no schema target, already current, or no
/// migrations declared). On success the config is persisted to disk.
pub fn run(schema: &Schema, config: &mut ConfigStore) -> Result<bool, Error> {
    let Some(target) = schema.schema_version else {
        return Ok(false);
    };

    let key = schema.version_key();
    // Files without the version key are treated as version 0, so every
    // migration applies. Operations are idempotent, so this is safe even for a
    // brand-new file whose keys never used the old layout.
    let current = config.get_version(key).unwrap_or(0);
    if current >= target {
        return Ok(false);
    }

    let mut pending: Vec<&Migration> = schema
        .migrations
        .iter()
        .filter(|m| m.version > current && m.version <= target)
        .collect();
    pending.sort_by_key(|m| m.version);

    for migration in pending {
        apply(migration, config);
    }

    // Record the target even when no operation matched, so the file is not
    // re-scanned on every launch.
    config.set_version(key, target);
    config.save()?;
    Ok(true)
}

fn apply(migration: &Migration, config: &mut ConfigStore) {
    for rename in &migration.rename {
        config.rename_key(&rename.from, &rename.to);
    }
    for transform in &migration.transform {
        match transform.kind {
            TransformKind::Rename => {
                config.rename_key(&transform.from, &transform.to);
            }
            TransformKind::EnumToBool => {
                config.transform_enum_to_bool(
                    &transform.from,
                    &transform.to,
                    transform.match_value.as_deref(),
                );
            }
        }
    }
    for delete in &migration.delete {
        config.delete_key(&delete.key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigStore;

    const SCHEMA_V2: &str = r#"
schema_version = 2

[[migration]]
version = 2
  [[migration.rename]]
  from = "display.font_size"
  to   = "general.font_size"
  [[migration.rename]]
  from = "display.sprite_size"
  to   = "free_roam.sprite_size"
  [[migration.delete]]
  key = "display.tick_rate"
  [[migration.transform]]
  from  = "mode"
  to    = "vivarium.enabled"
  type  = "enum_to_bool"
  match = "vivarium"

[[tabs]]
id = "main"
label = "Main"
"#;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "settings_engine_{}_{}.toml",
            name,
            std::process::id()
        ))
    }

    fn store(path: &std::path::Path, src: &str) -> ConfigStore {
        std::fs::write(path, src).unwrap();
        ConfigStore::load(path).unwrap()
    }

    #[test]
    fn full_v2_migration() {
        let schema = settings_schema::parse(SCHEMA_V2).unwrap();
        let path = temp_path("full");
        let mut config = store(
            &path,
            "mode = \"vivarium\"\n\n[display]\nfont_size = 14\nsprite_size = 32\ntick_rate = 60\n",
        );

        assert!(run(&schema, &mut config).unwrap());

        let reloaded = ConfigStore::load(&path).unwrap();
        assert_eq!(reloaded.get_number("general.font_size"), Some(14.0));
        assert_eq!(reloaded.get_number("free_roam.sprite_size"), Some(32.0));
        assert_eq!(reloaded.get_bool("vivarium.enabled"), Some(true));
        assert!(!reloaded.contains("display"));
        assert!(!reloaded.contains("mode"));
        assert_eq!(reloaded.get_version("schema_version"), Some(2));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn idempotent_second_run_is_noop() {
        let schema = settings_schema::parse(SCHEMA_V2).unwrap();
        let path = temp_path("idempotent");
        let mut config = store(&path, "mode = \"free\"\n\n[display]\nfont_size = 14\n");

        assert!(run(&schema, &mut config).unwrap());
        let after_first = std::fs::read_to_string(&path).unwrap();

        // A second run sees the recorded version and does nothing.
        let mut config = ConfigStore::load(&path).unwrap();
        assert!(!run(&schema, &mut config).unwrap());
        let after_second = std::fs::read_to_string(&path).unwrap();

        assert_eq!(after_first, after_second);
        assert_eq!(config.get_bool("vivarium.enabled"), Some(false));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn missing_version_treated_as_zero() {
        let schema = settings_schema::parse(SCHEMA_V2).unwrap();
        let path = temp_path("missing");
        // No schema_version key present -> treated as 0 -> v2 applies.
        let mut config = store(&path, "[display]\nfont_size = 18\n");

        assert!(run(&schema, &mut config).unwrap());
        assert_eq!(config.get_number("general.font_size"), Some(18.0));
        assert_eq!(config.get_version("schema_version"), Some(2));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn no_target_disables_migration() {
        let schema = settings_schema::parse(
            "[[tabs]]\nid = \"main\"\nlabel = \"Main\"\n",
        )
        .unwrap();
        let path = temp_path("no_target");
        let mut config = store(&path, "[display]\nfont_size = 14\n");

        assert!(!run(&schema, &mut config).unwrap());
        // Untouched.
        assert_eq!(config.get_number("display.font_size"), Some(14.0));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn only_versions_above_current_apply() {
        let schema = settings_schema::parse(
            r#"
schema_version = 2

[[migration]]
version = 1
  [[migration.rename]]
  from = "old.a"
  to   = "new.a"

[[migration]]
version = 2
  [[migration.rename]]
  from = "old.b"
  to   = "new.b"

[[tabs]]
id = "main"
label = "Main"
"#,
        )
        .unwrap();
        let path = temp_path("partial");
        // Already at version 1: only the v2 rename should run.
        let mut config = store(
            &path,
            "schema_version = 1\n\n[old]\na = 1\nb = 2\n",
        );

        assert!(run(&schema, &mut config).unwrap());
        // v1 skipped: old.a stays.
        assert_eq!(config.get_number("old.a"), Some(1.0));
        // v2 applied: old.b -> new.b.
        assert_eq!(config.get_number("new.b"), Some(2.0));
        assert!(!config.contains("old.b"));
        assert_eq!(config.get_version("schema_version"), Some(2));
        let _ = std::fs::remove_file(path);
    }
}
