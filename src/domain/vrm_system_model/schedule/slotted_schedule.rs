use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use crate::domain::vrm_system_model::reservation::reservations::Reservations;
use crate::domain::vrm_system_model::schedule::slot::Slot;
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::utils::id::SlottedScheduleId;
use crate::domain::vrm_system_model::utils::{
    load_buffer::{GlobalLoadContext, LoadBuffer},
    vrm_component_trait::VRMComponent,
};

use std::i64;
use std::sync::Arc;

pub mod core_functions;
pub mod fragmentation;
pub mod schedule_trait;

#[derive(Debug, Clone)]
pub struct SlottedSchedule {
    /// **Unique identifier** for this SlottedSchedule.
    id: SlottedScheduleId,

    /// A list of all time **Slots** defined for this schedule.
    slots: Vec<Slot>,

    /// The maximum total amount of resource "pieces" (e.g., cores, units) managed by this schedule.
    pub capacity: i64,

    /// The duration of a single time slot.
    slot_width: i64,

    /// The index of the earliest possible slot that can be used for scheduling.
    start_slot_index: i64,

    /// The index of the latest possible slot that defines the scheduling window's end.
    end_slot_index: i64,

    /// The **absolute start time** (e.g., Unix timestamp) of the current scheduling window being viewed.
    scheduling_window_start_time: i64,

    /// The **absolute end time** (e.g., Unix timestamp) of the current scheduling window being viewed.
    scheduling_window_end_time: i64,

    /// Internal buffer used to track transient or potential resource load.
    load_buffer: LoadBuffer,

    /// A map of all currently **active reservations** associated with this schedule.
    active_reservations: Reservations,

    /// Flag indicating if the stored **fragmentation_cache** value is valid and up-to-date.
    is_frag_cache_up_to_date: bool,

    /// The cached value of the system **fragmentation**.
    fragmentation_cache: f64,

    /// A configuration flag to determine if the system should utilize the **quadratic mean**
    /// or the standard formula for fragmentation calculation.
    use_quadratic_mean_fragmentation: bool,

    /// A flag indicating whether fragmentation calculation is required for the **prob requests**.
    is_frag_needed: bool,

    simulator: Arc<dyn SystemSimulator>,
}

impl SlottedSchedule {
    pub fn new(
        id: SlottedScheduleId,
        number_of_real_slots: i64,
        slot_width: i64,
        capacity: i64,
        use_quadratic_mean_fragmentation: bool,
        simulator: Arc<dyn SystemSimulator>,
        reservation_store: ReservationStore,
    ) -> Self {
        let mut slots: Vec<Slot> = Vec::new();

        // number_of_real_slots is the number of slots in the considered scheduling window
        for _ in 0..number_of_real_slots {
            slots.push(Slot::new(capacity));
        }

        let mut slotted_schedule: SlottedSchedule = SlottedSchedule {
            id: SlottedScheduleId::new(id),
            slots: slots,
            capacity: capacity,
            slot_width: slot_width,
            start_slot_index: 0,
            end_slot_index: -1,
            scheduling_window_start_time: 0,
            scheduling_window_end_time: -1,
            load_buffer: LoadBuffer::new(Arc::new(GlobalLoadContext::new())),
            active_reservations: Reservations::new_empty(reservation_store),
            is_frag_cache_up_to_date: true,
            fragmentation_cache: 0.0,
            use_quadratic_mean_fragmentation: use_quadratic_mean_fragmentation,
            // TODO Always false
            is_frag_needed: false,
            simulator: simulator,
        };

        slotted_schedule.update();

        return slotted_schedule;
    }
}

impl VRMComponent for SlottedSchedule {
    fn generate_statistics(&mut self) {
        todo!()
        // let load_metrics =
        //     self.load_buffer.get_effective_overall_load(self.capacity as f64, self.get_effective_start_time(), self.get_effective_end_time());

        // let mut event = StatisticEvent::new();
        // event.set(StatParameter::ComponentUtilization, load_metrics.utilization);
        // event
        //     .set(StatParameter::ReservationWorkload, load_metrics.avg_reserved_capacity * ((load_metrics.start_time - load_metrics.end_time) as f64));

        // event.set(StatParameter::ComponentCapacity, self.capacity);

        // return event;
    }
}
