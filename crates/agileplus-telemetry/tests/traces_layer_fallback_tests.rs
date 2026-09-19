//! Fallback-path tests for `agileplus_telemetry::traces::telemetry_layer`.
//!
//! `telemetry_layer` builds an OTLP exporter from `OTEL_EXPORTER_OTLP_ENDPOINT`
//! and silently falls back to a default provider when the exporter cannot be
//! built. The malformed-endpoint branch was previously only smoke tested (the
//! layer was built and dropped); this file proves that branch still yields a
//! usable, span-recording layer.
//!
//! Both the environment and the ambient `tracing` subscriber are process-global.
//! These tests install their subscriber thread-locally (`with_default`) and share
//! one file-wide lock over the environment variable.

use std::{
    ffi::{OsStr, OsString},
    io::Write,
    sync::{Arc, Mutex, MutexGuard},
};

use agileplus_telemetry::traces::{telemetry_layer, trace_layer};
use tracing_subscriber::prelude::*;

const ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";

// ---------------------------------------------------------------------------
// Environment guard
// ---------------------------------------------------------------------------

/// One lock for the whole file: the process environment is global state.
fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Restores one environment variable on drop, even if the test panics.
struct EnvGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvGuard {
    /// Capture and set `key`. Caller must hold [`env_lock`].
    fn set(key: &'static str, value: impl AsRef<OsStr>) -> Self {
        let guard = Self {
            key,
            original: std::env::var_os(key),
        };
        // SAFETY: the caller holds `env_lock`, so no other thread in this test
        // binary reads or writes the process environment concurrently.
        unsafe { std::env::set_var(key, value) };
        guard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match self.original.take() {
            // SAFETY: as above; the lock is still held by the caller's guard.
            Some(value) => unsafe { std::env::set_var(self.key, value) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

// ---------------------------------------------------------------------------
// Capture harness
// ---------------------------------------------------------------------------

/// A `MakeWriter` that appends formatted log lines into a shared buffer.
#[derive(Clone, Default)]
struct CaptureWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer
            .lock()
            .expect("capture buffer poisoned")
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CaptureWriter {
    type Writer = CaptureWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[test]
fn layer_built_from_a_malformed_endpoint_still_records_spans() {
    let _lock = env_lock();
    let _endpoint = EnvGuard::set(ENDPOINT, "definitely not a url");

    let writer = CaptureWriter::default();
    let buffer = Arc::clone(&writer.buffer);
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(writer)
                .with_current_span(true),
        )
        .with(telemetry_layer());

    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!("agileplus.command", command = "implement");
        assert!(
            span.id().is_some(),
            "the fallback layer must still accept spans, not disable them"
        );
        let _entered = span.enter();
        tracing::info!("fallback-marker");
    });

    let output = String::from_utf8(buffer.lock().expect("capture buffer").clone())
        .expect("captured output is UTF-8");
    assert!(
        output.contains("fallback-marker"),
        "events inside the fallback layer must still be delivered: {output:?}"
    );
    assert!(
        output.contains("implement"),
        "the span fields must still be reported: {output:?}"
    );
}

#[test]
fn trace_layer_alias_uses_the_same_fallback_path() {
    let _lock = env_lock();
    let _endpoint = EnvGuard::set(ENDPOINT, "definitely not a url");

    let writer = CaptureWriter::default();
    let buffer = Arc::clone(&writer.buffer);
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(writer)
                .with_current_span(true),
        )
        .with(trace_layer());

    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!("agileplus.agent", wp = "WP10");
        let _entered = span.enter();
        tracing::info!("alias-marker");
    });

    let output = String::from_utf8(buffer.lock().expect("capture buffer").clone())
        .expect("captured output is UTF-8");
    assert!(output.contains("alias-marker"), "output: {output:?}");
    assert!(output.contains("WP10"), "output: {output:?}");
}
