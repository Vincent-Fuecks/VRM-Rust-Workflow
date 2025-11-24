use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::domain::vrm_system_model::schedule::slotted_schedule::SlottedSchedule;

const SLOTS_TO_DROP_ON_START: i64 = 50;
const SLOTS_TO_DROP_ON_END: i64 = 50;

lazy_static! {
    static ref SLOT_INDEX_FIRST_GLOBAL_LOAD: Mutex<i64> = Mutex::new(i64::MAX);
    static ref SLOT_INDEX_LAST_GLOBAL_LOAD: Mutex<i64> = Mutex::new(-1);
}

pub struct LoadMetrics {
    pub start: i64,
    pub end: i64,
    pub avg_reserved_capacity: f64,
    pub possible_capacity: f64,
    pub utilization: f64,
}

pub struct LoadBuffer {
    pub load_metrices: LoadMetrics,
    pub schedule: SlottedSchedule,
    pub slot_index_last_local_load: i64,
    pub slots_since_last_load: i64,
    pub sum_reserved_capacity: i64,
    pub log_buffer: Vec<i64>,
    pub position_in_buffer: i64,
    pub items_in_buffer: i64,
}

impl LoadBuffer {
    pub fn new(schedule_to_use: &SlottedSchedule) {
        let load_metrices = LoadMetrics { 
            start: ,
            end: ,
            avg_reserved_capacity: ,
            possible_capacity: ,
            utilization: ,
        };

        LoadBuffer {
            load_metrices: load_metrices,
            schedule: SlottedSchedule,
            slot_index_last_local_load: i64,
            slots_since_last_load: i64,
            sum_reserved_capacity: i64,
            log_buffer: Vec<i64>,
            position_in_buffer: i64,
            items_in_buffer: i64,
        }
    }
}