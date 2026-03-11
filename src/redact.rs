//! Environment variable redaction for stack responses.
//!
//! Controls how env var values appear in tool outputs and resources.
//! Configured via `PORTAINER_ENV_DISPLAY`, `PORTAINER_SENSITIVE_NAMES`,
//! and `PORTAINER_VISIBLE_NAMES` environment variables.

use std::collections::HashSet;

use crate::models::Stack;

/// Controls how environment variable values are displayed in tool responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvDisplayMode {
    /// All env var values replaced with `[MASKED]`. This is the default.
    Masked,
    /// Values whose names match sensitive patterns are replaced with `[REDACTED]`;
    /// all other values are shown in cleartext.
    Filtered,
    /// All values shown in cleartext. Use with caution.
    Full,
}

impl EnvDisplayMode {
    /// Read the mode from the `PORTAINER_ENV_DISPLAY` environment variable.
    ///
    /// Returns [`Masked`](EnvDisplayMode::Masked) for unset, empty, or
    /// unrecognised values.
    pub fn from_env() -> Self {
        match std::env::var("PORTAINER_ENV_DISPLAY")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "filtered" => Self::Filtered,
            "full" => Self::Full,
            _ => Self::Masked,
        }
    }

    /// Human-readable label for this mode.
    pub fn label(self) -> &'static str {
        match self {
            Self::Masked => "masked",
            Self::Filtered => "filtered",
            Self::Full => "full",
        }
    }

    /// Short description of what this mode does.
    pub fn description(self) -> &'static str {
        match self {
            Self::Masked => "all environment variable values are hidden",
            Self::Filtered => "values matching sensitive patterns are redacted, others are visible",
            Self::Full => "all environment variable values are shown in cleartext",
        }
    }
}

// ── Redaction config ────────────────────────────────────────────────────────

/// Full redaction configuration parsed from environment variables.
///
/// Combines the display mode with user-supplied override lists that
/// control which names are treated as sensitive in `filtered` mode.
#[derive(Debug, Clone)]
pub struct RedactConfig {
    mode: EnvDisplayMode,
    /// Extra names to always treat as sensitive (lowercase).
    sensitive_names: HashSet<String>,
    /// Names to always treat as visible, overriding patterns and
    /// `sensitive_names` (lowercase).
    visible_names: HashSet<String>,
}

/// Parse a comma-separated env var into a lowercase `HashSet`.
fn parse_name_list(var: &str) -> HashSet<String> {
    std::env::var(var)
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

impl RedactConfig {
    /// Build configuration from environment variables:
    ///
    /// - `PORTAINER_ENV_DISPLAY` — display mode
    /// - `PORTAINER_SENSITIVE_NAMES` — comma-separated names to add as sensitive
    /// - `PORTAINER_VISIBLE_NAMES` — comma-separated names to force visible
    pub fn from_env() -> Self {
        Self {
            mode: EnvDisplayMode::from_env(),
            sensitive_names: parse_name_list("PORTAINER_SENSITIVE_NAMES"),
            visible_names: parse_name_list("PORTAINER_VISIBLE_NAMES"),
        }
    }

    /// The active display mode.
    pub fn mode(&self) -> EnvDisplayMode {
        self.mode
    }

    /// Whether a name should be redacted in `filtered` mode.
    ///
    /// Priority: explicit visible > explicit sensitive > built-in pattern match.
    pub fn should_redact(&self, name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        if self.visible_names.contains(&lower) {
            return false;
        }
        if self.sensitive_names.contains(&lower) {
            return true;
        }
        matches_sensitive_pattern(&lower)
    }

    /// Names in the user-configured sensitive list.
    pub fn custom_sensitive_names(&self) -> &HashSet<String> {
        &self.sensitive_names
    }

    /// Names in the user-configured visible list.
    pub fn custom_visible_names(&self) -> &HashSet<String> {
        &self.visible_names
    }
}

// ── Sensitivity patterns ────────────────────────────────────────────────────

/// Case-insensitive substrings that indicate a sensitive variable.
const SENSITIVE_SUBSTRINGS: &[&str] = &[
    "password",
    "passwd",
    "secret",
    "token",
    "credential",
    "private_key",
    "api_key",
    "database_url",
    "connection_string",
];

/// Case-insensitive suffixes that indicate a sensitive variable.
const SENSITIVE_SUFFIXES: &[&str] = &["_key", "_dsn"];

/// Returns `true` if `name` matches a built-in sensitive pattern.
///
/// Expects a **lowercase** input.
fn matches_sensitive_pattern(lower: &str) -> bool {
    SENSITIVE_SUBSTRINGS.iter().any(|p| lower.contains(p))
        || SENSITIVE_SUFFIXES.iter().any(|s| lower.ends_with(s))
}

// ── Redaction ───────────────────────────────────────────────────────────────

/// Redact environment variable values in a single stack.
pub fn redact_stack(stack: &mut Stack, config: &RedactConfig) {
    match config.mode() {
        EnvDisplayMode::Full => {}
        EnvDisplayMode::Masked => {
            for var in &mut stack.env {
                var.value = Some("[MASKED]".to_string());
            }
        }
        EnvDisplayMode::Filtered => {
            for var in &mut stack.env {
                let should_redact = match &var.name {
                    Some(name) => config.should_redact(name),
                    None => true, // unknown name — assume sensitive
                };
                if should_redact {
                    var.value = Some("[REDACTED]".to_string());
                }
            }
        }
    }
}

/// Redact environment variable values in a list of stacks.
pub fn redact_stacks(stacks: &mut [Stack], config: &RedactConfig) {
    if config.mode() == EnvDisplayMode::Full {
        return;
    }
    for stack in stacks {
        redact_stack(stack, config);
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EnvVar;

    fn default_config() -> RedactConfig {
        RedactConfig {
            mode: EnvDisplayMode::Filtered,
            sensitive_names: HashSet::new(),
            visible_names: HashSet::new(),
        }
    }

    fn config_with_mode(mode: EnvDisplayMode) -> RedactConfig {
        RedactConfig {
            mode,
            ..default_config()
        }
    }

    #[test]
    fn builtin_sensitive_patterns() {
        let cfg = default_config();
        assert!(cfg.should_redact("DB_PASSWORD"));
        assert!(cfg.should_redact("jwt_secret"));
        assert!(cfg.should_redact("GITHUB_TOKEN"));
        assert!(cfg.should_redact("AWS_SECRET_ACCESS_KEY"));
        assert!(cfg.should_redact("api_key"));
        assert!(cfg.should_redact("STRIPE_API_KEY"));
        assert!(cfg.should_redact("DATABASE_URL"));
        assert!(cfg.should_redact("REDIS_DSN"));
        assert!(cfg.should_redact("MY_PRIVATE_KEY"));
        assert!(cfg.should_redact("SERVICE_CREDENTIAL"));
    }

    #[test]
    fn builtin_non_sensitive_patterns() {
        let cfg = default_config();
        assert!(!cfg.should_redact("NODE_ENV"));
        assert!(!cfg.should_redact("LOG_LEVEL"));
        assert!(!cfg.should_redact("PORT"));
        assert!(!cfg.should_redact("RUST_LOG"));
        assert!(!cfg.should_redact("TZ"));
    }

    #[test]
    fn custom_sensitive_name() {
        let cfg = RedactConfig {
            sensitive_names: HashSet::from(["my_internal_url".to_string()]),
            ..default_config()
        };
        assert!(cfg.should_redact("MY_INTERNAL_URL"));
        assert!(!cfg.should_redact("NODE_ENV"));
    }

    #[test]
    fn custom_visible_overrides_pattern() {
        let cfg = RedactConfig {
            visible_names: HashSet::from(["auth_provider_token".to_string()]),
            ..default_config()
        };
        // Would match "token" pattern, but explicit visible wins.
        assert!(!cfg.should_redact("AUTH_PROVIDER_TOKEN"));
    }

    #[test]
    fn visible_overrides_sensitive() {
        let cfg = RedactConfig {
            sensitive_names: HashSet::from(["ambiguous_var".to_string()]),
            visible_names: HashSet::from(["ambiguous_var".to_string()]),
            ..default_config()
        };
        assert!(!cfg.should_redact("AMBIGUOUS_VAR"));
    }

    fn make_env(vars: &[(&str, &str)]) -> Vec<EnvVar> {
        vars.iter()
            .map(|(n, v)| EnvVar {
                name: Some(n.to_string()),
                value: Some(v.to_string()),
            })
            .collect()
    }

    fn make_stack(env: Vec<EnvVar>) -> Stack {
        Stack {
            env,
            ..Default::default()
        }
    }

    #[test]
    fn masked_mode_hides_all() {
        let cfg = config_with_mode(EnvDisplayMode::Masked);
        let mut stack = make_stack(make_env(&[
            ("NODE_ENV", "production"),
            ("DB_PASSWORD", "hunter2"),
        ]));
        redact_stack(&mut stack, &cfg);
        assert!(
            stack
                .env
                .iter()
                .all(|v| v.value.as_deref() == Some("[MASKED]"))
        );
    }

    #[test]
    fn filtered_mode_redacts_sensitive_only() {
        let cfg = config_with_mode(EnvDisplayMode::Filtered);
        let mut stack = make_stack(make_env(&[
            ("NODE_ENV", "production"),
            ("DB_PASSWORD", "hunter2"),
        ]));
        redact_stack(&mut stack, &cfg);
        assert_eq!(stack.env[0].value.as_deref(), Some("production"));
        assert_eq!(stack.env[1].value.as_deref(), Some("[REDACTED]"));
    }

    #[test]
    fn filtered_mode_with_custom_sensitive() {
        let cfg = RedactConfig {
            sensitive_names: HashSet::from(["node_env".to_string()]),
            ..config_with_mode(EnvDisplayMode::Filtered)
        };
        let mut stack = make_stack(make_env(&[
            ("NODE_ENV", "production"),
            ("LOG_LEVEL", "debug"),
        ]));
        redact_stack(&mut stack, &cfg);
        assert_eq!(stack.env[0].value.as_deref(), Some("[REDACTED]"));
        assert_eq!(stack.env[1].value.as_deref(), Some("debug"));
    }

    #[test]
    fn filtered_mode_with_custom_visible() {
        let cfg = RedactConfig {
            visible_names: HashSet::from(["db_password".to_string()]),
            ..config_with_mode(EnvDisplayMode::Filtered)
        };
        let mut stack = make_stack(make_env(&[("DB_PASSWORD", "hunter2")]));
        redact_stack(&mut stack, &cfg);
        assert_eq!(stack.env[0].value.as_deref(), Some("hunter2"));
    }

    #[test]
    fn full_mode_shows_all() {
        let cfg = config_with_mode(EnvDisplayMode::Full);
        let mut stack = make_stack(make_env(&[
            ("NODE_ENV", "production"),
            ("DB_PASSWORD", "hunter2"),
        ]));
        redact_stack(&mut stack, &cfg);
        assert_eq!(stack.env[0].value.as_deref(), Some("production"));
        assert_eq!(stack.env[1].value.as_deref(), Some("hunter2"));
    }

    #[test]
    fn filtered_mode_redacts_unknown_names() {
        let cfg = config_with_mode(EnvDisplayMode::Filtered);
        let mut stack = Stack {
            env: vec![EnvVar {
                name: None,
                value: Some("mystery".to_string()),
            }],
            ..Default::default()
        };
        redact_stack(&mut stack, &cfg);
        assert_eq!(stack.env[0].value.as_deref(), Some("[REDACTED]"));
    }
}
