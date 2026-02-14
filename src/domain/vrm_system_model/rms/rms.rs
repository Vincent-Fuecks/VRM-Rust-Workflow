use crate::api::rms_config_dto::rms_dto::{DummyRmsDto, RmsSystemWrapper};
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation::ReservationState;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::resource::link_resource::LinkResource;
use crate::domain::vrm_system_model::resource::node_resource::NodeResource;
use crate::domain::vrm_system_model::resource::resource_trait::Resource;
use crate::domain::vrm_system_model::resource::resources::Resources;
use crate::domain::vrm_system_model::schedule::slotted_schedule::network_slotted_schedule::topology::{Link, Node};
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::scheduler_type::{ScheduleContext, SchedulerType};
use crate::domain::vrm_system_model::utils::id::{AciId, ResourceName, RmsId, RouterId, ShadowScheduleId, SlottedScheduleId};
use crate::error::ConversionError;

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub trait Rms: std::fmt::Debug + Any {
    fn get_base(&self) -> &RmsBase;
    fn get_base_mut(&mut self) -> &mut RmsBase;
    fn as_any(&self) -> &dyn Any;

    fn get_shadow_schedule_keys(&self) -> Vec<ShadowScheduleId> {
        return self.get_base().shadow_schedules.keys().map(|key| key.clone()).collect();
    }

    fn get_shadow_schedules(&self) -> &HashMap<ShadowScheduleId, Box<dyn Schedule>> {
        return &self.get_base().shadow_schedules;
    }

    fn get_mut_shadow_schedules(&mut self) -> &HashMap<ShadowScheduleId, Box<dyn Schedule>> {
        return &self.get_base_mut().shadow_schedules;
    }

    fn get_schedule_box_copy(&mut self) -> Box<dyn Schedule> {
        return self.get_base_mut().schedule.clone_box();
    }

    fn get_shadow_schedule(&self, shadow_schedule_id: ShadowScheduleId) -> &Box<dyn Schedule> {
        return self.get_base().shadow_schedules.get(&shadow_schedule_id).expect("Shadow schedule id was in shadow schedules not found.");
    }

    fn get_mut_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> &mut Box<dyn Schedule> {
        return self.get_base_mut().shadow_schedules.get_mut(&shadow_schedule_id).expect("Shadow schedule id was in shadow schedules not found.");
    }

    fn get_master_schedule(&self) -> &Box<dyn Schedule> {
        return &self.get_base().schedule;
    }

    fn get_mut_master_schedule(&mut self) -> &mut Box<dyn Schedule> {
        return &mut self.get_base_mut().schedule;
    }

    fn set_reservation_state(&mut self, id: ReservationId, new_state: ReservationState) {
        self.get_base().reservation_store.update_state(id, new_state);
    }
}

#[derive(Debug)]
pub struct RmsBase {
    pub id: RmsId,
    pub schedule: Box<dyn Schedule>,
    pub shadow_schedules: HashMap<ShadowScheduleId, Box<dyn Schedule>>,
    pub slot_width: i64,
    pub num_of_slots: i64,
    pub resources: Resources,
    pub reservation_store: ReservationStore,
}

pub struct RmsContext {
    pub aci_id: AciId,
    pub rms_type: String,
    pub slot_width: i64,
    pub num_of_slots: i64,
    pub reservation_store: ReservationStore,
    pub simulator: Arc<dyn SystemSimulator>,
    pub schedule_type: SchedulerType,
}

impl RmsBase {
    pub fn new_only_nodes(ctx: RmsContext, nodes: &Vec<Node>) -> Self {
        let RmsContext { aci_id, rms_type, slot_width, num_of_slots, reservation_store, simulator, schedule_type } = ctx;

        let name = format!("AcI: {}, RmsType: {}", aci_id, &rms_type);
        let mut grid_nodes: Vec<Box<dyn Resource>> = Vec::new();
        let mut schedule_capacity: i64 = 0;

        for node in nodes.iter() {
            let mut connected_to_router: HashSet<RouterId> = HashSet::new();
            let connected_to_router_vec: Vec<RouterId> = node.connected_to_router.iter().map(|router_id| RouterId::new(router_id.clone())).collect();

            connected_to_router.extend(connected_to_router_vec);

            schedule_capacity += node.cpus;

            grid_nodes.push(Box::new(NodeResource::new(ResourceName::new(node.id.clone()), node.cpus, connected_to_router)));
        }

        let resources: Resources = Resources::new(grid_nodes, Vec::new());

        let schedule_context = ScheduleContext {
            id: SlottedScheduleId::new(name.clone()),
            number_of_slots: num_of_slots,
            slot_width: slot_width,
            capacity: schedule_capacity,
            simulator,
            reservation_store: reservation_store.clone(),
        };

        let schedule = schedule_type.get_instance(schedule_context);

        RmsBase {
            id: RmsId::new(name),
            schedule: schedule,
            shadow_schedules: HashMap::new(),
            slot_width: slot_width,
            num_of_slots: num_of_slots,
            resources: resources,
            reservation_store,
        }
    }

    pub fn new(ctx: RmsContext, nodes: &Vec<Node>, links: &Vec<Link>) -> Self {
        let RmsContext { aci_id, rms_type, slot_width, num_of_slots, reservation_store, simulator, schedule_type } = ctx;

        let name = format!("AcI: {}, RmsType: {}", aci_id, &rms_type);
        let mut resources: Vec<Box<dyn Resource>> = Vec::new();
        let mut schedule_capacity: i64 = 0;

        for node in nodes.iter() {
            let mut connected_to_router: HashSet<RouterId> = HashSet::new();
            let connected_to_router_vec: Vec<RouterId> = node.connected_to_router.iter().map(|router_id| RouterId::new(router_id.clone())).collect();

            connected_to_router.extend(connected_to_router_vec);

            schedule_capacity += node.cpus;

            resources.push(Box::new(NodeResource::new(NodeResourceId::new(node.id.clone()), node.cpus, connected_to_router)));
        }

        for link in links.iter() {
            resources.push(Box::new(LinkResource::new(LinkResourceId::new(link.id.clone()), link.source, link.target, link.capacity)));
        }

        let resources: Resources = Resources::new(resources, Vec::new());

        let schedule_context = ScheduleContext {
            id: SlottedScheduleId::new(name.clone()),
            number_of_slots: num_of_slots,
            slot_width: slot_width,
            capacity: schedule_capacity,
            simulator,
            reservation_store: reservation_store.clone(),
        };

        let schedule = schedule_type.get_instance(schedule_context);

        RmsBase {
            id: RmsId::new(name),
            schedule: schedule,
            shadow_schedules: HashMap::new(),
            slot_width: slot_width,
            num_of_slots: num_of_slots,
            resources: resources,
            reservation_store,
        }
    }
}

impl TryFrom<(DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore, SchedulerType)> for RmsBase {
    type Error = ConversionError;
    fn try_from(args: (DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore, SchedulerType)) -> Result<Self, Self::Error> {
        let (dto, simulator, aci_id, reservation_store, schedule_type) = args;

        let schedule_id: SlottedScheduleId = SlottedScheduleId::new(format!("AcI: {}, RmsType: {}", aci_id, &dto.scheduler_typ));

        let mut grid_nodes: Vec<Box<dyn Resource>> = Vec::new();

        let mut schedule_capacity: i64 = 0;
        // TODO What happen with the capacity of NetworkLinks?
        for node in dto.grid_nodes.iter() {
            let mut connected_to_router: HashSet<RouterId> = HashSet::new();
            let connected_to_router_vec: Vec<RouterId> = node.connected_to_router.iter().map(|router_id| RouterId::new(router_id.clone())).collect();

            connected_to_router.extend(connected_to_router_vec);

            schedule_capacity += node.cpus;

            grid_nodes.push(Box::new(NodeResource::new(NodeResourceId::new(node.id.clone()), node.cpus, connected_to_router)));
        }

        let resources: Resources = Resources::new(grid_nodes, Vec::new());

        let schedule_context = ScheduleContext {
            id: schedule_id,
            number_of_slots: dto.num_of_slots,
            slot_width: dto.slot_width,
            capacity: schedule_capacity,
            simulator,
            reservation_store: reservation_store.clone(),
        };

        let schedule = schedule_type.get_instance(schedule_context);

        Ok(RmsBase {
            id: rms_id,
            schedule: schedule,
            shadow_schedules: HashMap::new(),
            slot_width: dto.slot_width,
            num_of_slots: dto.num_of_slots,
            resources: resources,
            reservation_store,
        })
    }
}
