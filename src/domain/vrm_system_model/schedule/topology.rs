use crate::api::vrm_system_model_dto::aci_dto::RMSSystemDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use crate::domain::vrm_system_model::resource::link_resource::LinkResource;
use crate::domain::vrm_system_model::resource::resource_trait::{Resource, ResourceId};
use crate::domain::vrm_system_model::schedule::slotted_schedule::SlottedSchedule;
use crate::domain::vrm_system_model::utils::id::{LinkResourceId, RouterId, SlottedScheduleId};
use crate::error::ConversionError;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

/// The number of k shortest paths to calculate and cache between any two grid access points.
const K_NUMBER_OF_PATHS: usize = 10;

/// Represents a physical router within the grid network.
#[derive(Debug, Clone)]
pub struct Router {
    /// The unique identifier for the router.
    pub id: RouterId,

    /// Indicates whether this router serves as an entry/exit point for resources.
    ///
    /// If `true`, this router is directly connected to a Grid Node (Resource), making it
    /// a potential source for `VirtualLinkResource` calculations.
    pub is_grid_access_point: bool,
}

/// Represents a specific route through the network, consisting of a sequence of network links.
///
/// A `Path` is the physical realization of a connection between two routers. It is
/// composed of a vector of `LinkResourceId`s that must be traversed in order.
#[derive(Debug, Clone)]
pub struct Path {
    pub network_links: Vec<LinkResourceId>,
}

impl Path {
    pub fn new() -> Self {
        Self { network_links: Vec::new() }
    }
}

/// Represents the calculated virtual resource capability between two endpoints.
///
/// Unlike a physical link, a `VirtualLinkResource` represents the aggregated potential
/// connectivity between two Grid Access Points. It summarizes the capacity available
/// across the calculated K-shortest paths.
#[derive(Debug, Clone)]
pub struct VirtualLinkResource {
    /// Represents source Router form which the link starts
    pub source_router: RouterId,

    /// Represents the target, where the link leads.
    pub target_router: RouterId,

    /// The aggregated sum of the minimum bottleneck capacity of the k-shortest paths.
    /// This value represents the total theoretical throughput if all K paths were used simultaneously
    pub capacity: i64,

    /// The average bandwidth capacity across the calculated k-shortest paths.
    pub avg_bandwidth: i64,
}

/// Models the complete grid network topology.
///
/// The `NetworkTopology` acts as the graph representation of the system. It manages:
/// * **Physical Layer**: Routers and Network Links.
/// * **Connectivity**: Adjacency matrices defining how routers connect.
/// * **Routing Logic**: Caching of K-shortest paths and calculation of virtual resources.
/// * **Heuristics**: Importance databases for link weighting.
#[derive(Debug)]
pub struct NetworkTopology {
    /// A map of all routers in the system, indexed by their ID.
    routers: HashMap<RouterId, Router>,

    /// A map of all physical network links, indexed by their ID.
    network_links: HashMap<LinkResourceId, LinkResource>,

    /// The adjacency list representing the graph structure.
    /// Maps a `RouterId` to a set of outgoing `LinkResourceId`s, enabling efficient graph traversal.
    adjacency: HashMap<RouterId, HashSet<LinkResourceId>>,

    /// Stores heuristic weights for network links.
    /// Values represent how influential a link was in previous reservations.
    importance_database: HashMap<LinkResourceId, f64>,

    /// A cache storing the calculated K-shortest paths between pairs of routers.
    pub path_cache: HashMap<(RouterId, RouterId), Vec<Path>>,

    /// Stores the "virtual" resources created for endpoint pairs
    pub virtual_link_resources: Vec<VirtualLinkResource>,

    /// Tracks maximum bandwidth across all calculated paths (highest bottleneck bandwidth on all the found paths)
    pub max_bandwidth_all_paths: i64,
}

impl TryFrom<(RMSSystemDto, Arc<dyn SystemSimulator>, String, ReservationStore)> for NetworkTopology {
    type Error = ConversionError;

    fn try_from(args: (RMSSystemDto, Arc<dyn SystemSimulator>, String, ReservationStore)) -> Result<Self, Self::Error> {
        let (dto, simulator, _, reservation_store) = args;

        // 1.  Init physical links.
        let (network_links, importance_database) = NetworkTopology::setup_network_links(&dto, simulator.clone(), reservation_store);

        // 2.  Init router instances based on grid nodes and network link endpoints.
        let routers: HashMap<RouterId, Router> = NetworkTopology::setup_routers(&dto);

        // 3. Build the adjacency matrix
        let adjacency: HashMap<RouterId, HashSet<LinkResourceId>> = NetworkTopology::setup_adjacency_matrix(&network_links, &routers);

        let mut topology = NetworkTopology {
            routers,
            network_links,
            adjacency,
            importance_database,
            path_cache: HashMap::new(),
            virtual_link_resources: Vec::new(),
            max_bandwidth_all_paths: 0,
        };

        // 4.  Pre-calculating all K-shortest paths between Grid Access Points.
        topology.calc_all_paths();

        Ok(topology)
    }
}

impl NetworkTopology {
    pub fn new() -> Self {
        Self {
            routers: HashMap::new(),
            network_links: HashMap::new(),
            adjacency: HashMap::new(),
            importance_database: HashMap::new(),
            path_cache: HashMap::new(),
            virtual_link_resources: Vec::new(),
            max_bandwidth_all_paths: 0,
        }
    }
    /// Calculates the K-shortest paths between the source and target router using a Breadth-First Search (BFS) approach.
    /// # Returns
    ///
    /// Returns `Some(VirtualLinkResource)` if at least one path is found, otherwise `None`.
    fn calc_k_shortest_paths(&mut self, source_router: Router, target_router: Router) -> Option<VirtualLinkResource> {
        let mut found_solutions = Vec::new();
        let mut queue: VecDeque<Path> = VecDeque::new();

        // Initialize queue with all outgoing network links from source
        if let Some(outgoing_links) = self.adjacency.get(&source_router.id) {
            for link_id in outgoing_links {
                if self.network_links.contains_key(link_id) {
                    let mut p = Path::new();
                    p.network_links.push(link_id.clone());
                    queue.push_back(p);
                }
            }
        }

        while let Some(current_path) = queue.pop_front() {
            let current_last_network_link = current_path.network_links.last().expect("Path should not be empty");

            let last_network_link = self.network_links.get(current_last_network_link).expect("Network Link should exist.");

            let current_target_router_id = last_network_link.target.clone();

            if current_target_router_id.eq(&target_router.id) {
                found_solutions.push(current_path);

                if found_solutions.len() >= K_NUMBER_OF_PATHS {
                    break;
                }
            } else if self.adjacency.contains_key(&current_target_router_id) {
                for outgoing_link_id in self.adjacency.get(&current_target_router_id).unwrap().clone().iter() {
                    let outgoing_link_target_id = self.network_links.get(&outgoing_link_id).unwrap().target.clone();

                    let mut is_loop: bool = false;

                    for old_part_id in &current_path.network_links {
                        let old_part_source_id = self.network_links.get(&old_part_id).unwrap().source.clone();

                        if old_part_source_id == outgoing_link_target_id {
                            is_loop = true;
                            break;
                        }
                    }

                    if !is_loop {
                        let mut new_path = current_path.clone();
                        new_path.network_links.push(outgoing_link_id.clone());
                        queue.push_back(new_path);
                    }
                }
            }
        }

        if found_solutions.is_empty() {
            log::debug!("NoPathFound: {} => {}", source_router.id, target_router.id);
            return None;
        }

        self.path_cache.insert((source_router.id.clone(), target_router.id.clone()), found_solutions.clone());

        let mut total_bw: i64 = 0;

        for solution in &found_solutions {
            let mut bandwidth_bottleneck = i64::MAX;

            // Find bottleneck (min capacity) of this path
            for link_id in &solution.network_links {
                let link = self.network_links.get(link_id).unwrap();
                if link.get_capacity() < bandwidth_bottleneck {
                    bandwidth_bottleneck = link.get_capacity();
                }
            }

            // Update global max bandwidth tracking
            if bandwidth_bottleneck > self.max_bandwidth_all_paths {
                self.max_bandwidth_all_paths = bandwidth_bottleneck;
            }

            total_bw += bandwidth_bottleneck;
        }

        log::debug!("Paths found {} => {}: {} solutions, Max/Sum BW: {}", source_router.id, target_router.id, found_solutions.len(), total_bw);

        Some(VirtualLinkResource {
            source_router: source_router.id.clone(),
            target_router: target_router.id.clone(),
            capacity: total_bw,
            avg_bandwidth: total_bw / found_solutions.len() as i64,
        })
    }

    /// Performs preprocessing to identify and cache all paths between Grid Access Points.
    /// This iterates through all router pairs. If both routers are marked as `is_grid_access_point`,
    /// it invokes `calc_k_shortest_paths` and stores the resulting `VirtualLinkResource`.
    pub fn calc_all_paths(&mut self) {
        let router_ids: Vec<RouterId> = self.routers.keys().cloned().collect();

        for source_id in &router_ids {
            let source_router = self.routers.get(source_id).unwrap().clone();
            if !source_router.is_grid_access_point {
                continue;
            }

            for target_id in &router_ids {
                let target_router = self.routers.get(target_id).unwrap().clone();
                if !target_router.is_grid_access_point || source_id.eq(&target_id) {
                    continue;
                }

                log::debug!("Searching paths: Source Router Id: {} -> Target Router Id: {}", source_id, target_id);

                if let Some(virtual_link) = self.calc_k_shortest_paths(source_router.clone(), target_router.clone()) {
                    self.virtual_link_resources.push(virtual_link);
                }
            }
        }
    }

    /// Constructs the adjacency matrix for the network graph.
    pub fn setup_adjacency_matrix(
        network_links: &HashMap<LinkResourceId, LinkResource>,
        routers: &HashMap<RouterId, Router>,
    ) -> HashMap<RouterId, HashSet<LinkResourceId>> {
        let mut adjacency: HashMap<RouterId, HashSet<LinkResourceId>> = HashMap::new();

        for (network_link_id, network_link) in network_links {
            let source: RouterId = network_link.source.clone();
            let target: RouterId = network_link.target.clone();

            let mut source_found: bool = false;
            let mut target_found: bool = false;

            for (router_id, _) in routers.iter() {
                if router_id.eq(&source) {
                    source_found = true;

                    let resource_id = network_link.get_id();

                    if let ResourceId::Link(link_id) = resource_id {
                        adjacency.entry(source.clone()).or_insert_with(HashSet::new).insert(link_id);
                    } else {
                        panic!("Expected a LinkResource ID but found {:?}", resource_id);
                    }

                    if target_found {
                        break;
                    }
                }

                if router_id.eq(&target) {
                    target_found = true;

                    if source_found {
                        break;
                    }
                }
            }

            if !source_found {
                log::error!("InValidLinkNetworkConfiguration: The Source: {} was not found.", source)
            }

            if !target_found {
                log::error!("InValidLinkNetworkConfiguration: The Target: {} was not found.", target)
            }
        }

        return adjacency;
    }

    /// Derives the set of all Routers from the DTO configurations (GirdNodes, LinkResources).
    pub fn setup_routers(dto: &RMSSystemDto) -> HashMap<RouterId, Router> {
        let mut routers: HashMap<RouterId, Router> = HashMap::new();

        for grid_node in dto.grid_nodes.iter() {
            for router_name in grid_node.connected_to_router.iter() {
                let router_id = RouterId::new(router_name);

                if !routers.contains_key(&router_id) {
                    routers.insert(router_id.clone(), Router { id: router_id, is_grid_access_point: true });
                }
            }
        }

        for network_link in dto.network_links.iter() {
            let router_end_point_id = RouterId::new(network_link.end_point.clone());
            let router_start_point_id = RouterId::new(network_link.start_point.clone());

            if !routers.contains_key(&router_end_point_id) {
                routers.insert(router_end_point_id.clone(), Router { id: router_end_point_id, is_grid_access_point: false });
            }

            if !routers.contains_key(&router_start_point_id) {
                routers.insert(router_start_point_id.clone(), Router { id: router_start_point_id, is_grid_access_point: false });
            }
        }

        return routers;
    }

    /// Initializes all `LinkResource` structs and the importance database.
    pub fn setup_network_links(
        dto: &RMSSystemDto,
        simulator: Arc<dyn SystemSimulator>,
        reservation_store: ReservationStore,
    ) -> (HashMap<LinkResourceId, LinkResource>, HashMap<LinkResourceId, f64>) {
        let mut network_links: HashMap<LinkResourceId, LinkResource> = HashMap::new();
        let mut importance_database: HashMap<LinkResourceId, f64> = HashMap::new();

        for link in dto.network_links.iter() {
            let link_schedule_name = format!("Schedule LinkResource {} -> {}", link.start_point, link.end_point);
            let link_schedule = SlottedSchedule::new(
                SlottedScheduleId::new(link_schedule_name),
                dto.num_of_slots,
                dto.slot_width,
                link.capacity,
                true,
                simulator.clone_box().into(),
                reservation_store.clone(),
            );
            let network_link_id: LinkResourceId = LinkResourceId::new(link.id.clone());
            network_links.insert(
                network_link_id.clone(),
                LinkResource::new(
                    network_link_id.clone(),
                    HashSet::new(),
                    RouterId::new(link.start_point.clone()),
                    RouterId::new(link.end_point.clone()),
                    link.capacity,
                    link.capacity,
                    link_schedule,
                ),
            );

            importance_database.insert(network_link_id, 1.0);
        }

        if network_links.is_empty() {
            log::info!("Empty NullBroker Network: The newly created NullBroker contains no Network. NullRms should be utilized instead.");
        }
        return (network_links, importance_database);
    }
}
