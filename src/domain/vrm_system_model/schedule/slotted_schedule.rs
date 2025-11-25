use crate::domain::vrm_system_model::schedule::slot::Slot;
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::scheduler_type::SchedulerType;
use crate::domain::vrm_system_model::utils::load_buffer::LoadBuffer;
use crate::domain::workflow::reservation::{Reservation, ReservationKey, ReservationState};
use crate::domain::workflow::reservations::Reservations;

use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

lazy_static! {
    static ref FRAGMENTATION_POWER: Mutex<i64> = Mutex::new(2);
}

pub struct SlottedSchedule {
    id: String,
    slots: Vec<Slot>,
    pub capacity: i64,
    slot_width: i64,
    start_slot_index: i64,
    end_slot_index: i64,
    scheduling_window_start_time: i64,
    scheduling_window_end_time: i64,
    load_buffer: LoadBuffer,
    accepted_reservations: Reservations,
    is_frag_cach_up_to_date: bool,
    fragmentation_cach: f64,
    quadratic_mean_fragmentation_calculation: bool,
    is_frag_needed: bool,
}

impl SlottedSchedule {
    /**
     *
     * @param name schedule name
     * @param numberOfRealSlots number of real slots used as scheduling window
     * @param slotWidth amount of time contained in one slot
     * @param capacity amount of resource "pieces" managed by this schedule
     */
    pub fn new(id: String, number_of_real_slots: i64, slot_width: i64, capacity: i64) -> Self {
        let mut slots: Vec<Slot> = Vec::with_capacity(number_of_real_slots);

        for i in 0..number_of_real_slots {
            slots[i] = Slot::new(capacity)
        }

        SlottedSchedule {
            id: id,
            slots: slots,
            capacity: capacity,
            slot_width: slot_width,
            start_slot_index: (),
            end_slot_index: (),
            scheduling_window_start_time: 0,
            scheduling_window_end_time: -1,
            load_buffer: LoadBuffer::new(&Self),
            accepted_reservations: (),
            is_frag_cach_up_to_date: true,
            fragmentation_cach: 0.0,
            quadratic_mean_fragmentation_calculation: true,
            is_frag_needed: false,
        }
    }

    /**
     * computes the index of the virtual time-slot containing the given point in time.
     * @param time in seconds
     * @return slot index, containing this point in time
     * This is a virtual index, so there can be less real slots in the schedule then this index imply.
     */
    pub fn get_slot_index(&self, time: i64) -> i64 {
        let index: i64 = (time as f64 / self.slot_width as f64).floor() as i64;

        if index < 0 {
            return 0;
        }

        return index;
    }

    /**
     * computes the real index of a slot in the slot-vector for an virtual slot index in the schedule
     * @param index index in the schedule.
     * This is a virtual index, so there can be less real slots in the schedule then the index imply.
     * @return the real index of a slot in the slot-vector
     */
    pub fn get_real_slot_index(&self, index: i64) {
        return (index % self.slots.len()) as i64;
    }

    /**
     * returns the Slot with the given index
     * @param index index for the searched slot.
     * This is a virtual index, so there can be less real slots in the schedule then this index imply.
     * @return Slot with the given index
     */
    pub fn get_slot(&self, index: i64) -> Option<Slot> {
        if index < 0 {
            return None;
        }

        if index < self.start_slot_index || index > (self.end_slot_index + 1) {
            return None;
        }

        let real_index: i64 = self.get_real_slot_index(index);
        return self.slots.get(real_index);
    }

    /**
     * computes the start time of a virtual slot in seconds.
     * @param index index of the slot we want to get the start time for,
     * This is a virtual index, so there can be less real slots in the schedule then the index imply.
     * @return start time of the slot with index slotIndex
     */
    pub fn get_slot_start_time(&self, index: i64) {
        return index * self.slot_width;
    }

    /**
     * computes the end time of a virtual slot in seconds.
     * @param index index of the slot we want to get the end time for
     * This is a virtual index, so there can be less real slots in the schedule then the index imply.
     * @return end time of the slot with index slotIndex
     */
    pub fn get_slot_end_time(&self, index: i64) {
        return index * self.slot_width + self.slot_width - 1;
    }
}

impl Schedule for SlottedSchedule {
    fn clear(&mut self) {
        log::warn!(
            "In SlottedSchedule id: {}, where all Slots cleared.",
            self.id
        );

        for mut slot in self.slots {
            slot.reset();
        }

        self.accepted_reservations.clear();
    }

    fn update(&mut self) {
        // TODO long currentTime = Simulator.getCurrentTimeSeconds();
        let current_time: i64 = 42;
        let new_start_slot_index = self.get_slot_index(current_time);

        if self.start_slot_index < new_start_slot_index {
            self.is_frag_cach_up_to_date = false;
        }

        let mut keys_to_remove: HashSet<ReservationKey> = HashSet::new();

        for clean_index in self.start_slot_index..new_start_slot_index {
            if let Some(slot) = self.get_slot(clean_index) {
                for key in &slot.reservation_keys {
                    if let Some(reservation) = self.accepted_reservations.get(key) {
                        let last_slot_of_reservation =
                            self.get_slot_index(reservation.get_assigned_end());

                        if last_slot_of_reservation == clean_index {
                            keys_to_remove.insert(key.clone());
                        }
                    }
                }
            }
        }

        for key in keys_to_remove {
            self.accepted_reservations.reservations.remove(&key)
        }

        for clean_index in self.start_slot_index..new_start_slot_index {
            if let Some(slot) = self.get_slot_mut(clean_index) {
                // TODO: this.loadBuffer.add(nextSlot.getLoad(), cleanIndex);

                slot.reset();
            }
        }

        self.start_slot_index = new_start_slot_index;
        self.end_slot_index = new_start_slot_index + self.slots.len() - 1;
        self.scheduling_window_start_time = self.get_slot_start_time(self.start_slot_index);
        self.scheduling_window_end_time = self.get_slot_end_time(self.end_slot_index);
    }

    fn delete_reservation(&mut self, reservation: &Reservation) -> Option<Reservation> {
        let key = ReservationKey { id: reservation.id };

        // Can not Del unreserved reservation
        if !self.accepted_reservations.reservations.contains_key(key) {
            log::error!(
                "DEL Reservatlion form Schedule: {}, However Schedule does not contain reservation with id: {}",
                self.id,
                reservation.get_id()
            );

            reservation.set_state(ReservationState::Rejected);
            return reservation;
        }
        // TODO
    }
}
