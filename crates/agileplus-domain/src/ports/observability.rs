//! Observability port — tracing and metrics abstraction.

/// Lightweight span context passed between observability calls.
#[derive(Debug, Clone, Default)]
pub struct SpanContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
}

/// Severity levels for structured log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// A single structured log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub fields: Vec<(String, String)>,
}

/// Observability port — implemented by telemetry adapters (OpenTelemetry, no-op, test).
pub trait ObservabilityPort: Send + Sync {
    fn start_span(&self, name: &str, parent: Option<&SpanContext>) -> SpanContext;
    fn end_span(&self, ctx: &SpanContext);
    fn add_span_event(&self, ctx: &SpanContext, name: &str, attrs: &[(&str, &str)]);
    fn set_span_error(&self, ctx: &SpanContext, err: &str);
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
    fn log(&self, entry: &LogEntry);
    fn log_info(&self, message: &str);
    fn log_warn(&self, message: &str);
    fn log_error(&self, message: &str);
}
