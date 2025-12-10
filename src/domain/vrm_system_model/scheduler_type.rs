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
    // pub fn get_instance(
    //     &self,
    //     name: String,
    //     number_of_slots: i64,
    //     slot_width: i64,
    //     capacity: i64,
    // ) -> Box<dyn Schedule> {
    //     match self {
    //         SchedulerType::FreeListSchedule => {
    //             todo!()
    //         }
    //         SchedulerType::SlottedSchedule => {
    //             todo!()
    //         }
    //         SchedulerType::SlottedSchedule12 => {
    //             todo!()
    //         }
    //         SchedulerType::SlottedSchedule12000 => {
    //             todo!()
    //         }
    //         SchedulerType::SlottedScheduleResubmitFrag => {
    //             todo!()
    //         }
    //         SchedulerType::UnlimitedSchedule => {
    //             todo!()
    //         }
    //     }
    // }

    // Returns a trait object; TODO Is this nessessary?
    // pub fn clone_box(&self) -> Box<dyn Schedule> {
    //     todo!()
    // }
}
