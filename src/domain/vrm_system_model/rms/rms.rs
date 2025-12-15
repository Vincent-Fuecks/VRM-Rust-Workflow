use crate::api::vrm_system_model_dto::aci_dto::RMSSystemDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;
use crate::domain::vrm_system_model::resource::grid_node::GridNode;
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::scheduler_type::SchedulerType;
use crate::domain::vrm_system_model::utils::id::{GridNodeId, RmsId, RouterId, SlottedScheduleId};
use crate::error::ConversionError;

use std::any::Any;
use std::collections::HashMap;
use std::str::FromStr;

pub trait Rms: std::fmt::Debug + Any + Send + Sync {
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
    pub grid_nodes: Vec<GridNode>,
    pub slot_width: i64,
    pub num_of_slots: i64,
}

impl TryFrom<(RMSSystemDto, Box<dyn SystemSimulator>, String)> for RmsBase {
    type Error = ConversionError;
    fn try_from(args: (RMSSystemDto, Box<dyn SystemSimulator>, String)) -> Result<Self, Self::Error> {
        let (dto, simulator, aci_name) = args;
        let rms_id: RmsId = RmsId::new(format!("AcI: {}, RmsType: {}", aci_name.clone(), &dto.typ));
        let schedule_id: SlottedScheduleId = SlottedScheduleId::new(format!("AcI: {}, RmsType: {}", aci_name, &dto.scheduler_type));

        let mut grid_nodes: Vec<GridNode> = Vec::new();

        let mut schedule_capacity: i64 = 0;

        for node in dto.grid_nodes.iter() {
            let connected_to_router: Vec<RouterId> = node.connected_to_router.iter().map(|router_id| RouterId::new(router_id.clone())).collect();

            schedule_capacity += node.cpus;

            grid_nodes.push(GridNode { id: GridNodeId::new(node.id.clone()), cpus: node.cpus, connected_to_router });
        }

        let schedule_type = SchedulerType::from_str(&dto.scheduler_type)?;
        let schedule = schedule_type.get_instance(schedule_id, dto.num_of_slots, dto.slot_width, schedule_capacity, simulator);

        Ok(RmsBase {
            id: rms_id,
            schedule: schedule,
            shadow_schedules: HashMap::new(),
            grid_nodes: grid_nodes,
            slot_width: dto.slot_width,
            num_of_slots: dto.num_of_slots,
        })
    }
}
