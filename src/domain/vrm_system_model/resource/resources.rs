use std::collections::HashSet;

use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::resource::link_resource::LinkResource;
use crate::domain::vrm_system_model::resource::node_resource::NodeResource;
use crate::domain::vrm_system_model::resource::resource_trait::Resource;
use crate::domain::vrm_system_model::utils::id::RouterId;

#[derive(Debug, Clone)]
pub struct BaseResource<ID> {
    pub id: ID,
    pub capacity: i64,
    pub connected_routers: HashSet<RouterId>,
}

impl<ID: Clone> BaseResource<ID> {
    pub fn new(id: ID, capacity: i64, connected_routers: HashSet<RouterId>) -> Self {
        Self { id, capacity, connected_routers: connected_routers }
    }

    pub fn can_handle_capacity(&self, reservation: &Box<dyn Reservation>) -> bool {
        if !reservation.is_moldable() && reservation.get_reserved_capacity() > 0 {
            reservation.get_reserved_capacity() <= self.capacity
        } else {
            true
        }
    }

    pub fn get_id(&self) -> ID {
        self.id.clone()
    }
}

#[derive(Debug)]
pub struct Resources {
    inner: Vec<Box<dyn Resource>>,

    // TODO What if all routers are not connected? We have multiple clusters --> Is this allowed?
    /// Router list
    router_list: Vec<RouterId>,
}

impl Resources {
    pub fn new(inner: Vec<Box<dyn Resource>>, router_list: Vec<RouterId>) -> Self {
        Self { inner: inner, router_list: router_list }
    }

    pub fn add(&mut self, resource: Box<dyn Resource>, router_list: HashSet<RouterId>) {
        self.inner.push(resource);
        self.router_list.extend(router_list);
    }

    pub fn get_total_capacity(&self) -> i64 {
        self.inner.iter().map(|r| r.get_capacity()).sum()
    }

    pub fn can_handle(&self, reservation: &Box<dyn Reservation>) -> bool {
        // TODO value clone --> problem
        for resource in &self.inner {
            if resource.can_handle(reservation) {
                return true;
            }
        }
        false
    }

    /// Returns the number of NodeResources in Resources
    pub fn get_node_resource_count(&self) -> usize {
        self.inner.iter().filter(|r| r.as_any().is::<NodeResource>()).count()
    }

    /// Returns the number of LinkResources in Resources
    pub fn get_link_resource_count(&self) -> usize {
        self.inner.iter().filter(|r| r.as_any().is::<LinkResource>()).count()
    }

    /// Returns
    pub fn contains_router(&self, router_id: RouterId) -> bool {
        return self.router_list.contains(&router_id);
    }
}
