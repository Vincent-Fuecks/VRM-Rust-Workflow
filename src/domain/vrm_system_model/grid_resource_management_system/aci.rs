use crate::api::vrm_system_model_dto::aci_dto::AcIDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::grid_resource_management_system::grid_resource_management_system_trait::ExtendedReservationProcessor;
use crate::domain::vrm_system_model::grid_resource_management_system::reservation_submitter_trait::ReservationSubmitter;
use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationKey};
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::rms::advance_reservation_trait::AdvanceReservationRms;
use crate::domain::vrm_system_model::rms::rms::Rms;
use crate::domain::vrm_system_model::rms::rms_type::RmsType;
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId, RouterId, ShadowScheduleId};
use crate::error::ConversionError;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone)]
pub enum ScheduleID {
    FreeListSchedule,
    SlottedSchedule,
    SlottedScheduleResubmitFrag,
    SlottedSchedule12,
    SlottedSchedule12000,
    UnlimitedSchedule,
}

#[derive(Debug)]
pub struct ReservationContainer {
    // The ReservationSubmitter, which submitted the reservation.
    //owner: ReservationSubmitter,
    ///  Until which time the reservation has to be committed, if only reserved. VRM time in s.
    commit_deadline: i64,
    /// until which time the reservation has to finish execution. VRM time in s.
    execution_deadline: i64,
}

#[derive(Debug)]
struct ShadowScheduleReservations {
    inner_map: BTreeMap<ShadowScheduleId, ReservationContainer>,
}

#[derive(Debug)]
pub struct AcI {
    pub id: AciId,
    adc_ids: Vec<AdcId>,
    commit_timeout: i64,
    rms_system: Box<dyn AdvanceReservationRms>,
    shadow_schedule_reservations: ShadowScheduleReservations,
    not_committed_reservations: HashMap<ReservationKey, ReservationContainer>,

    simulator: Box<dyn SystemSimulator>,
    reservation_store: ReservationStore,
}

impl TryFrom<(AcIDto, Box<dyn SystemSimulator>)> for AcI {
    type Error = ConversionError;

    fn try_from(args: (AcIDto, Box<dyn SystemSimulator>)) -> Result<Self, ConversionError> {
        let (dto, simulator) = args;

        let aci_name = dto.id.clone();
        let adc_ids: Vec<AdcId> = dto.adc_ids.iter().map(|adc_id| AdcId::new(adc_id)).collect();

        // TODO Should be located in VRM
        let reservation_store: ReservationStore = ReservationStore::new(None);

        let rms_system = RmsType::get_instance(dto.rms_system, simulator, dto.id, reservation_store)?;

        Ok(AcI {
            id: AciId::new(aci_name),
            adc_ids,
            commit_timeout: dto.commit_timeout,
            rms_system,
            shadow_schedule_reservations: todo!(),
            not_committed_reservations: todo!(),
            simulator: simulator.clone_box(),
            reservation_store,
        })
        // TODO
        // start background worker thread
        // Simulator.start(this);
    }
}

impl ExtendedReservationProcessor for AcI {
    fn commit(&self, requester: Box<dyn ReservationSubmitter>, reservation: Box<dyn Reservation>) -> Box<dyn Reservation> {
        log::debug!("AcI: {} is committing the reservation: {}, of the requester: ", self.id, reservation.get_name());

        let arrival_time: i64 = self.simulator.get_current_time_in_ms();
        let reservation_id = reservation.get_name();

        // let reservation_container: Option<ReservationContainer> = self.not_committed_reservations.remove(&reservation_id);

        // match reservation_container {
        //     Some(reservation_container) => {}
        //     None => {
        //         Log::info("There was no reserve before the commit of reservation: {} was tried to allocate.", reservation.get_id());

        //         if
        //     }
        // }
        todo!()
    }

    fn commit_shadow_schedule(&self, shadow_schedule_id: String) -> bool {
        todo!()
    }

    fn create_shadow_schedule(&self, shadow_schedule_id: ShadowScheduleId) {
        todo!()
    }

    fn delete(
        &self,
        requester: Box<dyn ReservationSubmitter>,
        reservation: Box<dyn Reservation>,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) -> Box<dyn Reservation> {
        todo!()
    }

    fn get_load_metric(&mut self, start_time: i64, end_time: i64) -> crate::domain::vrm_system_model::utils::load_buffer::LoadMetric {
        todo!()
    }

    fn get_resources(&self) -> Vec<String> {
        todo!()
    }

    fn get_satisfaction(&self, start: u64, end: u64, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        todo!()
    }

    fn get_simulation_load_metric(&mut self) -> crate::domain::vrm_system_model::utils::load_buffer::LoadMetric {
        todo!()
    }

    fn get_system_satisfaction(&self, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        todo!()
    }

    fn probe(
        &self,
        requester: Box<dyn ReservationSubmitter>,
        reservation: Box<dyn Reservation>,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) -> Vec<Box<dyn Reservation>> {
        todo!()
    }

    fn probe_best<F>(&self, reservation: Box<dyn Reservation>, comparator: F) -> Option<Box<dyn Reservation>>
    where
        F: Fn(Box<dyn Reservation>, Box<dyn Reservation>) -> std::cmp::Ordering,
    {
        todo!()
    }

    fn reserve(
        &self,
        requester: Box<dyn ReservationSubmitter>,
        reservation: Box<dyn Reservation>,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) -> Box<dyn Reservation> {
        todo!()
    }

    fn rollback_shadow_schedule(&self, shadow_schedule_id: ShadowScheduleId) {
        todo!()
    }
}
