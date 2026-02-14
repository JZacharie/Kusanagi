//! Diagnostic Domain Entities
//!
//! Entities for system diagnostics and health checks

use serde::{Deserialize, Serialize};

// ==================== Check Status ====================

/// Status of a diagnostic check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Ok,
    Warning,
    Error,
    Skipped,
}

// ==================== Check Result ====================

/// Result of a single diagnostic check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub details: Option<String>,
    pub recommendation: Option<String>,
    pub duration_ms: u64,
}

// ==================== Diagnostic Summary ====================

/// Summary of all diagnostic checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSummary {
    pub total: usize,
    pub ok: usize,
    pub warning: usize,
    pub error: usize,
    pub skipped: usize,
}

impl DiagnosticSummary {
    pub fn new() -> Self {
        Self {
            total: 0,
            ok: 0,
            warning: 0,
            error: 0,
            skipped: 0,
        }
    }

    pub fn increment(&mut self, status: &CheckStatus) {
        self.total += 1;
        match status {
            CheckStatus::Ok => self.ok += 1,
            CheckStatus::Warning => self.warning += 1,
            CheckStatus::Error => self.error += 1,
            CheckStatus::Skipped => self.skipped += 1,
        }
    }
}

impl Default for DiagnosticSummary {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Diagnostic Report ====================

/// Complete diagnostic report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub overall_status: CheckStatus,
    pub timestamp: String,
    pub version: String,
    pub checks: Vec<CheckResult>,
    pub summary: DiagnosticSummary,
    pub recommendations: Vec<String>,
}

// ==================== Quick Health Response ====================

/// Quick health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickHealthResponse {
    pub healthy: bool,
    pub kubernetes: bool,
    pub permissions: bool,
    pub duration_ms: u64,
}
