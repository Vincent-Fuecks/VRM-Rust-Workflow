use crate::api::vrm_system_model_dto::aci_dto::AcIDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::grid_resource_management_system::grid_resource_management_system_trait::ExtendedReservationProcessor;
use crate::domain::vrm_system_model::grid_resource_management_system::reservation_submitter_trait::ReservationSubmitter;
use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationState};
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::rms::advance_reservation_trait::AdvanceReservationRms;
use crate::domain::vrm_system_model::rms::rms::Rms;
use crate::domain::vrm_system_model::rms::rms_type::RmsType;
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId, RouterId, ShadowScheduleId};
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;
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
pub struct ReservationDeadlines {
    ///  Until which time the reservation has to be committed, if only reserved. VRM time in s.
    commit_deadline: i64,
    /// until which time the reservation has to finish execution. VRM time in s.
    execution_deadline: i64,
}

#[derive(Debug)]
struct ShadowScheduleReservations {
    inner_map: BTreeMap<ShadowScheduleId, ReservationDeadlines>,
}

#[derive(Debug)]
pub struct AcI {
    pub id: AciId,
    adc_ids: Vec<AdcId>,
    commit_timeout: i64,
    rms_system: Box<dyn AdvanceReservationRms>,
    shadow_schedule_reservations: ShadowScheduleReservations,
    not_committed_reservations: HashMap<ReservationId, ReservationDeadlines>,

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
    fn commit(&mut self, reservation_id: ReservationId) -> bool {
        log::debug!("AcI: {} is committing the reservation: {:?}, of the requester: ", self.id, reservation_id);

        let arrival_time: i64 = self.simulator.get_current_time_in_ms();

        match self.not_committed_reservations.remove(&reservation_id) {
            Some(reservation_deadlines) => {}
            None => {
                log::info!("There was no reservation before the commit of the reservation (id: {:?}), please allocate it.", reservation_id);

                if todo!() {
                    self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
                    AcI::log_stat("Commit".to_string(), reservation_id, arrival_time);
                    log::debug!(
                        "There was no reservation ({:?}) before the commit and the AcI ({}) can't handle the request.",
                        reservation_id,
                        self.id
                    );
                    return reservation_id;
                } else {
                    possible_reservation = self.rms_system.reserve(reservation_id, None)
                }
            }
        };
        todo!()
    }

    fn commit_shadow_schedule(&self, shadow_schedule_id: ShadowScheduleId) -> bool {
        todo!()
    }

    fn create_shadow_schedule(&self, shadow_schedule_id: ShadowScheduleId) {
        todo!()
    }

    fn delete(
        &self,
        requester: Box<dyn ReservationSubmitter>,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) -> ReservationId {
        todo!()
    }

    fn delete_shadow_schedule(&self, shadow_schedule_id: ShadowScheduleId) {
        todo!()
    }

    fn get_load_metric(&mut self, start_time: i64, end_time: i64) -> LoadMetric {
        todo!()
    }

    fn get_satisfaction(&self, start: u64, end: u64, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        todo!()
    }

    fn get_simulation_load_metric(&mut self) -> LoadMetric {
        todo!()
    }

    fn get_system_satisfaction(&self, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        todo!()
    }

    fn probe(
        &self,
        requester: Box<dyn ReservationSubmitter>,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) -> crate::domain::vrm_system_model::reservation::reservations::Reservations {
        todo!()
    }

    fn probe_best<F>(&self, reservation_id: ReservationId, comparator: F) -> Option<ReservationId>
    where
        F: Fn(ReservationId, ReservationId) -> std::cmp::Ordering,
    {
        todo!()
    }

    fn reserve(
        &self,
        requester: Box<dyn ReservationSubmitter>,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) -> ReservationId {
        todo!()
    }
}

impl AcI {
    /**
     * Creates a {@link StatisticEvent} with the result of the given command
     * and adds it to the {@link Statistics}.
     *
     * @param command
     *            The executed command, mainly "reserve", "commit", or "delete"
     * @param answer
     *            The result of the operation
     * @param arrivalTimeAtAI
     * 			  The time the processing of this reservation started as get by
     * 			  {@link Simulator#getCurrentTimeMillis()} i.e. ms since 1.1.1970
     * @see AI#logStatProbe(Reservation, int, String)
     * @see Statistics
     */
    pub fn log_stat(command: String, reservation_id: ReservationId, arrival_time_at_aci: i64) {
        todo!()
    }
}
