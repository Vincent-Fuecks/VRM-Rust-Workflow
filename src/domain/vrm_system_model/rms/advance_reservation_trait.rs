use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationKey};
use crate::domain::vrm_system_model::rms::rms::Rms;
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetrics;

use std::cmp::Ordering;

pub trait AdvanceReservationRms: Rms {
    fn get_fragmentation(&self, start: i64, end: i64, shadow_schedule_id: Option<&str>) -> f64 {
        todo!()
    }

    fn get_system_fragmentation(&self, shadow_schedule_id: Option<&str>) -> f64 {
        todo!()
    }

    fn get_load(&self, start: i64, end: i64, shadow_schedule_id: Option<&str>) -> LoadMetrics {
        todo!()
    }

    fn get_simulation_load(&self) -> LoadMetrics {
        todo!()
    }

    fn create_shadow_schedule(&mut self, shadow_schedule_id: String) {
        todo!()
    }

    fn commit_shadow_schedule(&mut self, shadow_schedule_id: String) -> bool {
        todo!()
    }

    fn rollback_shadow_schedule(&mut self, shadow_schedule_id: String) {
        todo!()
    }

    fn probe(&self, res: Box<dyn Reservation>, shadow_schedule_id: Option<&str>) -> Vec<Box<dyn Reservation>> {
        todo!()
    }

    fn probe_best(
        &self,
        res: Box<dyn Reservation>,
        comparator: fn(Box<dyn Reservation>, Box<dyn Reservation>) -> Ordering,
    ) -> Option<Box<dyn Reservation>> {
        todo!()
    }

    fn reserve(&mut self, res: Box<dyn Reservation>, shadow_schedule_id: Option<&str>) -> Box<dyn Reservation> {
        todo!("")
    }

    fn delete_job(&mut self, res: Box<dyn Reservation>, shadow_schedule_id: Option<&str>) -> Box<dyn Reservation> {
        todo!()
    }

    fn commit(&mut self, res: Box<dyn Reservation>) -> Box<dyn Reservation> {
        todo!()
    }
}

impl<T: Rms> AdvanceReservationRms for T {}
