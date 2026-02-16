use std::collections::HashMap;

use crate::domain::vrm_system_model::{
    reservation::{
        reservation::{Reservation, ReservationTrait},
        reservation_store::{ReservationId, ReservationStore},
    },
    rms::{
        advance_reservation_trait::AdvanceReservationRms,
        rms::{Rms, RmsLoadMetric},
        rms_simulator::rms_simulator::RmsSimulator,
        slurm::slurm::SlurmRms,
    },
    scheduler_trait::Schedule,
    utils::id::ShadowScheduleId,
};

pub trait Helper {
    fn get_network_shadow_schedule(&self) -> &HashMap<ShadowScheduleId, Box<dyn Schedule>>;
    fn get_mut_network_shadow_schedule(&mut self) -> &mut HashMap<ShadowScheduleId, Box<dyn Schedule>>;

    fn get_node_shadow_schedule(&self) -> &HashMap<ShadowScheduleId, Box<dyn Schedule>>;
    fn get_mut_node_shadow_schedule(&mut self) -> &mut HashMap<ShadowScheduleId, Box<dyn Schedule>>;

    fn get_node_schedule(&self) -> &Box<dyn Schedule>;
    fn get_mut_node_schedule(&mut self) -> &mut Box<dyn Schedule>;

    fn get_network_schedule(&self) -> &Box<dyn Schedule>;
    fn get_mut_network_schedule(&mut self) -> &mut Box<dyn Schedule>;

    fn set_node_schedule(&mut self, new_node_schedule: Box<dyn Schedule>);
    fn set_network_schedule(&mut self, new_network_schedule: Box<dyn Schedule>);
}

trait RmsNodeNetwork: AdvanceReservationRms + Helper + Rms {}

impl<T: RmsNodeNetwork> AdvanceReservationRms for T {
    fn create_shadow_schedule(&mut self, shadow_schedule_id: &ShadowScheduleId) -> bool {
        if self.get_mut_network_shadow_schedule().contains_key(shadow_schedule_id)
            || self.get_mut_node_shadow_schedule().contains_key(shadow_schedule_id)
        {
            log::error!(
                "Creating new shadow schedule is not possible because shadow schedule id ({}) does already exist. Please first delete the old shadow schedule.",
                shadow_schedule_id
            );
            return false;
        }
        let node_schedule_clone = self.get_mut_node_schedule().clone_box();
        let network_schedule_clone = self.get_mut_network_schedule().clone_box();

        if !self.get_mut_node_shadow_schedule().insert(shadow_schedule_id.clone(), node_schedule_clone).is_none()
            || !self.get_mut_network_shadow_schedule().insert(shadow_schedule_id.clone(), network_schedule_clone).is_none()
        {
            log::error!("ErrorShadowScheduleAlreadyExists: ShadowSchedule is now curupted.");
            return false;
        }

        return true;
    }

    fn commit_shadow_schedule(&mut self, shadow_schedule_id: &ShadowScheduleId) -> bool {
        if self.get_mut_network_shadow_schedule().contains_key(shadow_schedule_id)
            && self.get_mut_node_shadow_schedule().contains_key(shadow_schedule_id)
        {
            let new_node_schedule = self.get_mut_node_shadow_schedule().remove(shadow_schedule_id);
            let new_network_schedule = self.get_mut_network_shadow_schedule().remove(shadow_schedule_id);

            if !new_node_schedule.is_none() && !new_network_schedule.is_none() {
                self.set_node_schedule(new_node_schedule.unwrap());
                self.set_network_schedule(new_network_schedule.unwrap());
                return true;
            }
        }

        log::error!("Finding and removing of shadow schedule with id {} was not possible", shadow_schedule_id.clone());
        return false;
    }

    fn delete_shadow_schedule(&mut self, shadow_schedule_id: &ShadowScheduleId) -> bool {
        if self.get_mut_network_shadow_schedule().contains_key(shadow_schedule_id)
            && self.get_mut_node_shadow_schedule().contains_key(shadow_schedule_id)
        {
            let removed_node_schedule = self.get_mut_node_shadow_schedule().remove(shadow_schedule_id);
            let removed_network_schedule = self.get_mut_network_shadow_schedule().remove(shadow_schedule_id);

            if removed_node_schedule.is_none() && removed_network_schedule.is_none() {
                return true;
            }
        }

        log::error!("Removing shadow schedule was not possible. Shadow schedule id ({}) was not found", shadow_schedule_id.clone());
        return false;
    }

    fn get_fragmentation(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        match shadow_schedule_id {
            Some(id) => {
                self.get_mut_network_shadow_schedule()
                    .get_mut(&id)
                    .expect("network_shadow_schedule contains ShadowSchedule.")
                    .get_fragmentation(start, end)
                    + self
                        .get_mut_node_shadow_schedule()
                        .get_mut(&id)
                        .expect("node_shadow_schedule contains ShadowSchedule.")
                        .get_fragmentation(start, end)
            }
            None => self.get_mut_network_schedule().get_fragmentation(start, end) + self.get_mut_node_schedule().get_fragmentation(start, end),
        }
    }

    fn get_system_fragmentation(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        match shadow_schedule_id {
            Some(id) => {
                self.get_mut_network_shadow_schedule()
                    .get_mut(&id)
                    .expect("network_shadow_schedule contains ShadowSchedule.")
                    .get_system_fragmentation()
                    + self
                        .get_mut_node_shadow_schedule()
                        .get_mut(&id)
                        .expect("node_shadow_schedule contains ShadowSchedule.")
                        .get_system_fragmentation()
            }
            None => self.get_mut_network_schedule().get_system_fragmentation() + self.get_mut_node_schedule().get_system_fragmentation(),
        }
    }

    fn can_handle_adc_request(&self, res: Reservation) -> bool {
        if res.is_link() || res.is_node() {
            return self.get_base().resource_store.can_handle_adc_request(res);
        }

        log::debug!(
            "The rms {:?} can not process Reservations of Type {:?} (ReservationName: {:?}) the rms can only process LinkReservations and NodeReservations.",
            self.get_base().id,
            res.get_type(),
            res.get_name()
        );
        return false;
    }

    fn can_handle_aci_request(&self, reservation_store: ReservationStore, reservation_id: ReservationId) -> bool {
        if reservation_store.is_link(reservation_id) || reservation_store.is_node(reservation_id) {
            return self.get_base().resource_store.can_handle_aci_request(reservation_store, reservation_id);
        }

        log::debug!(
            "The rms {:?} can not process Reservations of Type {:?} (ReservationName: {:?}) the rms can only process LinkReservations and NodeReservations.",
            self.get_base().id,
            reservation_store.get_type(reservation_id),
            reservation_store.get_name_for_key(reservation_id)
        );
        return false;
    }

    fn get_load_metric(&self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> RmsLoadMetric {
        match shadow_schedule_id {
            Some(id) => RmsLoadMetric {
                node_load_metric: Some(
                    self.get_node_shadow_schedule().get(&id).expect("network_shadow_schedule contains ShadowSchedule.").get_load_metric(start, end),
                ),
                link_load_metric: Some(
                    self.get_network_shadow_schedule().get(&id).expect("node_shadow_schedule contains ShadowSchedule.").get_load_metric(start, end),
                ),
            },
            None => RmsLoadMetric {
                node_load_metric: Some(self.get_node_schedule().get_load_metric(start, end)),
                link_load_metric: Some(self.get_network_schedule().get_load_metric(start, end)),
            },
        }
    }

    fn get_load_metric_up_to_date(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> RmsLoadMetric {
        match shadow_schedule_id {
            Some(id) => RmsLoadMetric {
                node_load_metric: Some(
                    self.get_mut_node_shadow_schedule()
                        .get_mut(&id)
                        .expect("network_shadow_schedule contains ShadowSchedule.")
                        .get_load_metric_up_to_date(start, end),
                ),
                link_load_metric: Some(
                    self.get_mut_network_shadow_schedule()
                        .get_mut(&id)
                        .expect("node_shadow_schedule contains ShadowSchedule.")
                        .get_load_metric_up_to_date(start, end),
                ),
            },
            None => RmsLoadMetric {
                node_load_metric: Some(self.get_mut_node_schedule().get_load_metric_up_to_date(start, end)),
                link_load_metric: Some(self.get_mut_network_schedule().get_load_metric_up_to_date(start, end)),
            },
        }
    }

    fn get_simulation_load_metric(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> RmsLoadMetric {
        match shadow_schedule_id {
            Some(id) => RmsLoadMetric {
                node_load_metric: Some(
                    self.get_mut_node_shadow_schedule()
                        .get_mut(&id)
                        .expect("network_shadow_schedule contains ShadowSchedule.")
                        .get_simulation_load_metric(),
                ),
                link_load_metric: Some(
                    self.get_mut_network_shadow_schedule()
                        .get_mut(&id)
                        .expect("node_shadow_schedule contains ShadowSchedule.")
                        .get_simulation_load_metric(),
                ),
            },
            None => RmsLoadMetric {
                node_load_metric: Some(self.get_mut_node_schedule().get_simulation_load_metric()),
                link_load_metric: Some(self.get_mut_network_schedule().get_simulation_load_metric()),
            },
        }
    }
}

impl RmsNodeNetwork for SlurmRms {}
impl RmsNodeNetwork for RmsSimulator {}
