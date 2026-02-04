//! Kusanagi - Version minimale pour tests

pub mod error;
pub mod config;
pub mod cache;
pub mod features;
pub mod response;
pub mod validation;

// Modules de base seulement
pub mod domain {
    pub mod entities {
        pub mod mod_minimal {
            use serde::{Deserialize, Serialize};
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct TestEntity {
                pub id: String,
                pub name: String,
            }
        }
    }
}

// Tests de base
#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_compilation() {
        assert_eq!(2 + 2, 4);
    }
    
    #[test]
    fn test_legacy_modules_exist() {
        use std::path::Path;
        assert!(Path::new("src/legacy").exists());
    }
    
    #[test]
    fn test_error_module() {
        use crate::error::KusanagiError;
        let err = KusanagiError::internal("test");
        assert!(!err.to_string().is_empty());
    }
}
