use std::collections::HashSet;

use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::resource::link_resource::LinkResource;
use crate::domain::vrm_system_model::resource::node_resource::NodeResource;
use crate::domain::vrm_system_model::resource::resource_trait::Resource;
use crate::domain::vrm_system_model::utils::id::{ResourceName, RouterId};

#[derive(Debug, Clone)]
pub struct BaseResource {
    pub name: ResourceName,
    pub capacity: i64,
}

impl BaseResource {
    pub fn new(name: ResourceName, capacity: i64) -> Self {
        Self { name, capacity }
    }

    pub fn can_handle_adc_capacity_request(&self, res: Reservation) -> bool {
        if res.get_base_reservation().is_moldable() && res.get_base_reservation().get_reserved_capacity() > 0 {
            res.get_base_reservation().get_reserved_capacity() <= self.capacity
        } else {
            true
        }
    }

    pub fn can_handle_aci_capacity_request(&self, reservation_store: ReservationStore, reservation_id: ReservationId) -> bool {
        if reservation_store.is_moldable(reservation_id) && reservation_store.get_reserved_capacity(reservation_id) > 0 {
            reservation_store.get_reserved_capacity(reservation_id) <= self.capacity
        } else {
            true
        }
    }

    pub fn get_name(&self) -> ResourceName {
        self.name.clone()
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

    pub fn can_handle_adc_request(&self, res: Reservation) -> bool {
        for resource in &self.inner {
            if resource.can_handle_adc_capacity_request(res.clone()) {
                return true;
            }
        }
        false
    }

    pub fn can_handle_aci_request(&self, reservation_store: ReservationStore, reservation_id: ReservationId) -> bool {
        for resource in &self.inner {
            if resource.can_handle_aci_capacity_request(reservation_store.clone(), reservation_id) {
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

    /// Returns the aggregated sum of the link resource capacities
    pub fn get_total_link_capacity(&self) -> i64 {
        self.inner.iter().filter_map(|r| r.as_any().downcast_ref::<LinkResource>()).map(|link| link.get_capacity()).sum()
    }

    /// Returns the aggregated sum of the node resource capacities
    pub fn get_total_node_capacity(&self) -> i64 {
        self.inner.iter().filter_map(|r| r.as_any().downcast_ref::<NodeResource>()).map(|node: &NodeResource| node.get_capacity()).sum()
    }

    /// Returns true, if provided RouterId exists in Router-List of RMS
    pub fn contains_router(&self, router_id: RouterId) -> bool {
        return self.router_list.contains(&router_id);
    }

    /// Return the, the list of all RouterIds, which the Resource contains
    pub fn get_router_list(&self) -> Vec<RouterId> {
        return self.router_list.clone();
    }
}
