use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::api::vrm_system_model_dto::vrm_dto::SimulatorDto;

pub trait SystemSimulator: std::fmt::Debug + Send + Sync {
    fn get_current_time_in_s(&self) -> i64;
    fn get_current_time_in_ms(&self) -> i64;
    fn clone_box(&self) -> SharedSimulator;
}

#[derive(Debug)]
pub struct SharedSimulator(pub Arc<dyn SystemSimulator>);

impl Clone for SharedSimulator {
    fn clone(&self) -> Self {
        self.0.clone_box()
    }
}

impl std::ops::Deref for SharedSimulator {
    type Target = dyn SystemSimulator;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[derive(Debug)]
struct SimulatorState {
    end_time: i64,
    is_simulation: bool,
    simulation_base_timestamp: i64,
    real_time_base_timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct Simulator {
    state: Arc<Mutex<SimulatorState>>,
}

impl Simulator {
    pub fn new(dto: SimulatorDto) -> Simulator {
        let current_real_time = Self::get_system_time_ms();

        let simulation_base_timestamp = 0;

        let state = SimulatorState {
            end_time: dto.end_time,
            is_simulation: dto.is_simulation,
            simulation_base_timestamp,
            real_time_base_timestamp: current_real_time,
        };

        Simulator { state: Arc::new(Mutex::new(state)) }
    }

    fn get_system_time_ms() -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_millis() as i64
    }

    pub fn run(&self) {
        let end_time = {
            let state = self.state.lock().unwrap();
            state.end_time
        };

        if end_time == i64::MIN || end_time == i64::MAX {
            loop {
                thread::park();
            }
        }

        loop {
            let current_time_s = self.get_current_time_in_s();

            if current_time_s >= end_time {
                break;
            }

            let wait_seconds = end_time - current_time_s;
            thread::sleep(Duration::from_secs(wait_seconds as u64));
        }

        println!("*** Simulation ended.");
    }
}

impl SystemSimulator for Simulator {
    fn get_current_time_in_s(&self) -> i64 {
        self.get_current_time_in_ms() / 1000
    }

    fn get_current_time_in_ms(&self) -> i64 {
        let state = self.state.lock().unwrap();

        if state.is_simulation {
            let current_real = Self::get_system_time_ms();
            state.simulation_base_timestamp + (current_real - state.real_time_base_timestamp)
        } else {
            Self::get_system_time_ms()
        }
    }

    fn clone_box(&self) -> SharedSimulator {
        SharedSimulator(Arc::new(self.clone()))
    }
}

impl From<SharedSimulator> for Arc<dyn SystemSimulator> {
    fn from(wrapper: SharedSimulator) -> Self {
        wrapper.0
    }
}
