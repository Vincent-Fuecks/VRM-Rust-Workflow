use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::sync::{OnceLock, mpsc};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// Each event consists of a set of key-value-pairs with the measured data or some meta data of the event.
/// This enum specifies all allowed key values and thus the column in the output file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum StatParameter {
    /// Time in seconds since simulation start.
    Time,

    /// Description why this entry was made
    LogDescription,

    // Component
    /// Type of the component is "CLIENT", "ADC", "AI" or "SIMULATOR"
    ComponentType,

    /// Name of component
    ComponentName,

    /// Overall or estimated capacity of the component
    ComponentCapacity,

    /// Load of component
    ComponentUtilization,

    /// Fragmentation of component */
    ComponentFragmentation,

    // Reservation
    /// Name of the reservation
    ReservationName,

    /// Size of Reservation (capacity)
    ReservationCapacity,

    /// Overall size of Reservation (capacity * duration)
    ReservationWorkload,

    /// ReservationState of Reservation
    ReservationState,

    /// ReservationProceeding of Reservation
    ReservationProceeding,

    /// Number of Tasks in Reservation. Is 1 if Reservation was not a Workflow
    NumberOfTasks,

    // Operation
    /// Command send or answered
    Command,

    /// Time to process command in ms
    ProcessingTime,

    /// System fragmentation before reservation  
    FragmentationBefore,

    /// System fragmentation after reservation
    FragmentationAfter,

    /// Number of CoAllocation Dependencies (if the reservation is a Workflow)
    NumberOfCoAllocationDependencies,

    /// Number of DataDependencies (if the reservation is a Workflow)
    NumberOfDataDependencies,
}

impl StatParameter {
    /// Returns the defined order of columns for the CSV header
    pub fn headers() -> Vec<&'static str> {
        vec![
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
        ]
    }
}

/// store values in their native format, only format them when writing to the CSV.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum StatValue {
    Integer(i64),
    Float(f64),
    Text(String),
    Bool(bool),
}

// Automatic conversion helpers
impl From<i32> for StatValue {
    fn from(v: i32) -> Self {
        StatValue::Integer(v as i64)
    }
}

impl From<i64> for StatValue {
    fn from(v: i64) -> Self {
        StatValue::Integer(v)
    }
}

impl From<f64> for StatValue {
    fn from(v: f64) -> Self {
        StatValue::Float(v)
    }
}

impl From<String> for StatValue {
    fn from(v: String) -> Self {
        StatValue::Text(v)
    }
}

impl From<&str> for StatValue {
    fn from(v: &str) -> Self {
        StatValue::Text(v.to_string())
    }
}

impl From<bool> for StatValue {
    fn from(v: bool) -> Self {
        StatValue::Bool(v)
    }
}

// --- 2. The Event Object ---

#[derive(Debug, Clone)]
pub struct StatisticEvent {
    data: HashMap<StatParameter, StatValue>,
}

impl StatisticEvent {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    pub fn set<V: Into<StatValue>>(&mut self, param: StatParameter, value: V) -> &mut Self {
        self.data.insert(param, value.into());
        self
    }

    pub fn get(&self, param: StatParameter) -> Option<&StatValue> {
        self.data.get(&param)
    }
}

/// Messages sent from the simulation threads to the writer thread.
enum StatsMessage {
    Log(StatisticEvent),
    Flush,
    Shutdown,
}

/// The global handle that allows components to log events.
/// It holds the "Sender" side of the channel.
pub struct StatsCollector {
    sender: mpsc::Sender<StatsMessage>,
    start_time: u64,
}

impl StatsCollector {
    /// Initialize the statistics system.
    /// Spawns a background thread that manages the file writing.
    pub fn init(filename: Option<String>) -> Self {
        let (tx, rx) = mpsc::channel();

        let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Spawn the background writer thread
        thread::spawn(move || {
            Self::worker_loop(rx, filename);
        });

        StatsCollector { sender: tx, start_time }
    }

    /// The logic running in the background thread.
    fn worker_loop(rx: mpsc::Receiver<StatsMessage>, filename: Option<String>) {
        // Setup Output (File or Stdout)
        let writer: Box<dyn Write> = match filename {
            Some(f) => Box::new(File::create(f).expect("Could not create statistics file")),
            None => Box::new(io::stdout()),
        };

        // Initialize CSV Writer
        let mut csv_wtr = csv::WriterBuilder::new().delimiter(b';').from_writer(writer);

        // Write Header
        let headers = StatParameter::headers();
        if let Err(e) = csv_wtr.write_record(&headers) {
            log::error!("Stats Error: Failed to write headers: {}", e);
        }

        // Process incoming messages
        for msg in rx {
            match msg {
                StatsMessage::Log(event) => {
                    // Convert the Map into a Row based on Header order
                    let row: Vec<String> = StatParameter::headers()
                        .iter()
                        .map(|header_str| {
                            // Find the enum variant matching this header string
                            // (In a real app, you might iterate the enum variants directly to avoid string matching overhead)
                            let param = Self::str_to_param(header_str);

                            match param {
                                Some(p) => match event.data.get(&p) {
                                    // Use Serde to format the value safely (handles quotes, etc)
                                    Some(val) => match val {
                                        StatValue::Text(t) => t.clone(),
                                        StatValue::Integer(i) => i.to_string(),
                                        StatValue::Float(f) => f.to_string(),
                                        StatValue::Bool(b) => b.to_string(),
                                    },
                                    None => "NA".to_string(),
                                },
                                None => "ERROR".to_string(),
                            }
                        })
                        .collect();

                    if let Err(e) = csv_wtr.write_record(&row) {
                        eprintln!("Stats Error: Failed to write record: {}", e);
                    }
                }
                StatsMessage::Flush => {
                    let _ = csv_wtr.flush();
                }
                StatsMessage::Shutdown => {
                    let _ = csv_wtr.flush();
                    break;
                }
            }
        }
    }

    // Helper to map header strings back to Enums (simple lookup)
    fn str_to_param(s: &str) -> Option<StatParameter> {
        // In a production app, use `strum` crate for EnumString derivation
        match s {
            "Time" => Some(StatParameter::Time),
            "LogDescription" => Some(StatParameter::LogDescription),
            "ComponentType" => Some(StatParameter::ComponentType),
            "ComponentName" => Some(StatParameter::ComponentName),
            "ComponentCapacity" => Some(StatParameter::ComponentCapacity),
            "ComponentUtilization" => Some(StatParameter::ComponentUtilization),
            "ComponentFragmentation" => Some(StatParameter::ComponentFragmentation),
            "ReservationName" => Some(StatParameter::ReservationName),
            "ReservationCapacity" => Some(StatParameter::ReservationCapacity),
            "ReservationWorkload" => Some(StatParameter::ReservationWorkload),
            "ReservationState" => Some(StatParameter::ReservationState),
            "ReservationProceeding" => Some(StatParameter::ReservationProceeding),
            "NumberOfJobs" => Some(StatParameter::NumberOfTasks),
            "Command" => Some(StatParameter::Command),
            "ProcessingTime" => Some(StatParameter::ProcessingTime),
            "FragmentationBefore" => Some(StatParameter::FragmentationBefore),
            "FragmentationAfter" => Some(StatParameter::FragmentationAfter),
            "NumberOfCoAllocationDependencies" => Some(StatParameter::NumberOfCoAllocationDependencies),
            "NumberOfDataDependencies" => Some(StatParameter::NumberOfDataDependencies),
            _ => None,
        }
    }

    /// Public API to log an event.
    /// This is non-blocking (just sends a message).
    pub fn add_event(&self, mut event: StatisticEvent) {
        // Inject timestamp automatically if not present
        if event.get(StatParameter::Time).is_none() {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            // Calculate relative time if needed, or just absolute
            let relative = now.saturating_sub(self.start_time);
            event.set(StatParameter::Time, relative as i64);
        }

        // Send to writer thread
        // We ignore errors here (e.g., if writer thread crashed) to not crash the simulation
        let _ = self.sender.send(StatsMessage::Log(event));
    }
}

static GLOBAL_STATS: OnceLock<StatsCollector> = OnceLock::new();

/// Initialize the global statistics collector.
pub fn init_global(filename: Option<String>) {
    let collector = StatsCollector::init(filename);
    let _ = GLOBAL_STATS.set(collector);
}

/// Helper to log an event to the global collector.
/// Safe to call from anywhere, from any thread.
pub fn add_global_event(event: StatisticEvent) {
    if let Some(collector) = GLOBAL_STATS.get() {
        collector.add_event(event);
    } else {
        log::error!("Warning: Statistics event dropped. Call init_global() first.");
    }
}

// fn main() {
//     init_global(Some("vrm_stats.csv".to_string()));

//     let mut component = MyVRMComponent { capacity: 100, load: 75.5 };
//     let event = component.generate_statistics();

//     add_global_event(event);
// }
