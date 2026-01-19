use std::{cmp::Ordering, collections::HashMap};

use crate::domain::vrm_system_model::{
    grid_resource_management_system::aci_manager::{AcIContainer, AcIManager},
    reservation::{reservation_store::ReservationId, reservations::Reservations},
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
pub struct OrderGridComponentResDatabase {
    pub store: HashMap<ReservationId, ComponentId>,
    res_comparator: Box<dyn Fn(ReservationId, ReservationId) -> Ordering>,
    ai_comparator: Box<dyn Fn(&AcIContainer, &AcIContainer) -> Ordering>,
}

impl OrderGridComponentResDatabase {
    pub fn new<F1, F2>(res_sort: F1, ai_sort: F2) -> Self
    where
        F1: Fn(ReservationId, ReservationId) -> Ordering + 'static,
        F2: Fn(&AcIContainer, &AcIContainer) -> Ordering + 'static,
    {
        Self { store: HashMap::new(), res_comparator: Box::new(res_sort), ai_comparator: Box::new(ai_sort) }
    }

    /// Adds a reservation and its corresponding AI container.
    pub fn put(&mut self, res: ReservationId, component_id: ComponentId) {
        self.store.insert(res, component_id);
    }

    /// Adds multiple reservations belonging to a single AI.
    pub fn put_all(&mut self, reservations: Reservations, component_id: ComponentId) {
        for res in reservations.iter() {
            self.store.insert(res.clone(), component_id.clone());
        }
    }

    fn compare_reservations(&self, aci_manager: &AcIManager, res1: ReservationId, res2: ReservationId) -> Ordering {
        let mut order = (self.res_comparator)(res1, res2);

        if order == Ordering::Equal {
            let ai1 = self.store.get(&res1);
            let ai2 = self.store.get(&res2);

            match (ai1, ai2) {
                (Some(a), Some(b)) => {
                    let aci0 = aci_manager.grid_components.get(a).unwrap();

                    let aci1 = aci_manager.grid_components.get(b).unwrap();

                    order = (self.ai_comparator)(aci0, aci1);
                }
                _ => {
                    panic!("FATAL: Reservations cannot be compared, as they are not elements of this container. {:?}, {:?}", res1, res2);
                }
            }
        }
        order
    }

    pub fn sorted_key_set(&self, aci_manager: &AcIManager) -> Vec<ReservationId> {
        let mut keys: Vec<ReservationId> = self.store.keys().cloned().collect();

        keys.sort_by(|a, b| self.compare_reservations(aci_manager, *a, *b));

        keys
    }
}
