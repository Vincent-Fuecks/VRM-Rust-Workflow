use std::sync::{
    Arc,
    atomic::{AtomicI64, Ordering},
};

use crate::domain::vrm_system_model::schedule::slotted_schedule::SlottedSchedule;

/// The number of slots to discard from the beginning of the measurement interval.
/// Acts as a "warm-up" period to avoid skewing data with initial system ramp-up.
const SLOTS_TO_DROP_ON_START: i64 = 50;

/// The number of slots to discard from the end of the measurement interval.
/// Acts as a "cool-down" period to ensure the tail of the simulation data is valid and consistent with the ring buffer logic.
const SLOTS_TO_DROP_ON_END: i64 = 50;

/// This context acts as a thread-safe synchronization point
/// to determine the global start and end times of activity across the AcI.
/// TODO Is AcI right?
pub struct GlobalLoadContext {
    /// The earliest slot index where load was observed globally.
    first_global_load: AtomicI64,

    /// The latest slot index where load was observed globally.
    last_global_load: AtomicI64,
}

impl GlobalLoadContext {
    pub fn new() -> Self {
        Self {
            first_global_load: AtomicI64::new(i64::MAX),
            last_global_load: AtomicI64::new(-1),
        }
    }

    /// Updates the global first load index if the provided index is smaller.
    pub fn update_first_load(&self, index: i64) {
        self.first_global_load.fetch_min(index, Ordering::SeqCst);
    }

    /// Updates the global last load index if the provided index is larger.
    pub fn update_last_load(&self, index: i64) {
        self.last_global_load.fetch_max(index, Ordering::SeqCst);
    }

    /// Returns the current global first load index.
    pub fn get_first_load(&self) -> i64 {
        self.first_global_load.load(Ordering::SeqCst)
    }

    /// Returns the current global last load index.
    pub fn get_last_load(&self) -> i64 {
        self.last_global_load.load(Ordering::SeqCst)
    }
}

/// Data Transfer Object (DTO) containing the calculated utilization metrics.
///
/// This struct represents the final "cut" and processed view of the resource usage,
/// excluding warm-up and cool-down periods.
#[derive(Debug, Clone)]
pub struct LoadMetrics {
    /// The timestamp representing the start of the valid data interval.
    pub start_time: i64,

    /// The timestamp representing the end of the valid data interval.
    pub end_time: i64,

    /// The average capacity reserved per slot during the valid interval.
    pub avg_reserved_capacity: f64,

    /// The maximum possible capacity of the SlottedSchedule.
    pub possible_capacity: f64,

    /// The ratio of average reserved capacity to possible capacity (0.0 to 1.0).
    pub utilization: f64,
}

/// A circular buffer implementation used to track resource capacity usage.
///
/// The `LoadBuffer` records load events over time and calculates utilization metrics.
/// It interacts with a [`GlobalLoadContext`] to synchronize the valid time window across
/// multiple resources, ensuring that metrics are calculated over a consistent global timeframe.
pub struct LoadBuffer {
    /// Shared state for synchronizing start/end times across the system.
    context: Arc<GlobalLoadContext>,

    /// The schedule definition providing capacity and time slot mappings.
    schedule: SlottedSchedule,

    /// The slot index of the last load added to this specific buffer.
    slot_index_last_local_load: i64,

    /// Counter for empty slots in a row since the last non-empty load.
    slots_since_last_load: i64,

    /// Accumulator for the total reserved capacity seen so far.
    sum_reserved_capacity: i64,

    /// Ring buffer storing recent load values.
    log_buffer: Vec<i64>,

    /// Current index in the ring buffer (where the next element will be inserted).
    position_in_buffer: usize,

    /// Current count of valid items in the ring buffer (up to log_buffer capacity).
    items_in_buffer: usize,
}

impl LoadBuffer {
    pub fn new(schedule_to_use: SlottedSchedule, context: Arc<GlobalLoadContext>) -> Self {
        LoadBuffer {
            context,
            schedule: schedule_to_use,
            slot_index_last_local_load: 0,
            slots_since_last_load: 0,
            sum_reserved_capacity: 0,
            log_buffer: vec![0; SLOTS_TO_DROP_ON_END as usize],
            position_in_buffer: 0,
            items_in_buffer: 0,
        }
    }

    fn signal_occurrence_of_load(&self, slot_index: i64) {
        if slot_index < 0 {
            self.context.update_first_load(0);
            return;
        }

        self.context.update_first_load(slot_index);
        self.context.update_last_load(slot_index);
    }

    fn add_intern(&mut self, load: i64) {
        self.sum_reserved_capacity += load;

        self.log_buffer[self.position_in_buffer] = load;
        self.position_in_buffer = (self.position_in_buffer + 1) % SLOTS_TO_DROP_ON_END as usize;

        if self.items_in_buffer < (SLOTS_TO_DROP_ON_END as usize) {
            self.items_in_buffer += 1;
        }
    }

    /// Records a load event at a specific slot index.
    ///
    /// If the load is `<= 0`, it is treated as an empty slot and buffered until
    /// the next positive load event occurs. This ensures that trailing empty slots
    /// are not counted unless they are followed by activity.
    ///
    /// # Arguments
    ///
    /// * `next_slot_load` - The amount of capacity reserved (<= 0 indicates idle).
    /// * `slot_index` - The time index where this load occurred.
    pub fn add(&mut self, next_slot_load: i64, slot_index: i64) {
        // Just count empty slots. We will add them if any load occurred.
        if next_slot_load <= 0 {
            self.slots_since_last_load += 1;
        } else {
            // Load occurred
            self.signal_occurrence_of_load(slot_index);
            self.slot_index_last_local_load = slot_index;

            let slots_since_first_load = slot_index - self.context.get_first_load();

            if slots_since_first_load > SLOTS_TO_DROP_ON_START {
                for _ in 0..self.slots_since_last_load {
                    // Add waiting empty slots
                    self.add_intern(0);
                }
                // Add new load
                self.add_intern(next_slot_load);
            }

            self.slots_since_last_load = 0;
        }
    }

    fn get_cutted_start_time(&self) -> i64 {
        let index_of_first_slot = self.context.get_first_load() + SLOTS_TO_DROP_ON_START;

        return self.schedule.get_slot_start_time(index_of_first_slot);
    }

    fn get_cutted_end_time(&self) -> i64 {
        let index_of_last_slot = self.context.get_last_load() - SLOTS_TO_DROP_ON_END;

        return self.schedule.get_slot_start_time(index_of_last_slot);
    }

    /// Computes the final load metrics for the measured interval.
    ///
    /// This method finalizes the calculation by:
    /// 1. Synchronizing this buffer's state up to the global last load time.
    /// 2. Dropping the "tail" load (the data currently in the ring buffer) to strictly enforce the cool-down period.
    /// 3. Calculating the average utilization.
    pub fn get_cutted_overall_load(&mut self) -> LoadMetrics {
        // 1. Sync buffer to global end time
        let empty_slots_to_add: i64 =
            self.context.get_last_load() - self.slot_index_last_local_load;

        for _ in 0..empty_slots_to_add {
            self.add_intern(0);
        }

        self.slot_index_last_local_load = self.context.get_last_load();

        // 2. Calculate load inside buffer (Tail drop)
        let mut buffer_reserved_capacity: i64 = 0;
        let buf_len = SLOTS_TO_DROP_ON_END as usize;

        for i in 1..=self.items_in_buffer {
            let next_index = if self.position_in_buffer >= i {
                self.position_in_buffer - i
            } else {
                self.position_in_buffer + buf_len - i
            };

            buffer_reserved_capacity += self.log_buffer[next_index];
        }

        let mut reserved_capacity = self.sum_reserved_capacity - buffer_reserved_capacity;

        // 3. Calculate Interval
        let index_of_first_slot: i64 = self.context.get_first_load() + SLOTS_TO_DROP_ON_START;
        let index_of_last_slot: i64 = self.context.get_last_load() - SLOTS_TO_DROP_ON_END;
        let number_of_logged_slots = index_of_last_slot - index_of_first_slot;

        let avg_reserved_capacity;
        if number_of_logged_slots <= 0 {
            avg_reserved_capacity = 0.0;
        } else {
            avg_reserved_capacity = (reserved_capacity as f64) / (number_of_logged_slots as f64);
        }

        LoadMetrics {
            start_time: self.get_cutted_start_time(),
            end_time: self.get_cutted_end_time(),
            avg_reserved_capacity: avg_reserved_capacity,
            possible_capacity: self.schedule.capacity,
            utilization: if self.schedule.capacity != 0.0 {
                avg_reserved_capacity / self.schedule.capacity
            } else {
                0.0
            },
        }
    }
}
