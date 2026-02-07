use std::sync::{Mutex, MutexGuard, PoisonError};

/// Helper trait to handle poisoned mutexes gracefully
pub trait MutexExt<T> {
    /// Lock the mutex, recovering from poison errors
    fn lock_safe(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T> {
        match self.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("Mutex was poisoned, recovering");
                poisoned.into_inner()
            }
        }
    }
}

/// Helper to handle Result unwrapping with logging
pub trait ResultExt<T, E> {
    fn unwrap_or_log(self, default: T, context: &str) -> T
    where
        E: std::fmt::Display;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn unwrap_or_log(self, default: T, context: &str) -> T
    where
        E: std::fmt::Display,
    {
        match self {
            Ok(value) => value,
            Err(e) => {
                tracing::error!("{}: {}", context, e);
                default
            }
        }
    }
}

/// Helper to get value from Option<String> without cloning
pub trait OptionStringExt {
    fn as_str_or_default(&self) -> &str;
}

impl OptionStringExt for Option<String> {
    fn as_str_or_default(&self) -> &str {
        self.as_deref().unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutex_ext() {
        let mutex = Mutex::new(42);
        let guard = mutex.lock_safe();
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_result_ext() {
        let ok: Result<i32, String> = Ok(42);
        assert_eq!(ok.unwrap_or_log(0, "test"), 42);

        let err: Result<i32, String> = Err("error".to_string());
        assert_eq!(err.unwrap_or_log(0, "test"), 0);
    }

    #[test]
    fn test_option_string_ext() {
        let some = Some("test".to_string());
        assert_eq!(some.as_str_or_default(), "test");

        let none: Option<String> = None;
        assert_eq!(none.as_str_or_default(), "");
    }
}
