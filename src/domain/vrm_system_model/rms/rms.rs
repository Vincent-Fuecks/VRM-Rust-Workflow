use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation::ReservationState;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::resource::node_resource::NodeResource;
use crate::domain::vrm_system_model::resource::resource_store::ResourceStore;
use crate::domain::vrm_system_model::schedule::slotted_schedule::network_slotted_schedule::topology::Node;
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::scheduler_type::{ScheduleContext, SchedulerType};
use crate::domain::vrm_system_model::utils::id::{AciId, RmsId, ShadowScheduleId, SlottedScheduleId};

use std::any::Any;
use std::collections::HashMap;
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
    pub resource_store: ResourceStore,
    pub reservation_store: ReservationStore,
}

pub struct RmsContext {
    pub aci_id: AciId,
    pub rms_type: String,
    pub schedule_capacity: i64,
    pub slot_width: i64,
    pub num_of_slots: i64,
    pub nodes: Vec<Node>,
    pub reservation_store: ReservationStore,
    pub simulator: Arc<dyn SystemSimulator>,
    pub schedule_type: SchedulerType,
}

impl RmsBase {
    pub fn new(ctx: RmsContext, resource_store: ResourceStore) -> Self {
        let RmsContext { aci_id, rms_type, schedule_capacity, slot_width, num_of_slots, nodes, reservation_store, simulator, schedule_type } = ctx;

        let name = format!("AcI: {}, RmsType: {}", aci_id, &rms_type);

        // Add nodes to ResourceStore
        for node in nodes.iter() {
            resource_store.add_node(NodeResource::new(node.name.clone(), node.cpus));
        }

        let schedule_context = ScheduleContext {
            id: SlottedScheduleId::new(name.clone()),
            number_of_slots: num_of_slots,
            slot_width: slot_width,
            capacity: schedule_capacity,
            simulator,
            reservation_store: reservation_store.clone(),
        };

        let schedule = schedule_type.get_instance(schedule_context);

        if resource_store.get_num_of_nodes() <= 0 {
            log::info!("Empty Rms: The newly created Rms of type {} of AcI {} contains no Nodes", rms_type, aci_id);
        }

        RmsBase {
            id: RmsId::new(name),
            schedule: schedule,
            shadow_schedules: HashMap::new(),
            slot_width: slot_width,
            num_of_slots: num_of_slots,
            resource_store,
            reservation_store,
        }
    }
}
