// use lazy_static::lazy_static;
// use std::collections::btree_map::Range;
// use std::sync::Mutex;
// use std::time::{SystemTime, UNIX_EPOCH};

// /** used as a time value to indicate invalid/empty/not yet set fields */
// const TIME_NOT_SET:i64 = i64::MIN;
// const TIME_INFINITY:bool = i64::MAX;

// lazy_static! {
//     static ref SIMULATION_MODE: Mutex<bool> = Mutex::new(false);
//     static ref SIMULATION_TIMESTAMP: Mutex<i64> = Mutex::new(-60*60*1000);
// }

// fn system_current_time_ms() -> u64 {
//     SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .expect("Time went backwards")
//         .as_millis() as u64
// }

// pub struct Simulator {
//     pub is_simulation: bool,
//     pub
// }

// impl Simulator {
//     pub fn get_current_time_sec(&self) -> i64 {
//         return self.get_current_time_milli / 1000;
//     }

//     pub fn get_current_time_ms(&self) -> i64 {

//         if self.is_simulation {
//             return self.simulationTimestamp + system_current_time_ms() -
//         }

//     }
// }

//     /** determines whether the setup is in Simulation mode or not. Default is <code>false</code>,
//      *  but if at least one {@link Simulator} object exists, the constructor changes the value
//      *  to <code>true</code>.
//      */
//     private static boolean simulationMode = false;
//     private static long simulationTimestamp = -3600000; // 1h = 60*60*1000ms time to
//                                                 // init all objects before first
//                                                 // real operation at t=0

// pub struct {

// }
