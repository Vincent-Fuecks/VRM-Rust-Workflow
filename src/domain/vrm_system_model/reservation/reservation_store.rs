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
    listener: Arc<dyn NotificationListener>,
}

impl ReservationStore {
    pub fn new(listener: Option<Arc<dyn NotificationListener>>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(StoreInner {
                slots: SlotMap::with_key(),
                name_index: HashMap::new(),
                client_index: HashMap::new(),
                handler_index: HashMap::new(),
                listener: listener.unwrap_or_else(|| Arc::new(NoOpenListener)),
            })),
        }
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

    /// Get Reservation with internal Id (ReservationId).
    ///  
    /// # Returns
    /// Returns the Some(Reservation) if ReservationId was present in SlotMap else return None.  
    pub fn get(&self, key: ReservationId) -> Option<Arc<RwLock<Reservation>>> {
        let guard = self.inner.read().expect("RwLock poisoned");
        guard.slots.get(key).cloned()
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
                return Some(workflow.clone().calculate_upward_rank(average_link_speed));
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

    /// Update the state of a reservation.
    /// Triggers the notification listener.
    pub fn update_state(&self, id: ReservationId, new_state: ReservationState) {
        // We scope the lock to be as short as possible
        let notify = {
            let guard = self.inner.read().unwrap();
            if let Some(res_lock) = guard.slots.get(id) {
                let mut res = res_lock.write().unwrap();
                res.set_state(new_state);
                true
            } else {
                false
            }
        };

        if notify {
            let guard = self.inner.read().unwrap();
            guard.listener.on_reservation_change(id, new_state);
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

    /// Creates a "Shadow" copy of the store.
    ///
    /// This creates a deep copy of all reservations.
    /// This means a Scheduler can work on the Shadow Store using the same Keys
    /// as the Master Store, but changes will not affect the Master.
    pub fn snapshot(&self) -> ReservationStore {
        let guard = self.inner.read().unwrap();

        let new_slots = guard.slots.clone();

        let new_inner = StoreInner {
            slots: new_slots,
            name_index: guard.name_index.clone(),
            client_index: guard.client_index.clone(),
            handler_index: guard.handler_index.clone(),
            listener: Arc::new(NoOpenListener), // TODO Shadows SlottedSchedule should not notify anyone or?
        };

        ReservationStore { inner: Arc::new(RwLock::new(new_inner)) }
    }
}
