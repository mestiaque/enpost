//! Security helpers shared by every module. Phase 1: redaction for logs and history.
//! Later: OS credential manager access (secrets never go into SQLite), TLS policy.
pub mod redact;
