//! Feature Flags System
//!
//! Dynamic feature toggling for gradual rollouts and A/B testing.
//! Supports multiple backends: in-memory, Redis (future), environment variables.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

/// Feature flag state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeatureState {
    /// Feature is enabled for everyone
    Enabled,
    /// Feature is disabled
    Disabled,
    /// Feature is enabled for a percentage of users (0-100)
    Percentage(u8),
}

impl Default for FeatureState {
    fn default() -> Self {
        FeatureState::Disabled
    }
}

/// Feature flag definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub name: String,
    pub state: FeatureState,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Feature flags manager
pub struct FeatureFlags {
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
}

impl FeatureFlags {
    /// Create new feature flags manager
    pub fn new() -> Self {
        let flags = Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Load from environment
        flags.load_from_env();
        
        flags
    }

    /// Load feature flags from environment variables
    /// Format: FEATURE_XXX=enabled|disabled|percentage:N
    fn load_from_env(&self) {
        for (key, value) in std::env::vars() {
            if key.starts_with("FEATURE_") {
                let name = key.trim_start_matches("FEATURE_").to_lowercase();
                let state = parse_feature_state(&value);
                
                self.set_internal(&name, state, Some(format!("From env var {}", key)));
                info!(feature = %name, state = ?state, "Loaded feature flag from environment");
            }
        }
    }

    /// Check if feature is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.check(name, None)
    }

    /// Check if feature is enabled for a specific user/context
    pub fn check(&self, name: &str, context: Option<&str>) -> bool {
        let flags = self.flags.read().unwrap();
        
        match flags.get(name) {
            Some(flag) => match flag.state {
                FeatureState::Enabled => true,
                FeatureState::Disabled => false,
                FeatureState::Percentage(pct) => {
                    // Use context to determine consistency
                    let hash = match context {
                        Some(ctx) => {
                            // Hash the context for consistent results
                            use std::collections::hash_map::DefaultHasher;
                            use std::hash::{Hash, Hasher};
                            let mut hasher = DefaultHasher::new();
                            ctx.hash(&mut hasher);
                            hasher.finish()
                        }
                        None => {
                            // Random check (not consistent)
                            rand::random::<u64>()
                        }
                    };
                    
                    (hash % 100) < (pct as u64)
                }
            },
            None => {
                // Feature not found - default to disabled
                warn!(feature = %name, "Feature flag not found, defaulting to disabled");
                false
            }
        }
    }

    /// Set feature flag state
    pub fn set(&self, name: &str, state: FeatureState, description: Option<String>) {
        self.set_internal(name, state, description);
        info!(feature = %name, state = ?state, "Feature flag updated");
    }

    fn set_internal(&self, name: &str, state: FeatureState, description: Option<String>) {
        let mut flags = self.flags.write().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        
        let flag = FeatureFlag {
            name: name.to_string(),
            state,
            description,
            created_at: flags.get(name).map(|f| f.created_at.clone()).unwrap_or_else(|| now.clone()),
            updated_at: now,
        };
        
        flags.insert(name.to_string(), flag);
    }

    /// Get all feature flags
    pub fn get_all(&self) -> Vec<FeatureFlag> {
        let flags = self.flags.read().unwrap();
        flags.values().cloned().collect()
    }

    /// Get specific feature flag
    pub fn get(&self, name: &str) -> Option<FeatureFlag> {
        let flags = self.flags.read().unwrap();
        flags.get(name).cloned()
    }

    /// Delete feature flag
    pub fn delete(&self, name: &str) -> bool {
        let mut flags = self.flags.write().unwrap();
        flags.remove(name).is_some()
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse feature state from string
fn parse_feature_state(s: &str) -> FeatureState {
    let s = s.to_lowercase();
    
    if s == "enabled" || s == "true" || s == "1" {
        FeatureState::Enabled
    } else if s.starts_with("percentage:") || s.starts_with("pct:") {
        let pct_str = s.split(':').nth(1).unwrap_or("0");
        if let Ok(pct) = pct_str.parse::<u8>() {
            FeatureState::Percentage(pct.min(100))
        } else {
            FeatureState::Disabled
        }
    } else {
        FeatureState::Disabled
    }
}

// ============== Convenience Functions ==============

static FEATURES: once_cell::sync::OnceCell<FeatureFlags> = once_cell::sync::OnceCell::new();

/// Initialize global feature flags
pub fn init() -> &'static FeatureFlags {
    FEATURES.get_or_init(FeatureFlags::new)
}

/// Check if feature is enabled
pub fn is_enabled(name: &str) -> bool {
    init().is_enabled(name)
}

/// Check feature with context
pub fn check(name: &str, context: Option<&str>) -> bool {
    init().check(name, context)
}

/// Set feature flag
pub fn set(name: &str, state: FeatureState, description: Option<String>) {
    init().set(name, state, description);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_state_parsing() {
        assert_eq!(parse_feature_state("enabled"), FeatureState::Enabled);
        assert_eq!(parse_feature_state("true"), FeatureState::Enabled);
        assert_eq!(parse_feature_state("disabled"), FeatureState::Disabled);
        assert_eq!(parse_feature_state("percentage:50"), FeatureState::Percentage(50));
        assert_eq!(parse_feature_state("pct:75"), FeatureState::Percentage(75));
    }

    #[test]
    fn test_feature_enabled() {
        let features = FeatureFlags::new();
        features.set("test", FeatureState::Enabled, None);
        assert!(features.is_enabled("test"));
    }

    #[test]
    fn test_feature_disabled() {
        let features = FeatureFlags::new();
        features.set("test", FeatureState::Disabled, None);
        assert!(!features.is_enabled("test"));
    }

    #[test]
    fn test_feature_percentage_consistency() {
        let features = FeatureFlags::new();
        features.set("test", FeatureState::Percentage(50), None);
        
        // Same context should give same result
        let context = "user_123";
        let result1 = features.check("test", Some(context));
        let result2 = features.check("test", Some(context));
        assert_eq!(result1, result2);
    }
}
