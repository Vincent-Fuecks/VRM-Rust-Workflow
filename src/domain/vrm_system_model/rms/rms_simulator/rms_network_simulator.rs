use crate::api::rms_config_dto::rms_dto::DummyRmsDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationTrait};
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::resource::resource_store::ResourceStore;
use crate::domain::vrm_system_model::rms::advance_reservation_trait::AdvanceReservationRms;
use crate::domain::vrm_system_model::rms::rms::{Rms, RmsBase, RmsLoadMetric};
use crate::domain::vrm_system_model::schedule::slotted_schedule::network_slotted_schedule::topology::NetworkTopology;
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::scheduler_type::{ScheduleContext, SchedulerType};
use crate::domain::vrm_system_model::utils::id::{AciId, ShadowScheduleId, SlottedScheduleId};
use crate::error::ConversionError;
use std::any::Any;
use std::collections::HashMap;
use std::i64;
use std::str::FromStr;
use std::sync::Arc;

/// Only simulates a cluster with Links (Nodes are not simulated)
#[derive(Debug)]
pub struct RmsNetworkSimulator {
    pub base: RmsBase,
    pub network_schedule: Box<dyn Schedule>,
    pub network_shadow_schedule: HashMap<ShadowScheduleId, Box<dyn Schedule>>,
}

impl Rms for RmsNetworkSimulator {
    fn get_base(&self) -> &RmsBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut RmsBase {
        &mut self.base
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_mut_active_schedule(&mut self, shadow_schedule_id: Option<ShadowScheduleId>, reservation_id: ReservationId) -> &mut Box<dyn Schedule> {
        if self.base.reservation_store.is_node(reservation_id) {
            match shadow_schedule_id {
                Some(id) => self.network_shadow_schedule.get_mut(&id).expect("node_shadow_schedule contains ShadowSchedule."),
                None => &mut self.network_schedule,
            }
        } else {
            panic!(
                "RmsSimulatorErrorNoScheduleForReservation: The rms RmsSimulator has no Scheduler for Reservation type {:?}. ReservationName: {:?} ShadowScheduleId {:?}",
                self.base.reservation_store.get_type(reservation_id),
                self.base.reservation_store.get_name_for_key(reservation_id),
                shadow_schedule_id
            );
        }
    }
}

impl TryFrom<(DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)> for RmsNetworkSimulator {
    type Error = ConversionError;

    fn try_from(args: (DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)) -> Result<Self, Self::Error> {
        let (dto, simulator, aci_id, reservation_store) = args;
        let (nodes, links) = RmsBase::get_nodes_and_links(&dto);
        let resource_store = ResourceStore::new();

        // Adds Links to Resource Store
        let topology = NetworkTopology::new(
            &links,
            &nodes,
            dto.slot_width,
            dto.num_of_slots,
            simulator.clone(),
            aci_id.clone(),
            reservation_store.clone(),
            resource_store.clone(),
        );

        let name = format!("AcI: {}, RmsType: {}", aci_id, dto.typ);
        let schedule_context = ScheduleContext {
            id: SlottedScheduleId::new(name.clone()),
            number_of_slots: dto.num_of_slots,
            slot_width: dto.slot_width,
            capacity: i64::MAX,
            simulator: simulator.clone(),
            reservation_store: reservation_store.clone(),
        };

        let mut scheduler_type = SchedulerType::from_str(&dto.scheduler_typ)?;
        scheduler_type = scheduler_type.get_network_scheduler_variant(topology, resource_store.clone());
        let network_schedule = scheduler_type.get_instance(schedule_context);

        let base = RmsBase::new(aci_id, dto.typ, reservation_store, resource_store.clone());

        Ok(RmsNetworkSimulator { base, network_schedule, network_shadow_schedule: HashMap::new() })
    }
}

impl AdvanceReservationRms for RmsNetworkSimulator {
    fn create_shadow_schedule(&mut self, shadow_schedule_id: &ShadowScheduleId) -> bool {
        if self.network_shadow_schedule.contains_key(shadow_schedule_id) {
            log::error!(
                "Creating new shadow schedule is not possible because shadow schedule id ({}) does already exist. Please first delete the old shadow schedule.",
                shadow_schedule_id
            );
            return false;
        }

        if self.network_shadow_schedule.insert(shadow_schedule_id.clone(), self.network_schedule.clone_box()).is_none() {
            log::error!("ErrorShadowScheduleAlreadyExists: ShadowSchedule is now curupted.");
            return false;
        }

        return true;
    }

    fn commit_shadow_schedule(&mut self, shadow_schedule_id: &ShadowScheduleId) -> bool {
        if self.network_shadow_schedule.contains_key(shadow_schedule_id) {
            let new_network_schedule = self.network_shadow_schedule.remove(shadow_schedule_id);

            if !new_network_schedule.is_none() {
                self.network_schedule = new_network_schedule.unwrap();
                return true;
            }
        }

        log::error!("Finding and removing of shadow schedule with id {} was not possible", shadow_schedule_id.clone());
        return false;
    }

    fn delete_shadow_schedule(&mut self, shadow_schedule_id: &ShadowScheduleId) -> bool {
        if self.network_shadow_schedule.contains_key(shadow_schedule_id) {
            let removed_network_schedule = self.network_shadow_schedule.remove(shadow_schedule_id);

            if removed_network_schedule.is_none() {
                return true;
            }
        }

        log::error!("Removing shadow schedule was not possible. Shadow schedule id ({}) was not found", shadow_schedule_id.clone());
        return false;
    }

    fn get_fragmentation(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        match shadow_schedule_id {
            Some(id) => {
                self.network_shadow_schedule.get_mut(&id).expect("network_shadow_schedule contains ShadowSchedule.").get_fragmentation(start, end)
            }
            None => self.network_schedule.get_fragmentation(start, end),
        }
    }

    fn get_system_fragmentation(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        match shadow_schedule_id {
            Some(id) => {
                self.network_shadow_schedule.get_mut(&id).expect("network_shadow_schedule contains ShadowSchedule.").get_system_fragmentation()
            }
            None => self.network_schedule.get_system_fragmentation(),
        }
    }

    fn can_handle_adc_request(&self, res: Reservation) -> bool {
        if res.is_link() {
            return self.get_base().resource_store.can_handle_adc_request(res);
        }

        log::debug!(
            "The rms {:?} can not process Reservations of Type {:?} (ReservationName: {:?}) the rms can only process LinkReservations.",
            self.base.id,
            res.get_type(),
            res.get_name()
        );
        return false;
    }

    fn can_handle_aci_request(&self, reservation_store: ReservationStore, reservation_id: ReservationId) -> bool {
        if reservation_store.is_link(reservation_id) {
            return self.get_base().resource_store.can_handle_aci_request(reservation_store, reservation_id);
        }

        log::debug!(
            "The rms {:?} can not process Reservations of Type {:?} (ReservationName: {:?}) the rms can only process LinkReservations.",
            self.base.id,
            reservation_store.get_type(reservation_id),
            reservation_store.get_name_for_key(reservation_id)
        );
        return false;
    }

    fn get_load_metric(&self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> RmsLoadMetric {
        match shadow_schedule_id {
            Some(id) => RmsLoadMetric {
                node_load_metric: Some(
                    self.network_shadow_schedule.get(&id).expect("network_shadow_schedule contains ShadowSchedule.").get_load_metric(start, end),
                ),
                link_load_metric: None,
            },
            None => RmsLoadMetric { node_load_metric: Some(self.network_schedule.get_load_metric(start, end)), link_load_metric: None },
        }
    }

    fn get_load_metric_up_to_date(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> RmsLoadMetric {
        match shadow_schedule_id {
            Some(id) => RmsLoadMetric {
                node_load_metric: Some(
                    self.network_shadow_schedule
                        .get_mut(&id)
                        .expect("network_shadow_schedule contains ShadowSchedule.")
                        .get_load_metric_up_to_date(start, end),
                ),
                link_load_metric: None,
            },
            None => RmsLoadMetric { node_load_metric: Some(self.network_schedule.get_load_metric_up_to_date(start, end)), link_load_metric: None },
        }
    }

    fn get_simulation_load_metric(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> RmsLoadMetric {
        match shadow_schedule_id {
            Some(id) => RmsLoadMetric {
                node_load_metric: Some(
                    self.network_shadow_schedule.get_mut(&id).expect("network_shadow_schedule contains ShadowSchedule.").get_simulation_load_metric(),
                ),
                link_load_metric: None,
            },
            None => RmsLoadMetric { node_load_metric: Some(self.network_schedule.get_simulation_load_metric()), link_load_metric: None },
        }
    }
}
