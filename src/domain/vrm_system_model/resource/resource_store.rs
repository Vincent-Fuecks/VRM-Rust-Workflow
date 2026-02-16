use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use slotmap::{SlotMap, new_key_type};

use crate::domain::vrm_system_model::{
    reservation::{
        reservation::{Reservation, ReservationTrait},
        reservation_store::{ReservationId, ReservationStore},
    },
    resource::{link_resource::LinkResource, node_resource::NodeResource, resource_trait::Resource},
    schedule::slotted_schedule::slotted_schedule::SlottedSchedule,
    utils::id::{ResourceName, RouterId},
};

new_key_type! {
    pub struct NodeResourceId;
    pub struct LinkResourceId;
}

#[derive(Debug, Clone)]
pub struct ResourceStore {
    inner: Arc<RwLock<StoreInner>>,
}

#[derive(Debug, Default, Clone)]
struct StoreInner {
    nodes: SlotMap<NodeResourceId, Arc<RwLock<NodeResource>>>,
    links: SlotMap<LinkResourceId, Arc<RwLock<LinkResource>>>,

    name_index: HashMap<RouterId, LinkResourceId>,
}

impl ResourceStore {
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(StoreInner::default())) }
    }

    //---------------------
    // --- Node Methods ---
    //---------------------
    pub fn add_node(&self, node: NodeResource) -> NodeResourceId {
        let mut guard = self.inner.write().unwrap();
        guard.nodes.insert(Arc::new(RwLock::new(node)))
    }

    pub fn get_node(&self, node_id: NodeResourceId) -> Option<Arc<RwLock<NodeResource>>> {
        let guard = self.inner.read().unwrap();
        guard.nodes.get(node_id).cloned()
    }

    pub fn get_total_node_capacity(&self) -> i64 {
        let guard = self.inner.read().unwrap();
        guard.nodes.values().map(|node| node.read().unwrap().get_capacity()).sum()
    }

    pub fn get_num_of_nodes(&self) -> i64 {
        let guard = self.inner.read().unwrap();
        guard.nodes.len() as i64
    }

    fn can_handle_node_request(&self, is_res_moldable: bool, res_reserved_capacity: i64) -> bool {
        let guard = self.inner.read().unwrap();

        for node in guard.nodes.values() {
            let node = node.read().unwrap();

            if node.base.can_handle_adc_capacity_request(is_res_moldable, res_reserved_capacity) {
                return true;
            }
        }
        return false;
    }

    //---------------------
    // --- Link Methods ---
    //---------------------
    pub fn add_link(&self, link: LinkResource) -> LinkResourceId {
        let mut guard = self.inner.write().unwrap();
        guard.links.insert(Arc::new(RwLock::new(link)))
    }

    pub fn get_link(&self, link_id: LinkResourceId) -> Option<Arc<RwLock<LinkResource>>> {
        let guard = self.inner.read().unwrap();
        guard.links.get(link_id).cloned()
    }

    pub fn get_mut_link(&self, link_id: LinkResourceId) -> Option<Arc<RwLock<LinkResource>>> {
        let mut guard = self.inner.write().unwrap();
        guard.links.get_mut(link_id).cloned()
    }

    pub fn get_source(&self, link_id: LinkResourceId) -> RouterId {
        if let Some(handle) = self.get_link(link_id) {
            let link = handle.read().unwrap();
            return link.source.clone();
        } else {
            panic!("LinkResource (id: {:?}) was not found in the ResourceStore.", link_id);
        }
    }

    pub fn get_target(&self, link_id: LinkResourceId) -> RouterId {
        if let Some(handle) = self.get_link(link_id) {
            let link = handle.read().unwrap();
            return link.target.clone();
        } else {
            panic!("LinkResource (id: {:?}) was not found in the ResourceStore.", link_id);
        }
    }

    pub fn get_name(&self, link_id: LinkResourceId) -> ResourceName {
        if let Some(handle) = self.get_link(link_id) {
            let link = handle.read().unwrap();
            return link.get_name();
        } else {
            panic!("LinkResource (id: {:?}) was not found in the ResourceStore.", link_id);
        }
    }

    pub fn get_capacity(&self, link_id: LinkResourceId) -> i64 {
        if let Some(handle) = self.get_link(link_id) {
            let link = handle.read().unwrap();
            return link.get_capacity();
        } else {
            panic!("LinkResource (id: {:?}) was not found in the ResourceStore.", link_id);
        }
    }

    pub fn get_total_link_capacity(&self) -> i64 {
        let guard = self.inner.read().unwrap();
        guard.links.values().map(|link| link.read().unwrap().get_capacity()).sum()
    }

    pub fn get_num_of_links(&self) -> usize {
        let guard = self.inner.read().unwrap();
        guard.links.len()
    }

    pub fn with_mut_schedule<F, R>(&self, link_id: LinkResourceId, f: F) -> R
    where
        F: FnOnce(&mut SlottedSchedule) -> R,
    {
        if let Some(handle) = self.get_mut_link(link_id) {
            let mut link = handle.write().unwrap();
            f(&mut link.schedule)
        } else {
            panic!("LinkResource {:?} not found", link_id);
        }
    }

    fn can_handle_link_request(
        &self,
        link_source: Option<RouterId>,
        link_target: Option<RouterId>,
        is_res_moldable: bool,
        res_reserved_capacity: i64,
    ) -> bool {
        if link_source.is_none() || link_target.is_none() {
            return false;
        }

        let link_source = link_source.unwrap();
        let link_target = link_target.unwrap();

        let guard = self.inner.read().unwrap();

        for link in guard.links.values() {
            let link = link.read().unwrap();

            if link.source != link_source || link.target != link_target {
                return false;
            } else if link.base.can_handle_adc_capacity_request(is_res_moldable, res_reserved_capacity) {
                return true;
            }
        }
        return false;
    }

    //----------------------------
    // --- Aggregation Methods ---
    //----------------------------

    pub fn get_total_capacity(&self) -> i64 {
        self.get_total_link_capacity() + self.get_total_node_capacity()
    }

    /// TODO Currently also works for Workflows, is this right?
    /// Returns true if a resource can handle the reservation
    pub fn can_handle_adc_request(&self, res: Reservation) -> bool {
        match res.as_link() {
            Some(link_reservation) => self.can_handle_link_request(
                link_reservation.get_start_point(),
                link_reservation.get_end_point(),
                link_reservation.is_moldable(),
                link_reservation.get_reserved_capacity(),
            ),
            None => self.can_handle_node_request(res.is_moldable(), res.get_reserved_capacity()),
        }
    }

    /// TODO Currently also works for Workflows, is this right?
    /// Returns true if a resource can handle the reservation
    pub fn can_handle_aci_request(&self, reservation_store: ReservationStore, reservation_id: ReservationId) -> bool {
        if reservation_store.is_link(reservation_id) {
            return self.can_handle_link_request(
                reservation_store.get_start_point(reservation_id),
                reservation_store.get_end_point(reservation_id),
                reservation_store.is_moldable(reservation_id),
                reservation_store.get_reserved_capacity(reservation_id),
            );
        } else {
            return self
                .can_handle_node_request(reservation_store.is_moldable(reservation_id), reservation_store.get_reserved_capacity(reservation_id));
        }
    }
}
