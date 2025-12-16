use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationKey, ReservationState};
use crate::domain::vrm_system_model::reservation::reservations::Reservations;
use crate::domain::vrm_system_model::rms::rms::Rms;
use crate::domain::vrm_system_model::schedule;
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;

use std::cmp::Ordering;

/**
 * Direct interface to a local resource management system (RMS) which
 * is capable to make advance reservations.
 *
 * Each operation should to be forwarded directly to the RMS i.e. {@link #probe(Reservation, String)}
 * should only provide reservation candidates, which are at the moment available at the RMS. There
 * are four main methods:
 * <ul>
 * <li>{@link #probe(Reservation, String)} - ask for possible reservation candidates, but doesn't reserve anything
 * <li>{@link #reserve(Reservation, String)} - reserve a job, it should be now submitted at the real RMS
 * <li>{@link #commit(Reservation)} - the {@link AI} got at commit message, the reservation is fixed for the AI now and there should be no {@link #deleteJob(Reservation, String)}
 * call anymore.
 * <li>{@link #deleteJob(Reservation, String)} - cancels a job on the RMS. Should only happen after {@link #reserve(Reservation, String)} and before {@link #commit(Reservation)}, but
 * may also happen later, if the end user cancels the job.
 * </ul>
 *
 * There are two special implementations for simulation and resources without RMS: {@link NullRMS}
 * and {@link NullBroker}.
 *
 * The interface also provides support for shadow schedules. A shadow schedule is the copy of
 * actual booked schedule. All operations on the schedule should be performed virtually. This
 * means, the real RMS schedule has to remain be untouched. But all operations have to return
 * only successful, if the RMS would accept the reservation. The method {@link #commitShadowSchedule(String)}
 * is used to perform all changes and will fail, if the shadow schedule couldn't be submitted to
 * the real RMS.
 *
 * By convention each element has to provide a constructor
 * <code>AdvanceReservationRMS({@link AI}, {@link Element})</code>
 * getting the AI the RMS belongs to and a XML node with the RMS
 * configuration.
 *
 *
 */
pub trait AdvanceReservationRms: Rms {
    fn create_shadow_schedule(&mut self, shadow_schedule_id: &ReservationKey) {
        if self.get_shadow_schedule_keys().contains(shadow_schedule_id) {
            log::error!("Creating new shadow schedule is not possible because shadow schedule id ({}) does already exist", shadow_schedule_id);
            return;
        }

        let new_shadow_schedule = self.get_base_mut().schedule.clone_box();
        self.get_base_mut().shadow_schedules.insert(shadow_schedule_id.clone(), new_shadow_schedule);
    }

    fn commit_shadow_schedule(&mut self, shadow_schedule_id: &ReservationKey) -> bool {
        let new_schedule = self.get_base_mut().shadow_schedules.remove(shadow_schedule_id);

        if new_schedule.is_some() {
            self.get_base_mut().schedule = new_schedule.unwrap();
            return true;
        }

        log::error!("Finding and removing shadow schedule with id {} was not possible", shadow_schedule_id.clone());
        return false;
    }

    fn get_fragmentation(&mut self, start: i64, end: i64, shadow_schedule_id: ReservationKey) -> f64 {
        return self.get_mut_shadow_schedule(shadow_schedule_id).get_fragmentation(start, end);
    }

    fn get_system_fragmentation(&mut self, shadow_schedule_id: ReservationKey) -> f64 {
        return self.get_mut_shadow_schedule(shadow_schedule_id).get_system_fragmentation();
    }

    fn get_load_metric(&mut self, start: i64, end: i64, shadow_schedule_id: ReservationKey) -> LoadMetric {
        return self.get_mut_shadow_schedule(shadow_schedule_id).get_load_metric(start, end);
    }

    fn get_simulation_load_current_schedule(&mut self) -> LoadMetric {
        return self.get_base_mut().schedule.get_simulation_load_metric();
    }

    fn probe(&mut self, reservation_key: ReservationKey, shadow_schedule_id: ReservationKey) -> Reservations {
        return self.get_mut_shadow_schedule(shadow_schedule_id).probe(reservation_key);
    }

    fn reserve(&mut self, mut reservation: Box<dyn Reservation>, shadow_schedule_id: ReservationKey) -> Option<Box<dyn Reservation>> {
        let shadow_schedule = self.get_base_mut().shadow_schedules.get_mut(&shadow_schedule_id);

        match shadow_schedule {
            Some(shadow_schedule) => shadow_schedule.reserve(reservation),
            None => {
                reservation.set_state(ReservationState::Rejected);
                return Some(reservation);
            }
        }
    }

    // TODO is this right?
    fn commit(&mut self, mut reservation: Box<dyn Reservation>) -> Box<dyn Reservation> {
        log::info!("RmsNull committed reservation with id: {}. Please look at the implementation maybe it is wrong", reservation.get_id());

        reservation.set_state(ReservationState::Committed);
        return reservation;
    }

    fn delete_shadow_schedule(&mut self, shadow_schedule_id: ReservationKey) {
        if self.get_base_mut().shadow_schedules.remove(&shadow_schedule_id).is_none() {
            log::error!("Removing shadow schedule was not possible. Shadow schedule id ({}) was not found", shadow_schedule_id);
        }
    }

    fn probe_best<C>(&mut self, request_key: ReservationKey, mut comparator: C) -> Option<Box<dyn Reservation>>
    where
        C: FnMut(Box<dyn Reservation>, Box<dyn Reservation>) -> Ordering,
    {
        return self.get_base_mut().schedule.probe_best(request_key, &mut comparator);
    }

    fn delete_task(&mut self, reservation_key: ReservationKey, shadow_schedule_id: &ReservationKey) -> Option<Box<dyn Reservation>> {
        self.get_mut_shadow_schedule(shadow_schedule_id.clone()).delete_reservation(reservation_key);
        return None;
    }
}

impl<T: Rms> AdvanceReservationRms for T {}
