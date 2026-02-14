use std::sync::{Arc, RwLock};

use slotmap::{SlotMap, new_key_type};

use crate::domain::vrm_system_model::{
    resource::{link_resource::LinkResource, node_resource::NodeResource, resource_trait::Resource},
    schedule::slotted_schedule::slotted_schedule::SlottedSchedule,
    scheduler_trait::Schedule,
    utils::id::{ResourceName, RouterId},
};

new_key_type! {
    pub struct NodeResourceId;
    pub struct LinkResourceId;
}

#[derive(Debug, Clone)]
pub struct ResourceStore {
    /// Both maps are protected with a single lock.
    inner: Arc<RwLock<StoreInner>>,
}

#[derive(Debug, Default, Clone)]
struct StoreInner {
    nodes: SlotMap<NodeResourceId, Arc<RwLock<NodeResource>>>,
    links: SlotMap<LinkResourceId, Arc<RwLock<LinkResource>>>,
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
}
