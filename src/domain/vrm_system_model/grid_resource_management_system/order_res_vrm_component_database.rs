use std::{cmp::Ordering, collections::HashMap};

use crate::domain::vrm_system_model::{
    grid_resource_management_system::vrm_component_manager::{VrmComponentContainer, VrmComponentManager},
    reservation::{probe_reservations::ProbeReservations, reservation_store::ReservationId, reservations::Reservations},
    utils::id::ComponentId,
};

/**
 * A mapping between reservations and the AIs which handle them. In this
 * map the names of the reservations do not have to be unique
 * (see {@link Reservation#getJobName()}.
 *
 * There is the similar class {@link AIReservationDatabase} which can handle only
 * entries with unique names and is therefore saver and faster.
 *
 * @see AIReservationDatabase
 */
pub struct OrderResVrmComponentDatabase {
    pub store: HashMap<ReservationId, ComponentId>,
    res_comparator: Box<dyn Fn(ReservationId, ReservationId) -> Ordering>,
    ai_comparator: Box<dyn Fn(&VrmComponentContainer, &VrmComponentContainer) -> Ordering>,
}

impl OrderResVrmComponentDatabase {
    pub fn new<F1, F2>(res_sort: F1, ai_sort: F2) -> Self
    where
        F1: Fn(ReservationId, ReservationId) -> Ordering + 'static,
        F2: Fn(&VrmComponentContainer, &VrmComponentContainer) -> Ordering + 'static,
    {
        Self { store: HashMap::new(), res_comparator: Box::new(res_sort), ai_comparator: Box::new(ai_sort) }
    }

    /// Adds a reservation and its corresponding AI container.
    pub fn put(&mut self, res: ReservationId, component_id: ComponentId) {
        self.store.insert(res, component_id);
    }

    /// Adds multiple reservations belonging to a single AI.
    /// TODO Should I do something with the ShadowScheduleId?
    pub fn put_all(&mut self, probe_reservations: ProbeReservations) {
        // for res_id in probe_reservations.get_ids() {
        //     let (component_id, shadow_schedule_id) = probe_reservations.get_origin_information(res_id);

        //     if component_id.is_some() && shadow_schedule_id.is_none() {
        //         self.store.insert(res_id, component_id.unwrap());
        //     } else {
        //         panic!(
        //             "ErrorProbeReservations: ComponentId {:?} should be Some() and ShadowScheduleId {:?} should be None.",
        //             component_id, shadow_schedule_id
        //         );
        //     }
        // }
    }

    fn compare_reservations(&self, manager: &VrmComponentManager, res1: ReservationId, res2: ReservationId) -> Ordering {
        let mut order = (self.res_comparator)(res1, res2);

        if order == Ordering::Equal {
            let ai1 = self.store.get(&res1);
            let ai2 = self.store.get(&res2);

            match (ai1, ai2) {
                (Some(a), Some(b)) => {
                    let container0 = manager.vrm_components.get(a).unwrap();

                    let container1 = manager.vrm_components.get(b).unwrap();

                    order = (self.ai_comparator)(container0, container1);
                }
                _ => {
                    panic!("FATAL: Reservations cannot be compared, as they are not elements of this container. {:?}, {:?}", res1, res2);
                }
            }
        }
        order
    }

    pub fn sorted_key_set(&self, manager: &VrmComponentManager) -> Vec<ReservationId> {
        let mut keys: Vec<ReservationId> = self.store.keys().cloned().collect();

        keys.sort_by(|a, b| self.compare_reservations(manager, *a, *b));

        keys
    }
}
