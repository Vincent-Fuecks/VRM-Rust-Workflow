use slotmap::{SlotMap, new_key_type};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationState};
use crate::domain::vrm_system_model::utils::id::{ClientId, ComponentId, ReservationName};

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

///
#[derive(Debug, Clone)]
pub struct ReservationStore {
    /// Both maps are protected with a single lock.
    inner: Arc<RwLock<StoreInner>>,
}

#[derive(Debug, Clone)]
struct StoreInner {
    /// Reservation Storage.
    slots: SlotMap<ReservationId, Arc<RwLock<Box<dyn Reservation>>>>,

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
    pub fn add(&self, reservation: Box<dyn Reservation>) -> ReservationId {
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
    pub fn get(&self, key: ReservationId) -> Option<Arc<RwLock<Box<dyn Reservation>>>> {
        let guard = self.inner.read().expect("RwLock poisoned");
        guard.slots.get(key).cloned()
    }

    /// Get Reservation with User reservation name (ReservationName).
    ///  
    /// # Returns
    /// Returns Some(Reservation) if ReservationName was present in SlotMap else return None.  
    pub fn get_by_name(&self, name: &ReservationName) -> Option<Arc<RwLock<Box<dyn Reservation>>>> {
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

    pub fn get_reserved_capacity(&self, reservation_id: ReservationId) -> i64 {
        if let Some(handle) = self.get(reservation_id) {
            let res = handle.read().unwrap();
            return res.get_reserved_capacity();
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", reservation_id);
            return 0;
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
