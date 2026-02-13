use slotmap::{SlotMap, new_key_type};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use std::sync::{Arc, RwLock};

use crate::domain::vrm_system_model::reservation::link_reservation::LinkReservation;
use crate::domain::vrm_system_model::reservation::reservation::{
    Reservation, ReservationProceeding, ReservationState, ReservationTrait, ReservationTyp,
};
use crate::domain::vrm_system_model::utils::id::{ClientId, ComponentId, ReservationName, RouterId};
use crate::domain::vrm_system_model::workflow::workflow::Workflow;
use crate::domain::vrm_system_model::workflow::workflow_node::WorkflowNode;

// TODO Move in separate file
pub trait NotificationListener: Send + Sync + Debug {
    fn on_reservation_change(&self, key: ReservationId, new_state: ReservationState);
}

#[derive(Debug, Clone)]
struct NoOpenListener;
impl NotificationListener for NoOpenListener {
    fn on_reservation_change(&self, _: ReservationId, _: ReservationState) {}
}

new_key_type! {
    pub struct ReservationId;
}

/// TODO
#[derive(Debug, Clone)]
pub struct ReservationStore {
    /// Both maps are protected with a single lock.
    inner: Arc<RwLock<StoreInner>>,
}

#[derive(Debug, Clone)]
struct StoreInner {
    /// Reservation Storage.
    slots: SlotMap<ReservationId, Arc<RwLock<Reservation>>>,

    /// Index lookup InternalKey (ReservationId) using input reservation name (ReservationName).
    name_index: HashMap<ReservationName, ReservationId>,

    /// Lookup table of all Reservation of a client.
    client_index: HashMap<ClientId, HashSet<ReservationId>>,

    /// Lookup table of all Reservation of a component is currently handling (Acd or AcI).
    handler_index: HashMap<ComponentId, HashSet<ReservationId>>,

    // TODO Probably rework of mechanism is needed.
    /// Listener for changes
    listeners: Vec<Arc<dyn NotificationListener>>,
}

impl ReservationStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(StoreInner {
                slots: SlotMap::with_key(),
                name_index: HashMap::new(),
                client_index: HashMap::new(),
                handler_index: HashMap::new(),
                listeners: Vec::new(),
            })),
        }
    }

    /// This allows multiple components to subscribe to state changes.
    pub fn add_listener(&self, listener: Arc<dyn NotificationListener>) {
        let mut guard = self.inner.write().expect("RwLock poisoned");
        guard.listeners.push(listener);
    }

    /// Adds Reservation to ReservationStore.
    ///
    /// # Returns
    /// Returns the ReservationId (internal Key for ReservationStore).
    pub fn add(&self, reservation: Reservation) -> ReservationId {
        let mut guard = self.inner.write().unwrap();

        let name = reservation.get_name().clone();
        let client = reservation.get_client_id().clone();
        let handler = reservation.get_handler_id().clone();

        let key = guard.slots.insert(Arc::new(RwLock::new(reservation)));

        guard.name_index.insert(name, key);
        guard.client_index.entry(client).or_default().insert(key);
        if let Some(h) = handler {
            guard.handler_index.entry(h).or_default().insert(key);
        }

        return key;
    }

    /// Checks if the provided reservation ids are in the ReservationStore
    ///
    /// # Returns
    /// Returns true, if all reservation ids are in the store otherwise false is returned.     
    pub fn contains_reservations(&self, reservation_ids: Vec<ReservationId>) -> bool {
        let guard = self.inner.read().expect("RwLock poisoned");

        for reservation_id in reservation_ids {
            if !guard.slots.contains_key(reservation_id) {
                return false;
            }
        }
        return true;
    }

    /// Get Reservation with internal Id (ReservationId).
    ///  
    /// # Returns
    /// Returns the Some(Reservation) if ReservationId was present in SlotMap else return None.  
    pub fn get(&self, key: ReservationId) -> Option<Arc<RwLock<Reservation>>> {
        let guard = self.inner.read().expect("RwLock poisoned");
        guard.slots.get(key).cloned()
    }

    /// Returns true, if provided ReservationId is in store otherwise return false.
    pub fn contains(&self, reservation_id: ReservationId) -> bool {
        match self.get(reservation_id) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn get_reservation_snapshot(&self, reservation_id: ReservationId) -> Option<Reservation> {
        let guard = self.inner.read().expect("Repository lock poisoned");

        guard.slots.get(reservation_id).map(|arc_lock| {
            let res_guard = arc_lock.read().expect("Individual reservation lock poisoned");
            res_guard.clone()
        })
    }

    /// Get Reservation with User reservation name (ReservationName).
    ///  
    /// # Returns
    /// Returns Some(Reservation) if ReservationName was present in SlotMap else return None.  
    pub fn get_by_name(&self, name: &ReservationName) -> Option<Arc<RwLock<Reservation>>> {
        let guard = self.inner.read().expect("RwLock poisoned");
        let key = guard.name_index.get(name)?;
        guard.slots.get(*key).cloned()
    }

    /// Get Reservation user name (ReservationName) with internal reservation id (ReservationId).
    ///  
    /// # Returns
    /// Returns Some(ReservationName) if ReservationId was present in SlotMap else return None.  
    pub fn get_name_for_key(&self, key: ReservationId) -> Option<ReservationName> {
        self.get(key).map(|handle| handle.read().unwrap().get_name().clone())
    }

    /// Get Reservation id (ReservationId) for user name (ReservationName).
    ///  
    /// # Returns
    /// Returns Some(ReservationId) if ReservationName was present in SlotMap else return None.  
    pub fn get_key_for_name(&self, name: ReservationName) -> ReservationId {
        let guard = self.inner.read().expect("RwLock poisoned");
        let key = guard.name_index.get(&name);
        return key.unwrap().clone();
    }

    // TODO later return Reservations object if possible.
    /// Retrieve all keys belonging to a specific Client
    pub fn get_client_reservations(&self, client_id: &ClientId) -> Vec<ReservationId> {
        let guard = self.inner.read().unwrap();
        guard.client_index.get(client_id).map(|set| set.iter().cloned().collect()).unwrap_or_default()
    }

    // TODO later return Reservations object if possible.
    /// Retrieve all keys managed by a specific ADC/AI
    pub fn get_managed_reservations(&self, component_id: &ComponentId) -> Vec<ReservationId> {
        let guard = self.inner.read().unwrap();
        guard.handler_index.get(component_id).map(|set| set.iter().cloned().collect()).unwrap_or_default()
    }

    /// Retrieves form the provided reservation id the reserved_capacity
    pub fn get_reserved_capacity(&self, reservation_id: ReservationId) -> i64 {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_reserved_capacity();
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id);
            return 0;
        }
    }

    /// Retrieves form the provided reservation id the start_point, if it is a LinkReservation
    pub fn get_start_point(&self, reservation_id: ReservationId) -> Option<RouterId> {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            let res = res.as_any().downcast_ref::<LinkReservation>();
            return res.unwrap().start_point.clone();
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id);
            return None;
        }
    }

    /// Retrieves form the provided reservation id the end_point, if it is a LinkReservation.
    pub fn get_end_point(&self, reservation_id: ReservationId) -> Option<RouterId> {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            let res = res.as_any().downcast_ref::<LinkReservation>();
            return res.unwrap().end_point.clone();
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id);
            return None;
        }
    }

    /// Returns the client_id of the provided reservation_id. Panics if no client id was found.
    pub fn get_client_id(&self, reservation_id: ReservationId) -> ClientId {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_client_id();
        } else {
            panic!("Reservation (id: {:?}) does not contain a client id.", reservation_id);
        }
    }

    /// Returns the handler_id of the provided reservation_id. Panics if no handler_id was found.
    pub fn get_handler_id(&self, reservation_id: ReservationId) -> Option<ComponentId> {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_handler_id();
        } else {
            panic!("Reservation (id: {:?}) does not contain a handler id.", reservation_id);
        }
    }

    /// Returns the assigned_start of the provided reservation_id. Panics if no client id was found.
    pub fn get_assigned_start(&self, reservation_id: ReservationId) -> i64 {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_assigned_start();
        } else {
            panic!("Reservation (id: {:?}) does not contain a assigned end time.", reservation_id);
        }
    }

    /// Returns the assigned_end of the provided reservation_id. Panics if no client id was found.
    pub fn get_assigned_end(&self, reservation_id: ReservationId) -> i64 {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_assigned_end();
        } else {
            panic!("Reservation (id: {:?}) does not contain a assigned end time.", reservation_id);
        }
    }

    /// Returns the state of the provided reservation_id. Panics if no state was found.
    pub fn get_state(&self, reservation_id: ReservationId) -> ReservationState {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_state();
        } else {
            panic!("Reservation (id: {:?}) does not contain a assigned end time.", reservation_id);
        }
    }

    /// Returns the task_duration of the provided reservation_id. Panics if no state was found.
    pub fn get_task_duration(&self, reservation_id: ReservationId) -> i64 {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_task_duration();
        } else {
            panic!("Reservation (id: {:?}) does not contain a assigned end time.", reservation_id);
        }
    }

    /// Returns the ReservationProceeding state of the provided reservation_id. Panics if no state was found.
    pub fn get_reservation_proceeding(&self, reservation_id: ReservationId) -> ReservationProceeding {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_reservation_proceeding();
        } else {
            panic!("Reservation (id: {:?}) does not contain the ReservationProceeding value.", reservation_id);
        }
    }

    /// Returns the booking_interval_start of the provided reservation_id. Panics if no value was found.
    pub fn get_booking_interval_start(&self, reservation_id: ReservationId) -> i64 {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_booking_interval_start();
        } else {
            self.dump_store_contents();
            panic!("Reservation (id: {:?}) does not contain a booking interval start time.", reservation_id);
        }
    }

    /// Returns the booking_interval_end of the provided reservation_id. Panics if no value was found.
    pub fn get_booking_interval_end(&self, reservation_id: ReservationId) -> i64 {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_booking_interval_end();
        } else {
            panic!("Reservation (id: {:?}) does not contain a booking interval end time.", reservation_id);
        }
    }

    // Updates the frag_delta value of the corresponding reservation of the provided reservation_id.
    pub fn set_frag_delta(&mut self, reservation_id: ReservationId, frag_delta: f64) {
        if let Some(handle) = self.get(reservation_id) {
            let mut res = handle.write().unwrap();
            res.set_frag_delta(frag_delta);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id)
        }
    }

    // Updates the booking_interval_start value of the corresponding reservation of the provided reservation_id.
    pub fn set_booking_interval_start(&mut self, reservation_id: ReservationId, booking_interval_start: i64) {
        if let Some(handle) = self.get(reservation_id) {
            let mut res = handle.write().unwrap();
            res.set_booking_interval_start(booking_interval_start);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id)
        }
    }

    // Updates the booking_interval_end value of the corresponding reservation of the provided reservation_id.
    pub fn set_booking_interval_end(&mut self, reservation_id: ReservationId, booking_interval_end: i64) {
        if let Some(handle) = self.get(reservation_id) {
            let mut res = handle.write().unwrap();
            res.set_booking_interval_end(booking_interval_end);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id)
        }
    }

    // Updates the assigned_start value of the corresponding reservation of the provided reservation_id.
    pub fn set_assigned_start(&mut self, reservation_id: ReservationId, assigned_start: i64) {
        if let Some(handle) = self.get(reservation_id) {
            let mut res = handle.write().unwrap();
            res.set_assigned_start(assigned_start);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id)
        }
    }

    // Updates the assigned_end value of the corresponding reservation of the provided reservation_id.
    pub fn set_assigned_end(&mut self, reservation_id: ReservationId, assigned_end: i64) {
        if let Some(handle) = self.get(reservation_id) {
            let mut res = handle.write().unwrap();
            res.set_assigned_end(assigned_end);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id)
        }
    }

    // Updates the reserved_capacity value of the corresponding reservation of the provided reservation_id.
    pub fn set_reserved_capacity(&mut self, reservation_id: ReservationId, reserved_capacity: i64) {
        if let Some(handle) = self.get(reservation_id) {
            let mut res = handle.write().unwrap();
            res.set_reserved_capacity(reserved_capacity);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id)
        }
    }

    // Updates the task_duration value of the corresponding reservation of the provided reservation_id.
    pub fn set_task_duration(&mut self, reservation_id: ReservationId, task_duration: i64) {
        if let Some(handle) = self.get(reservation_id) {
            let mut res = handle.write().unwrap();
            res.set_task_duration(task_duration);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id)
        }
    }

    // Updates the is_moldable value of the corresponding reservation of the provided reservation_id.
    pub fn set_is_moldable(&mut self, reservation_id: ReservationId, is_moldable: bool) {
        if let Some(handle) = self.get(reservation_id) {
            let mut res = handle.write().unwrap();
            res.set_is_moldable(is_moldable);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id)
        }
    }

    /// Retrieves form the provided reservation id the is_moldable.
    pub fn is_moldable(&self, reservation_id: ReservationId) -> bool {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.is_moldable();
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id);
            return false;
        }
    }

    pub fn is_workflow(&self, reservation_id: ReservationId) -> bool {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return matches!(res.get_typ(), ReservationTyp::Workflow);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id);
            return false;
        }
    }

    pub fn is_link(&self, reservation_id: ReservationId) -> bool {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return matches!(res.get_typ(), ReservationTyp::Link);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id);
            return false;
        }
    }

    pub fn is_reservation_proceeding(&self, reservation_id: ReservationId, reservation_proceeding: ReservationProceeding) -> bool {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_reservation_proceeding() == reservation_proceeding;
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id);
            return false;
        }
    }

    pub fn get_typ(&self, reservation_id: ReservationId) -> Option<ReservationTyp> {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return Some(res.get_typ());
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id);
            return None;
        }
    }

    pub fn get_upward_rank(&self, reservation_id: ReservationId, average_link_speed: i64) -> Option<Vec<WorkflowNode>> {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            if let Some(workflow) = res.as_any().downcast_ref::<Workflow>() {
                return Some(workflow.clone().calculate_upward_rank(average_link_speed, self));
            } else {
                log::error!("Reservation {:?} is not a Workflow", reservation_id);
            }
        }

        return None;
    }

    /// Evaluates if a specific reservation has reached or exceeded a target
    /// level of commitment in the distributed lifecycle.
    ///
    /// The progression hierarchy is defined as:
    /// **Finished** > **Committed** > **ReserveAnswer** > **ProbeAnswer** >
    /// **Open** > **Deleted** > **Rejected**.
    ///
    /// # Parameters
    /// * `reservation_id` - The unique identifier of the reservation to check.
    /// * `state` - The minimum required `ReservationState` to compare against.
    ///
    /// # Returns
    /// Returns `true` if the current state of the reservation is equal to or
    /// higher than the provided `state`. Returns `false` if the ID is not found.
    pub fn is_reservation_state_at_least(&self, reservation_id: ReservationId, state: ReservationState) -> bool {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            if res.get_state() >= state { true } else { false }
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id);
            return false;
        }
    }

    pub fn adjust_capacity(&self, reservation_id: ReservationId, capacity: i64) {
        if let Some(handle) = self.get(reservation_id) {
            let mut res = handle.write().unwrap();
            res.adjust_capacity(capacity);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id)
        }
    }

    pub fn adjust_task_duration(&self, reservation_id: ReservationId, duration: i64) {
        if let Some(handle) = self.get(reservation_id) {
            let mut res = handle.write().unwrap();
            res.adjust_task_duration(duration);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id)
        }
    }

    /// Replaces a Reservation of the provided ReservationId with the provided Reservation.
    /// ReservationName, ClientId and HandlerId must be the same of the replacement Reservation.
    /// Return the true, if the replacement was a success otherwise false.
    pub fn replace_reservation(&mut self, reservation_id: ReservationId, new_reservation: Reservation) -> bool {
        let guard = self.inner.read().unwrap();
        let arc = guard.slots.get(reservation_id).unwrap();
        let mut current_res = arc.write().unwrap();

        if current_res.get_name() != new_reservation.get_name() {
            log::error!("FailedReservationReplacement: Cannot replace: ReservationName mismatch");
            return false;
        }
        if current_res.get_client_id() != new_reservation.get_client_id() {
            log::error!("FailedReservationReplacement: Cannot replace: ClientId mismatch");
            return false;
        }
        if current_res.get_handler_id() != new_reservation.get_handler_id() {
            log::error!("FailedReservationReplacement: Cannot replace: HandlerId mismatch");
            return false;
        }

        *current_res = new_reservation;

        log::info!(
            "ReservationReplacementWasSuccessful: ReservationId {:?} with Name {:?} was replaced.",
            reservation_id,
            self.get_name_for_key(reservation_id)
        );

        return true;
    }

    /// Update the state of a reservation.
    /// Triggers the notification listeners.
    pub fn update_state(&self, id: ReservationId, new_state: ReservationState) {
        let should_notify = {
            let guard = self.inner.read().unwrap();
            if let Some(res_lock) = guard.slots.get(id) {
                let mut res = res_lock.write().unwrap();
                res.set_state(new_state);
                true
            } else {
                false
            }
        };

        if should_notify {
            let listeners = {
                let guard = self.inner.read().unwrap();
                guard.listeners.clone()
            };

            for listener in listeners {
                listener.on_reservation_change(id, new_state);
            }
        }
    }

    /// Provides mutable access to a workflow for scheduling purposes.
    pub fn with_workflow_mut<F, R>(&self, reservation_id: ReservationId, f: F) -> Option<R>
    where
        F: FnOnce(&mut Workflow) -> R,
    {
        let handle = self.get(reservation_id).unwrap();
        let mut guard = handle.write().expect("Lock poisoned");

        guard.as_workflow_mut().map(f)
    }

    /// Sorts the provided Reservation Ids by there arrival time (ascending)
    pub fn get_sorted_res_ids_with_arrival_time(&self, reservation_ids: Vec<ReservationId>) -> Vec<(ReservationId, i64)> {
        let guard = self.inner.read().unwrap();

        let mut res_id_arrival_time_list = Vec::new();
        for res_id in reservation_ids {
            let res = guard.slots.get(res_id).expect("Reservation should exist in store.");
            res_id_arrival_time_list.push((res_id, res.read().expect("Lock poisoned").get_arrival_time()));
        }
        res_id_arrival_time_list.iter().is_sorted_by(|a, b| a.1 <= b.1);
        return res_id_arrival_time_list;
    }

    /// Creates a "Shadow" copy of the store.
    ///
    /// This creates a deep copy of all reservations to allow isolated modification.
    /// This means a Scheduler can work on the Shadow Store using the same Keys
    /// as the Master Store, but changes will not affect the Master.
    /// Note: ReservationStore snapshot has no active Listeners.
    pub fn snapshot(&self) -> ReservationStore {
        let guard = self.inner.read().unwrap();

        let mut new_slots = SlotMap::with_key();
        new_slots = guard.slots.clone();

        for (key, arc_lock) in new_slots.iter_mut() {
            let original_res = arc_lock.read().expect("Lock poisoned during snapshot").clone();
            *arc_lock = Arc::new(RwLock::new(original_res));
        }

        let new_inner = StoreInner {
            slots: new_slots,
            name_index: guard.name_index.clone(),
            client_index: guard.client_index.clone(),
            handler_index: guard.handler_index.clone(),
            listeners: guard.listeners.clone(),
        };

        ReservationStore { inner: Arc::new(RwLock::new(new_inner)) }
    }

    /// Iterates through all reservations and logs their ID and Name to the error log.
    pub fn dump_store_contents(&self) {
        let guard = self.inner.read().expect("RwLock poisoned");
        log::error!("=== RESERVATION STORE DUMP ({} entries) ===", guard.slots.len());

        for (id, res_handle) in &guard.slots {
            // We attempt to read the reservation name directly from the object
            match res_handle.read() {
                Ok(res) => {
                    log::error!("  -> ID: {:?} | Name: {:?} | State: {:?}", id, res.get_name(), res.get_state());
                }
                Err(_) => {
                    log::error!("  -> ID: {:?} | [Lock Poisoned]", id);
                }
            }
        }
        log::error!("=== END OF DUMP ===");
    }
}
