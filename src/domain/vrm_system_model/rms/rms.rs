use crate::api::vrm_system_model_dto::aci_dto::RMSSystemDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;
use crate::domain::vrm_system_model::resource::{grid_node::GridNode, network_link::NetworkLink};
use crate::domain::vrm_system_model::schedule::slotted_schedule::SlottedSchedule;
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::scheduler_type::SchedulerType;
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
    pub id: ReservationKey,
    pub schedule: Box<dyn Schedule>,
    pub shadow_schedules: HashMap<ReservationKey, Box<dyn Schedule>>,
    pub grid_nodes: Vec<GridNode>,
    pub network_links: Vec<NetworkLink>,
    pub slot_width: i64,
    pub num_of_slots: i64,
}

impl TryFrom<(RMSSystemDto, Box<dyn SystemSimulator>, String)> for RmsBase {
    type Error = ConversionError;
    fn try_from(args: (RMSSystemDto, Box<dyn SystemSimulator>, String)) -> Result<Self, Self::Error> {
        let (dto, simulator, aci_name) = args;
        let rms_id: ReservationKey = ReservationKey { id: aci_name.clone() + "---" + &dto.typ };
        let schedule_id: ReservationKey = ReservationKey { id: aci_name + "---" + &dto.scheduler_type };

        let mut grid_nodes: Vec<GridNode> = Vec::new();
        let mut network_links: Vec<NetworkLink> = Vec::new();

        let mut schedule_capacity: i64 = 0;
        let mut network_capacity: i64 = 0;

        for node in dto.grid_nodes.iter() {
            let connected_to_router: Vec<ReservationKey> =
                node.connected_to_router.iter().map(|router_id| ReservationKey { id: router_id.clone() }).collect();

            schedule_capacity += node.cpus;

            grid_nodes.push(GridNode { id: ReservationKey { id: node.id.clone() }, cpus: node.cpus, connected_to_router });
        }

        for link in dto.network_links.iter() {
            network_capacity += link.capacity;

            let link_schedule_name = format!("Schedule NetworkLink {} -> {}", link.start_point, link.end_point);
            let link_schedule = SlottedSchedule::new(
                ReservationKey::new(link_schedule_name),
                dto.num_of_slots,
                dto.slot_width,
                link.capacity,
                true,
                simulator.clone_box(),
            );

            network_links.push(NetworkLink {
                id: ReservationKey { id: link.id.clone() },
                source: ReservationKey { id: link.start_point.clone() },
                target: ReservationKey { id: link.end_point.clone() },
                schedule: link_schedule,
            });
        }

        let schedule_type = SchedulerType::from_str(&dto.scheduler_type)?;
        let schedule = schedule_type.get_instance(schedule_id, dto.num_of_slots, dto.slot_width, schedule_capacity, simulator);

        Ok(RmsBase {
            id: rms_id,
            schedule: schedule,
            shadow_schedules: HashMap::new(),
            grid_nodes: grid_nodes,
            network_links: network_links,
            slot_width: dto.slot_width,
            num_of_slots: dto.num_of_slots,
        })
    }
}
