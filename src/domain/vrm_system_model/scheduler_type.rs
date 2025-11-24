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
    /// Factory method to create a concrete Schedule implementation
    pub fn get_instance(
        &self,
        name: String,
        number_of_slots: i64,
        slot_width: i64,
        capacity: i64,
    ) -> Box<dyn Schedule> {
        match self {
            SchedulerType::FreeListSchedule => {
                // Box::new(FreeListSchedule::new(...))
                todo!()
            }
            SchedulerType::SlottedSchedule => {
                todo!()
                // Box::new(SlottedSchedule::new(
                // name,
                // number_of_slots,
                // slot_width,
                // capacity,
                // )),
            }
            SchedulerType::SlottedSchedule12 => {
                todo!()
                // let scaled_slots = (number_of_slots * (slot_width + 11)) / 12;
                // Box::new(SlottedSchedule::new(name, scaled_slots, 12, capacity))
            }
            SchedulerType::SlottedSchedule12000 => {
                todo!()
                // let scaled_slots = (number_of_slots * (slot_width + 11999)) / 12000;
                // Box::new(SlottedSchedule::new(name, scaled_slots, 12000, capacity))
            }
            SchedulerType::SlottedScheduleResubmitFrag => {
                todo!()
                // let mut result = SlottedSchedule::new(name, number_of_slots, slot_width, capacity);
                // result.set_quadratic_mean_fragmentation_calculation(false);
                // Box::new(result)
            }
            SchedulerType::UnlimitedSchedule => {
                // Return a concrete UnlimitedSchedule struct
                // Box::new(UnlimitedSchedule::new(...))
                todo!()
            }
        }
    }

    /// Returns a trait object; TODO Is this nessessary?
    pub fn clone_box(&self) -> Box<dyn Schedule> {
        todo!()
    }
}
