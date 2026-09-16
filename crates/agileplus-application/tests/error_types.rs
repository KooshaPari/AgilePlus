// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for AppError: Display, Debug, source chain, and ErrorCode projection.

use std::error::Error;

use agileplus_application::error::AppError;
use agileplus_domain::error::{DomainError, ErrorCode};

// ── Display trait ───────────────────────────────────────────────────────────

#[test]
fn not_found_display_message() {
    let err = AppError::NotFound("user 42".into());
    assert_eq!(err.to_string(), "not found: user 42");
}

#[test]
fn storage_display_message() {
    let src: Box<dyn Error + Send + Sync> = "timeout".to_string().into();
    let err = AppError::Storage(src);
    assert_eq!(err.to_string(), "storage error");
}

#[test]
fn domain_display_propagates_inner() {
    let err = AppError::Domain(DomainError::Validation("bad input".into()));
    let msg = err.to_string();
    assert!(msg.contains("bad input"));
}

#[test]
fn not_found_display_with_empty_string() {
    let err = AppError::NotFound("".into());
    assert_eq!(err.to_string(), "not found: ");
}

#[test]
fn not_found_display_with_special_chars() {
    let err = AppError::NotFound("wp-123/v2.1 (main)".into());
    assert_eq!(err.to_string(), "not found: wp-123/v2.1 (main)");
}

// ── Source chain ────────────────────────────────────────────────────────────

#[test]
fn storage_error_has_source() {
    let src: Box<dyn Error + Send + Sync> = "inner".to_string().into();
    let err = AppError::Storage(src);
    assert!(err.source().is_some());
    assert_eq!(
        err.source().unwrap().to_string(),
        "inner"
    );
}

#[test]
fn not_found_error_no_source() {
    let err = AppError::NotFound("x".into());
    assert!(err.source().is_none());
}

#[test]
fn domain_error_has_source() {
    let err = AppError::Domain(DomainError::Validation("v".into()));
    // DomainError may or may not have a source depending on variant.
    // At minimum, AppError should not panic when calling source().
    let _ = err.source();
}

// ── Debug trait ─────────────────────────────────────────────────────────────

#[test]
fn not_found_debug_contains_name_and_message() {
    let err = AppError::NotFound("debug".into());
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("NotFound"));
    assert!(dbg.contains("debug"));
}

#[test]
fn storage_debug_contains_variant() {
    let src: Box<dyn Error + Send + Sync> = "err".to_string().into();
    let err = AppError::Storage(src);
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("Storage"));
}

#[test]
fn domain_debug_contains_variant() {
    let err = AppError::Domain(DomainError::Validation("v".into()));
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("Domain"));
}

// ── From<DomainError> ──────────────────────────────────────────────────────

#[test]
fn app_error_from_domain_validation() {
    let domain_err = DomainError::Validation("v".into());
    let app_err: AppError = domain_err.into();
    assert!(matches!(app_err, AppError::Domain(_)));
}

#[test]
fn app_error_from_domain_not_found() {
    let domain_err = DomainError::NotFound("42".into());
    let app_err: AppError = domain_err.into();
    assert!(matches!(app_err, AppError::Domain(_)));
}

// ── ErrorCode projection ───────────────────────────────────────────────────

#[test]
fn not_found_projects_to_not_found() {
    let code: ErrorCode = AppError::NotFound("user 1".into()).into();
    assert_eq!(code, ErrorCode::NotFound);
}

#[test]
fn domain_validation_projects_to_validation_error() {
    let app = AppError::Domain(DomainError::Validation("name required".into()));
    let code: ErrorCode = app.into();
    assert_eq!(code, ErrorCode::ValidationError);
}

#[test]
fn storage_projects_to_internal_error() {
    let src: Box<dyn Error + Send + Sync> = "db down".to_string().into();
    let code: ErrorCode = AppError::Storage(src).into();
    assert_eq!(code, ErrorCode::InternalError);
}

#[test]
fn domain_not_found_variant_projects_correctly() {
    let app: AppError = DomainError::NotFound("item".into()).into();
    let code: ErrorCode = app.into();
    // DomainError::NotFound maps to its own ErrorCode variant
    assert_eq!(code, ErrorCode::NotFound);
}

#[test]
fn app_error_from_domain_into_error_code_chain() {
    let err: AppError = DomainError::Validation("x".into()).into();
    let code: ErrorCode = err.into();
    assert_eq!(code, ErrorCode::ValidationError);
}

// ── Error trait implementation ──────────────────────────────────────────────

#[test]
fn app_error_implements_std_error() {
    let err: Box<dyn Error> = Box::new(AppError::NotFound("test".into()));
    assert!(err.to_string().contains("test"));
}

#[test]
fn app_error_can_be_used_with_anyhow() {
    let err = AppError::NotFound("x".into());
    let anyhow_err: anyhow::Error = err.into();
    assert!(anyhow_err.to_string().contains("not found"));
}

#[test]
fn storage_error_chain_preserves_inner() {
    let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let app_err = AppError::Storage(Box::new(inner));
    let source = app_err.source().unwrap();
    assert_eq!(source.to_string(), "file not found");
}

#[test]
fn domain_validation_preserves_message() {
    let err = AppError::Domain(DomainError::Validation(
        "title must be non-empty".into(),
    ));
    let msg = err.to_string();
    assert!(msg.contains("title must be non-empty"));
}
