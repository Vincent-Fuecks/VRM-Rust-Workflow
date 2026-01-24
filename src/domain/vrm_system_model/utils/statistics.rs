use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};

use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling;
use tracing_subscriber::{Layer, layer::Context};

pub const ANALYTICS_TARGET: &str = "analytics";

const CSV_HEADERS: &[&str] = &[
    "Time",
    "LogDescription",
    "ComponentType",
    "ComponentName",
    "ComponentCapacity",
    "ComponentUtilization",
    "ComponentFragmentation",
    "ReservationName",
    "ReservationCapacity",
    "ReservationWorkload",
    "ReservationState",
    "ReservationProceeding",
    "NumberOfTasks",
    "Command",
    "ProcessingTime",
    "FragmentationBefore",
    "FragmentationAfter",
    "NumberOfCoAllocationDependencies",
    "NumberOfDataDependencies",
    "ProbeAnswers",
];

/// A custom Tracing Layer that writes analytics events to a CSV.
pub struct CsvAnalyticsLayer {
    /// Mutex is required because `Layer::on_event` is immutable (&self),
    /// but writing to the CSV writer requires mutability.
    /// We use `NonBlocking` writer here to ensure the application thread
    /// never blocks on disk I/O.
    writer: Mutex<csv::Writer<NonBlocking>>,
}

impl CsvAnalyticsLayer {
    pub fn new(writer: NonBlocking) -> Self {
        // We do not wrap in BufWriter because tracing_appender::NonBlocking
        // already buffers internally.
        let mut wtr = csv::WriterBuilder::new().delimiter(b';').from_writer(writer);

        // Write the header row immediately.
        // Note: In a rolling file scenario, this header writes every time the app starts,
        // but new rolling files created while running might miss headers unless handled
        // by a more complex custom appender.
        // For a prototype, writing headers on startup is sufficient.
        if let Err(e) = wtr.write_record(CSV_HEADERS) {
            eprintln!("Failed to write CSV headers: {}", e);
        }
        let _ = wtr.flush();

        Self { writer: Mutex::new(wtr) }
    }
}

impl<S> Layer<S> for CsvAnalyticsLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // 1. Filter: Ignore standard logs, only process our "analytics" target
        if event.metadata().target() != ANALYTICS_TARGET {
            return;
        }

        // 2. Extract: Use a Visitor to walk the fields of the event
        let mut visitor = CsvVisitor::default();
        event.record(&mut visitor);

        // 3. Write: Lock the writer and output the row
        // Because we use NonBlocking, this lock is held only for the duration
        // of a memory copy, not a disk write.
        if let Ok(mut wtr) = self.writer.lock() {
            // Map the collected fields to the strict CSV_HEADERS order
            let row: Vec<String> =
                CSV_HEADERS.iter().map(|header| visitor.fields.get(*header).cloned().unwrap_or_else(|| "N/A".to_string())).collect();

            if let Err(e) = wtr.write_record(&row) {
                eprintln!("Failed to write analytics row: {}", e);
            }
            // No need to flush manually; the background worker handles it.
        }
    }
}

/// A helper struct to "visit" (extract) values from a tracing Event.
#[derive(Default)]
struct CsvVisitor {
    fields: HashMap<String, String>,
}

impl Visit for CsvVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        // Fallback for types not handled specifically below
        self.fields.insert(field.name().to_string(), format!("{:?}", value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name().to_string(), value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields.insert(field.name().to_string(), value.to_string());
    }

    // Implement record_bool, record_u64, etc. as needed...
}

/// Initialize the global subscriber for high-throughput analytics.
///
/// # Returns
/// Returns a `WorkerGuard`. This guard MUST be assigned to a variable in `main`
/// (e.g. `let _guard = init_tracing(...)`). Dropping it will shut down the background writer.
pub fn init_tracing(directory: &str, filename_prefix: &str) -> WorkerGuard {
    use tracing_subscriber::prelude::*;

    // Create a rolling file appender (e.g., logs/analytics.2023-10-27)
    let file_appender = rolling::daily(directory, filename_prefix);

    // Wrap it in a non-blocking writer.
    // 'non_blocking' is the writer handle, 'guard' keeps the worker thread alive.
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let csv_layer = CsvAnalyticsLayer::new(non_blocking);

    tracing_subscriber::registry()
        .with(csv_layer)
        .with(tracing_subscriber::fmt::layer()) // Keep standard console logging
        .init();

    guard
}
