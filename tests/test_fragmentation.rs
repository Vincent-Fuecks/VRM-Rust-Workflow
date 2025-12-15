use vrm_rust_workflow::domain::{
    simulator::simulator::{Simulator, SystemSimulator},
    vrm_system_model::{reservation::reservation::ReservationKey, schedule::slotted_schedule::SlottedSchedule, utils::id::SlottedScheduleId},
};

#[derive(Debug, Clone)]
pub struct MockSystemSimulator {
    is_simulation: bool,
}

impl SystemSimulator for MockSystemSimulator {
    fn get_current_time_in_s(&self) -> i64 {
        0
    }

    // Required method to enable cloning of the trait object
    fn clone_box(&self) -> Box<dyn SystemSimulator> {
        Box::new(self.clone())
    }
}

impl MockSystemSimulator {
    pub fn new(is_simulation: bool) -> MockSystemSimulator {
        MockSystemSimulator { is_simulation: is_simulation }
    }
}

#[test]
fn test_complex_fragmentation_scenario() {
    // Setup
    let capacity = 3;
    // Create schedule with enough slots (indices 0 to 3 require 4 slots)

    let simulator: Box<dyn SystemSimulator> = Box::new(MockSystemSimulator::new(true));

    let mut schedule = SlottedSchedule::new(SlottedScheduleId::new("Test-SlottedSchedule"), 4, 2, capacity, true, simulator);

    // Define loads
    schedule.set_slot_load(0, 0); // Free 3
    schedule.set_slot_load(1, 1); // Free 2
    schedule.set_slot_load(2, 2); // Free 1
    schedule.set_slot_load(3, 0); // Free 3

    // Execution
    let result = schedule.get_fragmentation_quadratic_mean(0, 3);

    // Verification
    // Level 1: 0.0
    let expected_level_1 = 0.0;
    // Level 2: 1.0 - (5.0 / 9.0) ~ 0.4444
    let expected_level_2 = 1.0 - (5.0 / 9.0);
    // Level 3: 1.0 - (2.0 / 4.0) = 0.5
    let expected_level_3 = 0.5;

    let expected_average = (expected_level_1 + expected_level_2 + expected_level_3) / 3.0;

    // Assert with epsilon for floating point comparison
    assert!((result - expected_average).abs() < 0.0001, "Complex fragmentation calculation failed. Expected {}, got {}", expected_average, result);
}

/// TEST 2: Zero Fragmentation (Perfectly Free)
#[test]
fn test_zero_fragmentation_all_free() {
    let capacity = 5;
    let simulator: Box<dyn SystemSimulator> = Box::new(MockSystemSimulator::new(true));
    // 10 slots
    let schedule = SlottedSchedule::new(SlottedScheduleId::new("Test-SlottedSchedule-02"), 10, 2, capacity, true, simulator);

    let result = schedule.get_fragmentation_quadratic_mean(0, 9);

    assert!(result.abs() < 0.0001, "Perfectly free schedule should have 0 fragmentation, got {}", result);
}

/// TEST 3: Full Load (No Availability)
#[test]
fn test_zero_fragmentation_full_load() {
    let capacity = 4;
    let simulator: Box<dyn SystemSimulator> = Box::new(MockSystemSimulator::new(true));

    let mut schedule = SlottedSchedule::new(SlottedScheduleId::new("Test-SlottedSchedule-02"), 5, 2, capacity, true, simulator);

    // Set load equal to capacity for all slots
    for i in 0..5 {
        schedule.set_slot_load(i, capacity);
    }

    let result = schedule.get_fragmentation_quadratic_mean(0, 4);

    assert!(result.abs() < 0.0001, "Fully loaded schedule should have 0 fragmentation, got {}", result);
}
