use crate::api::vrm_system_model_dto::aci_dto::RMSSystemDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use crate::domain::vrm_system_model::resource::node_resource::NodeResource;
use crate::domain::vrm_system_model::resource::resource_trait::Resource;
use crate::domain::vrm_system_model::resource::resources::Resources;
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::scheduler_type::SchedulerType;
use crate::domain::vrm_system_model::utils::id::{NodeResourceId, RmsId, RouterId, SlottedScheduleId};
use crate::error::ConversionError;

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

pub trait Rms: std::fmt::Debug + Any {
    fn get_base(&self) -> &RmsBase;
    fn get_base_mut(&mut self) -> &mut RmsBase;
    fn as_any(&self) -> &dyn Any;

    fn get_shadow_schedule_keys(&self) -> Vec<ReservationKey> {
        return self.get_base().shadow_schedules.keys().map(|key| key.clone()).collect();
    }

    fn get_shadow_schedules(&self) -> &HashMap<ReservationKey, Box<dyn Schedule>> {
        return &self.get_base().shadow_schedules;
    }

    fn get_mut_shadow_schedules(&mut self) -> &HashMap<ReservationKey, Box<dyn Schedule>> {
        return &self.get_base_mut().shadow_schedules;
    }

    fn get_schedule_box_copy(&mut self) -> Box<dyn Schedule> {
        return self.get_base_mut().schedule.clone_box();
    }

    fn get_shadow_schedule(&self, shadow_schedule_id: ReservationKey) -> &Box<dyn Schedule> {
        return self.get_base().shadow_schedules.get(&shadow_schedule_id).expect("Shadow schedule id was in shadow schedules not found.");
    }

    fn get_mut_shadow_schedule(&mut self, shadow_schedule_id: ReservationKey) -> &mut Box<dyn Schedule> {
        return self.get_base_mut().shadow_schedules.get_mut(&shadow_schedule_id).expect("Shadow schedule id was in shadow schedules not found.");
    }
}

#[derive(Debug)]
pub struct RmsBase {
    pub id: RmsId,
    pub schedule: Box<dyn Schedule>,
    pub shadow_schedules: HashMap<ReservationKey, Box<dyn Schedule>>,
    pub slot_width: i64,
    pub num_of_slots: i64,
    pub resources: Resources,
    pub reservation_store: ReservationStore,
}

impl TryFrom<(RMSSystemDto, Box<dyn SystemSimulator>, String, ReservationStore)> for RmsBase {
    type Error = ConversionError;
    fn try_from(args: (RMSSystemDto, Box<dyn SystemSimulator>, String, ReservationStore)) -> Result<Self, Self::Error> {
        let (dto, simulator, aci_name, reservation_store) = args;
        let rms_id: RmsId = RmsId::new(format!("AcI: {}, RmsType: {}", aci_name.clone(), &dto.typ));
        let schedule_id: SlottedScheduleId = SlottedScheduleId::new(format!("AcI: {}, RmsType: {}", aci_name, &dto.scheduler_type));

        let mut grid_nodes: Vec<Box<dyn Resource>> = Vec::new();

        let mut schedule_capacity: i64 = 0;

        for node in dto.grid_nodes.iter() {
            let mut connected_to_router: HashSet<RouterId> = HashSet::new();
            let connected_to_router_vec: Vec<RouterId> = node.connected_to_router.iter().map(|router_id| RouterId::new(router_id.clone())).collect();

            connected_to_router.extend(connected_to_router_vec);

            schedule_capacity += node.cpus;

            grid_nodes.push(Box::new(NodeResource::new(NodeResourceId::new(node.id.clone()), node.cpus, connected_to_router)));
        }

        let resources: Resources = Resources::new(grid_nodes, Vec::new());

        let schedule_type = SchedulerType::from_str(&dto.scheduler_type)?;
        let schedule = schedule_type.get_instance(schedule_id, dto.num_of_slots, dto.slot_width, schedule_capacity, simulator, reservation_store);

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
