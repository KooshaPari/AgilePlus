//! Shared test doubles for the port-contract integration tests.
//!
//! `RecordingStorage` implements only the *required* methods of
//! [`StoragePort`](agileplus_domain::ports::StoragePort) and
//! [`ContentStoragePort`](agileplus_domain::ports::ContentStoragePort), so every
//! trait default is exercised exactly as a real adapter would receive it. Each
//! call is recorded so tests can prove which method a default (or the blanket
//! repository impl) actually reached.
#![allow(dead_code, unused_imports)]

use std::sync::Mutex;

use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::domain::backlog::{
    BacklogFilters, BacklogItem, BacklogPriority, BacklogStatus,
};
use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures};
use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::{Evidence, GovernanceContract, PolicyRule};
use agileplus_domain::domain::metric::Metric;
use agileplus_domain::domain::module::{Module, ModuleFeatureTag, ModuleWithFeatures};
use agileplus_domain::domain::project::Project;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_domain::domain::user::{User, UserRole, UserStatus};
use agileplus_domain::domain::work_package::{WorkPackage, WpDependency, WpState};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::{ContentStoragePort, StoragePort};
use async_trait::async_trait;

mod content_impl;
mod storage_impl;

/// A `StoragePort` that records every required-method call and returns canned
/// success values. Trait defaults are deliberately *not* overridden.
#[derive(Debug, Default)]
pub struct RecordingStorage {
    /// Value returned by every `Result<i64, _>` method.
    pub next_id: i64,
    calls: Mutex<Vec<String>>,
}

impl RecordingStorage {
    pub fn new(next_id: i64) -> Self {
        Self {
            next_id,
            calls: Mutex::new(Vec::new()),
        }
    }

    /// Names of the required storage methods called so far, in order.
    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    pub fn reset(&self) {
        self.calls.lock().unwrap().clear();
    }

    fn record(&self, method: &str) {
        self.calls.lock().unwrap().push(method.to_string());
    }
}

/// A `ContentStoragePort` that records every required-method call. The trait's
/// single default (`list_features_by_label`) is deliberately left default.
#[derive(Debug, Default)]
pub struct RecordingContentStorage {
    pub next_id: i64,
    calls: Mutex<Vec<String>>,
}

impl RecordingContentStorage {
    pub fn new(next_id: i64) -> Self {
        Self {
            next_id,
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    pub fn reset(&self) {
        self.calls.lock().unwrap().clear();
    }

    fn record(&self, method: &str) {
        self.calls.lock().unwrap().push(method.to_string());
    }
}
