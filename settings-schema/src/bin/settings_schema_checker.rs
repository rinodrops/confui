//! Validate a `schema.toml` file (parse, cross-references, CEL syntax).
//!
//! Usage: `settings-schema-checker <path-to-schema.toml>`

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let path = match env::args().nth(1) {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!("usage: settings-schema-checker <path-to-schema.toml>");
            eprintln!();
            eprintln!("Validates TOML syntax, constraint references, option_states,");
            eprintln!("and CEL expression syntax.");
            return ExitCode::from(2);
        }
    };

    match settings_schema::check_schema_file(&path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprint!("{msg}");
            ExitCode::from(1)
        }
    }
}
