use std::{any::Any, collections::HashMap, str::FromStr, sync::Arc};

use crate::{
    api::rms_config_dto::rms_dto::DummyRmsDto,
    domain::{
        simulator::simulator::SystemSimulator,
        vrm_system_model::{
            reservation::{
                reservation::{Reservation, ReservationTrait},
                reservation_store::{ReservationId, ReservationStore},
            },
            resource::{node_resource::NodeResource, resource_store::ResourceStore},
            rms::{
                advance_reservation_trait::AdvanceReservationRms,
                rms::{Rms, RmsBase, RmsLoadMetric},
                rms_node_network_trait::Helper,
            },
            schedule::slotted_schedule::network_slotted_schedule::topology::NetworkTopology,
            scheduler_trait::Schedule,
            scheduler_type::{ScheduleContext, SchedulerType},
            utils::id::{AciId, ShadowScheduleId, SlottedScheduleId},
        },
    },
    error::ConversionError,
};

/// Simulates both links and nodes of a cluster
#[derive(Debug)]
pub struct RmsSimulator {
    pub base: RmsBase,
    pub node_schedule: Box<dyn Schedule>,
    pub network_schedule: Box<dyn Schedule>,
    pub node_shadow_schedule: HashMap<ShadowScheduleId, Box<dyn Schedule>>,
    pub network_shadow_schedule: HashMap<ShadowScheduleId, Box<dyn Schedule>>,
}

impl Rms for RmsSimulator {
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
        if self.base.reservation_store.is_link(reservation_id) {
            match shadow_schedule_id {
                Some(id) => self.network_shadow_schedule.get_mut(&id).expect("network_shadow_schedule contains ShadowSchedule."),
                None => &mut self.network_schedule,
            }
        } else if self.base.reservation_store.is_node(reservation_id) {
            match shadow_schedule_id {
                Some(id) => self.node_shadow_schedule.get_mut(&id).expect("node_shadow_schedule contains ShadowSchedule."),
                None => &mut self.node_schedule,
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

impl TryFrom<(DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)> for RmsSimulator {
    type Error = ConversionError;

    fn try_from(args: (DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)) -> Result<Self, Self::Error> {
        let (dto, simulator, aci_id, reservation_store) = args.clone();
        let resource_store = ResourceStore::new();
        let (nodes, links) = RmsBase::get_nodes_and_links(&dto);

        // Setup RmsNodeSimulator
        let mut schedule_capacity = 0;

        // Add nodes to ResourceStore
        for node in nodes.iter() {
            schedule_capacity += node.cpus;
            resource_store.add_node(NodeResource::new(node.name.clone(), node.cpus));
        }

        let name = format!("AcI: {}, RmsType: {}", aci_id, dto.typ);
        let schedule_context = ScheduleContext {
            id: SlottedScheduleId::new(name.clone()),
            number_of_slots: dto.num_of_slots,
            slot_width: dto.slot_width,
            capacity: schedule_capacity,
            simulator: simulator.clone(),
            reservation_store: reservation_store.clone(),
        };

        let scheduler_type = SchedulerType::from_str(&dto.scheduler_typ)?; // TODO
        let node_schedule = scheduler_type.get_instance(schedule_context);

        // Setup RmsNetworkSimulator
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

        let mut scheduler_type = SchedulerType::from_str(&dto.scheduler_typ)?; // TODO
        scheduler_type = scheduler_type.get_network_scheduler_variant(topology, resource_store.clone());
        let network_schedule = scheduler_type.get_instance(schedule_context);

        if resource_store.get_num_of_nodes() <= 0 {
            log::info!("Empty Rms: The newly created Rms of type {} of AcI {} contains no Nodes", dto.typ, aci_id);
        }

        let base = RmsBase::new(aci_id, dto.typ, reservation_store, resource_store.clone());

        Ok(RmsSimulator { base, node_schedule, network_schedule, node_shadow_schedule: HashMap::new(), network_shadow_schedule: HashMap::new() })
    }
}

impl Helper for RmsSimulator {
    fn get_node_shadow_schedule(&self) -> &HashMap<ShadowScheduleId, Box<dyn Schedule>> {
        &self.node_shadow_schedule
    }

    fn get_mut_network_shadow_schedule(&mut self) -> &mut HashMap<ShadowScheduleId, Box<dyn Schedule>> {
        &mut self.network_shadow_schedule
    }

    fn get_network_shadow_schedule(&self) -> &HashMap<ShadowScheduleId, Box<dyn Schedule>> {
        &self.node_shadow_schedule
    }

    fn get_mut_node_shadow_schedule(&mut self) -> &mut HashMap<ShadowScheduleId, Box<dyn Schedule>> {
        &mut self.node_shadow_schedule
    }

    fn get_node_schedule(&self) -> &Box<dyn Schedule> {
        &self.node_schedule
    }

    fn get_mut_node_schedule(&mut self) -> &mut Box<dyn Schedule> {
        &mut self.node_schedule
    }

    fn get_network_schedule(&self) -> &Box<dyn Schedule> {
        &self.network_schedule
    }

    fn get_mut_network_schedule(&mut self) -> &mut Box<dyn Schedule> {
        &mut self.network_schedule
    }

    fn set_node_schedule(&mut self, new_node_schedule: Box<dyn Schedule>) {
        self.node_schedule = new_node_schedule;
    }

    fn set_network_schedule(&mut self, new_network_schedule: Box<dyn Schedule>) {
        self.network_schedule = new_network_schedule;
    }
}
