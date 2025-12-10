use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;
use crate::domain::vrm_system_model::schedule::slotted_schedule::SlottedSchedule;
use crate::domain::vrm_system_model::scheduler_trait::Schedule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerType {
    FreeListSchedule,
    SlottedSchedule,
    SlottedScheduleResubmitFrag,
    SlottedSchedule12,
    SlottedSchedule12000,
    UnlimitedSchedule,
}

impl SchedulerType {
    // Factory method to create a concrete Schedule implementation
    pub fn get_instance(
        &self,
        id: ReservationKey,
        number_of_slots: i64,
        slot_width: i64,
        capacity: i64,
        simulator: Box<dyn SystemSimulator>,
    ) -> Box<dyn Schedule> {
        let use_quadratic_mean_fragmentation = true;

        match self {
            SchedulerType::FreeListSchedule => {
                todo!()
            }
            SchedulerType::SlottedSchedule => {
                Box::new(SlottedSchedule::new(id, number_of_slots, slot_width, capacity, use_quadratic_mean_fragmentation, simulator))
            }

            SchedulerType::SlottedSchedule12 => {
                let number_of_real_slots = (number_of_slots * (slot_width + 11)) / 12;
                Box::new(SlottedSchedule::new(id, number_of_real_slots, 12, capacity, use_quadratic_mean_fragmentation, simulator))
            }
            SchedulerType::SlottedSchedule12000 => {
                let number_of_real_slots = (number_of_slots * (slot_width + 11999)) / 12000;
                Box::new(SlottedSchedule::new(id, number_of_real_slots, 12000, capacity, use_quadratic_mean_fragmentation, simulator))
            }
            SchedulerType::SlottedScheduleResubmitFrag => Box::new(SlottedSchedule::new(id, number_of_slots, slot_width, capacity, false, simulator)),
            SchedulerType::UnlimitedSchedule => {
                todo!()
            }
        }
    }

    // Returns a trait object; TODO Is this necessary?
    pub fn clone_box(&self) -> Box<dyn Schedule> {
        todo!()
    }
}
