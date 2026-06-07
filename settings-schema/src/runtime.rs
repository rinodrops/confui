//! Runtime CEL evaluation against a live config snapshot.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use cel::objects::{Key, Map, Value};
use cel::{Context, ExecutionError, Program};

use crate::{Constraint, Field, FieldType, OptionState, ResolvedValidateRule, Schema};

// ---------------------------------------------------------------------------
// Config access

/// Read-only view of config values addressed by dot-separated paths.
pub trait ConfigView {
    fn get_str(&self, path: &str) -> Option<&str>;
    fn get_bool(&self, path: &str) -> Option<bool>;
    fn get_number(&self, path: &str) -> Option<f64>;
    /// Immediate child keys of the table at `path` (`""` = top level).
    fn child_keys(&self, path: &str) -> Vec<String>;
}

/// Overrides a single path for hypothetical evaluation (`option_states.when`).
pub struct ConfigOverlay<'a, V: ConfigView + ?Sized> {
    pub base: &'a V,
    pub path: &'a str,
    pub string: Option<&'a str>,
    pub number: Option<f64>,
    pub boolean: Option<bool>,
}

impl<V: ConfigView + ?Sized> ConfigView for ConfigOverlay<'_, V> {
    fn get_str(&self, path: &str) -> Option<&str> {
        if path == self.path {
            return self.string.or_else(|| self.base.get_str(path));
        }
        self.base.get_str(path)
    }

    fn get_bool(&self, path: &str) -> Option<bool> {
        if path == self.path {
            return self.boolean.or_else(|| self.base.get_bool(path));
        }
        self.base.get_bool(path)
    }

    fn get_number(&self, path: &str) -> Option<f64> {
        if path == self.path {
            return self.number.or_else(|| self.base.get_number(path));
        }
        self.base.get_number(path)
    }

    fn child_keys(&self, path: &str) -> Vec<String> {
        self.base.child_keys(path)
    }
}

// ---------------------------------------------------------------------------
// CEL context

fn cel_leaf(view: &dyn ConfigView, path: &str) -> Value {
    if let Some(b) = view.get_bool(path) {
        return Value::Bool(b);
    }
    if let Some(n) = view.get_number(path) {
        if n.fract() == 0.0 && n.is_finite() {
            return Value::Int(n as i64);
        }
        return Value::Float(n);
    }
    if let Some(s) = view.get_str(path) {
        return Value::String(Arc::new(s.to_owned()));
    }
    // Missing keys behave as zero in numeric cross-field constraints.
    Value::Int(0)
}

fn cel_value_at(view: &dyn ConfigView, path: &str) -> Value {
    let children = view.child_keys(path);
    if children.is_empty() {
        return cel_leaf(view, path);
    }
    let mut map = HashMap::new();
    for child in children {
        let child_path = if path.is_empty() {
            child.clone()
        } else {
            format!("{path}.{child}")
        };
        map.insert(
            Key::String(Arc::new(child)),
            cel_value_at(view, &child_path),
        );
    }
    Value::Map(Map {
        map: Arc::new(map),
    })
}

fn build_context(view: &dyn ConfigView) -> Context<'_> {
    let mut ctx = Context::default();
    for key in view.child_keys("") {
        let val = cel_value_at(view, &key);
        let _ = ctx.add_variable_from_value(key, val);
    }
    ctx
}

fn eval_bool(program: &Program, view: &dyn ConfigView) -> bool {
    let ctx = build_context(view);
    match program.execute(&ctx) {
        Ok(Value::Bool(b)) => b,
        Ok(_) => false,
        Err(ExecutionError::NoSuchKey(_)) => false,
        Err(_) => false,
    }
}

fn compile_expr(expr: &str) -> Result<Program, String> {
    Program::compile(expr).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Engine

/// Compiled CEL programs for constraints and field rules.
#[derive(Debug, Default)]
pub struct ValidationEngine {
    constraints: BTreeMap<String, Program>,
    inline_programs: HashMap<String, Program>,
}

impl ValidationEngine {
    /// Compile all constraint expressions. Field-inline exprs are compiled lazily.
    pub fn compile(schema: &Schema) -> Result<Self, String> {
        let mut constraints = BTreeMap::new();
        for c in &schema.constraints {
            constraints.insert(c.id.clone(), compile_expr(&c.expr)?);
        }
        Ok(Self {
            constraints,
            inline_programs: HashMap::new(),
        })
    }

    fn program_for_expr(&mut self, expr: &str) -> Result<&Program, String> {
        if !self.inline_programs.contains_key(expr) {
            let program = compile_expr(expr)?;
            self.inline_programs.insert(expr.to_owned(), program);
        }
        Ok(self.inline_programs.get(expr).expect("just inserted"))
    }

    fn eval_expr(&mut self, expr: &str, view: &dyn ConfigView) -> bool {
        match self.program_for_expr(expr) {
            Ok(program) => eval_bool(program, view),
            Err(_) => false,
        }
    }

    fn eval_constraint(&self, constraint: &Constraint, view: &dyn ConfigView) -> bool {
        self.constraints
            .get(&constraint.id)
            .map(|p| eval_bool(p, view))
            .unwrap_or(false)
    }

    /// Returns localized error messages for all failing `validate` rules on `field`.
    pub fn field_errors(
        &mut self,
        schema: &Schema,
        field: &Field,
        _key_path: &str,
        view: &dyn ConfigView,
    ) -> Vec<String> {
        let Ok(rules) = field.resolved_validate_rules(schema) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for rule in rules {
            let (expr, message) = match &rule {
                ResolvedValidateRule::Named { expr, message, .. } => (expr.as_str(), message.get()),
                ResolvedValidateRule::Inline(inline) => {
                    (inline.expr.as_str(), inline.message.get())
                }
            };
            if !self.eval_expr(expr, view) {
                out.push(message.to_owned());
            }
        }
        out
    }

    /// Whether `option` may be selected on `field` at `key_path`.
    pub fn option_enabled(
        &mut self,
        schema: &Schema,
        field: &Field,
        key_path: &str,
        option: &str,
        view: &dyn ConfigView,
    ) -> bool {
        let state = match field.option_states.iter().find(|s| s.value == option) {
            Some(s) => s,
            None => return true,
        };
        option_state_enabled(self, schema, field, key_path, state, view)
    }

    /// `true` when no field fails a `validate` rule.
    pub fn is_valid(&mut self, schema: &Schema, view: &dyn ConfigView) -> bool {
        for (field, key_path) in iter_validated_field_paths(schema, view) {
            if !self.field_errors(schema, field, &key_path, view).is_empty() {
                return false;
            }
        }
        true
    }
}

fn iter_validated_field_paths<'a>(
    schema: &'a Schema,
    view: &dyn ConfigView,
) -> Vec<(&'a Field, String)> {
    let mut out = Vec::new();
    for tab in &schema.tabs {
        if let Some(fields) = &tab.fields {
            for field in fields {
                if field.validate.is_empty() {
                    continue;
                }
                out.push((field, field.key.clone()));
            }
        }
        if let Some(sm) = &tab.section_map {
            for field in &sm.fields {
                if field.validate.is_empty() {
                    continue;
                }
                for section in view.child_keys(&sm.key_prefix) {
                    out.push((field, format!("{}.{}.{}", sm.key_prefix, section, field.key)));
                }
            }
        }
    }
    out
}

fn option_state_enabled(
    engine: &mut ValidationEngine,
    schema: &Schema,
    field: &Field,
    key_path: &str,
    state: &OptionState,
    view: &dyn ConfigView,
) -> bool {
    if let Some(when_id) = &state.when {
        let Some(constraint) = schema.constraints.iter().find(|c| c.id == *when_id) else {
            return true;
        };
        let overlay = hypothetical_overlay(field, key_path, &state.value, view);
        return engine.eval_constraint(constraint, &overlay);
    }
    if let Some(expr) = &state.enabled {
        return engine.eval_expr(expr, view);
    }
    true
}

fn hypothetical_overlay<'a>(
    field: &Field,
    key_path: &'a str,
    option: &'a str,
    view: &'a dyn ConfigView,
) -> ConfigOverlay<'a, dyn ConfigView + 'a> {
    match field.field_type {
        FieldType::Number => ConfigOverlay {
            base: view,
            path: key_path,
            string: None,
            number: option.parse::<f64>().ok(),
            boolean: None,
        },
        _ => ConfigOverlay {
            base: view,
            path: key_path,
            string: Some(option),
            number: None,
            boolean: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct MapConfig(BTreeMap<String, String>);

    impl ConfigView for MapConfig {
        fn get_str(&self, path: &str) -> Option<&str> {
            self.0.get(path).map(String::as_str)
        }

        fn get_bool(&self, _path: &str) -> Option<bool> {
            None
        }

        fn get_number(&self, path: &str) -> Option<f64> {
            self.0.get(path)?.parse().ok()
        }

        fn child_keys(&self, path: &str) -> Vec<String> {
            let prefix = if path.is_empty() {
                String::new()
            } else {
                format!("{path}.")
            };
            let mut keys = BTreeMap::<String, ()>::new();
            for full in self.0.keys() {
                if !full.starts_with(&prefix) {
                    continue;
                }
                let rest = &full[prefix.len()..];
                if let Some((first, _)) = rest.split_once('.') {
                    keys.insert(first.to_owned(), ());
                } else if !rest.is_empty() {
                    keys.insert(rest.to_owned(), ());
                }
            }
            keys.into_keys().collect()
        }
    }

    #[test]
    fn cel_sum_constraint() {
        let mut cfg = MapConfig(BTreeMap::from([
            ("validation.chars.a".into(), "0".into()),
            ("validation.chars.b".into(), "0".into()),
            ("validation.chars.c".into(), "0".into()),
        ]));
        let src = r#"
[[constraints]]
id = "min_one"
expr = "validation.chars.a + validation.chars.b + validation.chars.c >= 1"

[[tabs]]
id = "t"
label = "T"

[[tabs.fields]]
key = "validation.chars.a"
label = "A"
type = "number"
widget = "segmented_control"
options = ["0", "1"]

[[tabs.fields.option_states]]
value = "0"
when = "min_one"
"#;
        let schema = crate::parse(src).unwrap();
        schema.validate().unwrap();
        let mut engine = ValidationEngine::compile(&schema).unwrap();

        let field = &schema.tabs[0].fields.as_ref().unwrap()[0];
        assert!(!engine.option_enabled(&schema, field, "validation.chars.a", "0", &cfg));

        cfg.0.insert("validation.chars.b".into(), "2".into());
        assert!(engine.option_enabled(&schema, field, "validation.chars.a", "0", &cfg));
    }

    #[test]
    fn field_validate_shows_all_failures() {
        let cfg = MapConfig(BTreeMap::from([("x".into(), "".into())]));
        let src = r#"
[[tabs]]
id = "t"
label = "T"

[[tabs.fields]]
key = "x"
label = "X"
widget = "text_input"

[[tabs.fields.validate]]
expr = "x.size() > 0"
message = "Required"

[[tabs.fields.validate]]
expr = "x.matches('^a+$')"
message = "Must be a's"
"#;
        let schema = crate::parse(src).unwrap();
        let mut engine = ValidationEngine::compile(&schema).unwrap();
        let field = &schema.tabs[0].fields.as_ref().unwrap()[0];
        let errs = engine.field_errors(&schema, field, "x", &cfg);
        assert_eq!(errs, vec!["Required", "Must be a's"]);
    }
}
