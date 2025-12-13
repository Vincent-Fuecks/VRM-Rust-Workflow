use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;
use crate::domain::vrm_system_model::resource::network_link::NetworkLink;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Router {
    pub id: ReservationKey,
    pub is_endpoint: bool,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub links: Vec<ReservationKey>,
}

pub struct NetworkTopology {
    routers: HashMap<ReservationKey, Router>,
    network_links: HashMap<ReservationKey, NetworkLink>,
    adjacency: HashMap<ReservationKey, Vec<ReservationKey>>,
    path_cache: HashMap<(ReservationKey, ReservationKey), Vec<Path>>,

    next_router_id: usize,
    next_link_id: usize,
}

impl NetworkTopology {
    pub fn new() -> Self {
        Self {
            routers: HashMap::new(),
            network_links: HashMap::new(),
            adjacency: HashMap::new(),
            path_cache: HashMap::new(),
            next_router_id: 0,
            next_link_id: 0,
        }
    }

    pub fn get_link_mut(&mut self, id: ReservationKey) -> Option<&mut NetworkLink> {
        self.network_links.get_mut(&id)
    }

    pub fn get_paths(&self, start: ReservationKey, end: ReservationKey) -> Option<&Vec<Path>> {
        self.path_cache.get(&(start, end))
    }

    pub fn add_network_link() {}
}
