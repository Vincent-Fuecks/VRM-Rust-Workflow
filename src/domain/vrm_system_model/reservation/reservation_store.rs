use slotmap::{SlotMap, new_key_type};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::utils::id::ReservationName;

new_key_type! {
    pub struct ReservationId;
}

#[derive(Debug)]
struct StoreInner {
    /// Reservation Storage.
    slots: SlotMap<ReservationId, Arc<RwLock<Box<dyn Reservation>>>>,

    /// Index lookup InternalKey (ReservationId) using input reservation name (ReservationName).
    name_index: HashMap<ReservationName, ReservationId>,
}

#[derive(Debug)]
pub struct ReservationStore {
    /// Both maps are protected with a single lock.
    inner: Arc<RwLock<StoreInner>>,
}

impl ReservationStore {
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(StoreInner { slots: SlotMap::with_key(), name_index: HashMap::new() })) }
    }

    /// Adds Reservation to ReservationStore.
    ///
    /// # Returns
    /// Returns the ReservationId (internal Key for ReservationStore).
    pub fn add(&self, reservation: Box<dyn Reservation>) -> ReservationId {
        let mut guard = self.inner.write().expect("RwLock poisoned");
        let name_clone = reservation.get_name().clone();
        let key = guard.slots.insert(Arc::new(RwLock::new(reservation)));

        guard.name_index.insert(name_clone, key);

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
}
