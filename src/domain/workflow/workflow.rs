use std::collections::HashMap;

use crate::api::workflow_dto::reservation_dto::{ReservationProceedingDto, ReservationStateDto};
use crate::api::workflow_dto::workflow_dto::{TaskDto, WorkflowDto};
use crate::domain::workflow::co_allocation::CoAllocation;
use crate::domain::workflow::dependency::{CoAllocationDependency, DataDependency, SyncDependency};
use crate::domain::workflow::reservation::{
    LinkReservation, NodeReservation, ReservationBase, ReservationProceeding, ReservationState,
};
use crate::domain::workflow::workflow_node::WorkflowNode;
use crate::error::Error;

use union_find::{QuickUnionUf, UnionBySize, UnionFind};

#[derive(Debug, Clone)]
pub struct Workflow {
    pub base: ReservationBase,

    // The graph components, stored in HashMaps
    pub nodes: HashMap<String, WorkflowNode>,
    pub data_dependencies: HashMap<String, DataDependency>,
    pub sync_dependencies: HashMap<String, SyncDependency>,

    // The CoAllocations are later utilized for scheduling.
    pub co_allocations: HashMap<String, CoAllocation>,
    pub co_allocation_dependencies: HashMap<String, CoAllocationDependency>,

    /// Keys to Workflow.nodes
    pub entry_nodes: Vec<String>,

    /// Keys to Workflow.nodes
    pub exit_nodes: Vec<String>,

    /// Keys to Workflow.co_allocation
    pub entry_co_allocation: Vec<String>,

    /// Keys to Workflow.co_allocation
    pub exit_co_allocation: Vec<String>,
}

// A temporary struct to hold dependencies that have a source but no target yet.
#[derive(Debug, Clone)]
enum DanglingDependency {
    Data(DataDependency),
    Sync(SyncDependency),
}

/// Constructs a complete Workflow graph from a WorkflowDto.
///
/// This is the main entry point for parsing a DTO into the internal domain model.
/// Also builds the **CoAllocation graph**, which is later utilized for scheduling.
impl TryFrom<WorkflowDto> for Workflow {
    type Error = Error;

    fn try_from(dto: WorkflowDto) -> Result<Self, Self::Error> {
        // Phase 0: Create the base workflow object
        let base = Self::build_base_workflow(&dto);

        // Phase 1: Create all WorkflowNodes from the DTO tasks
        let mut nodes = Self::generate_workflow_nodes(&dto);

        // Phase 2: Create all Data and Sync dependencies from DTO
        let (data_dependencies, sync_dependencies) = Self::build_all_dependencies(&dto)?;

        // Phase 3: Populate the adjacency lists (incoming/outgoing) on each node
        Self::populate_node_adjacency_lists(&mut nodes, &data_dependencies, &sync_dependencies);

        // Phase 4: Build SyncGroups (co-allocation groups) using a Disjoint Set Union
        let (mut co_allocations, node_to_co_allocation) =
            Self::build_co_allocations(&nodes, &sync_dependencies)?;

        // Phase 5: Build the "overlay graph" of dependencies *between* SyncGroups
        let co_allocation_dependencies = Self::build_co_allocation_dependencies(
            &data_dependencies,
            &node_to_co_allocation,
            &mut co_allocations,
        )?;

        // Phase 6: Find the entry/exit points for both graphs
        let (entry_nodes, exit_nodes, entry_co_allocation, exit_co_allocation) =
            Self::find_entry_exit_points(&nodes, &co_allocations);

        // Final-Step: Update all nodes with their final CoAllocation key
        for (node_id, group_id) in node_to_co_allocation {
            if let Some(node) = nodes.get_mut(&node_id) {
                node.co_allocation_key = group_id;
            }
        }

        Ok(Workflow {
            base,
            nodes,
            data_dependencies,
            sync_dependencies,
            co_allocations,
            co_allocation_dependencies,
            entry_nodes,
            exit_nodes,
            entry_co_allocation,
            exit_co_allocation,
        })
    }
}

// Helper functions for the **TryFrom** implementation
impl Workflow {
    /// **Phase 0: Build Base Workflow**
    ///
    /// Creates the root `ReservationBase` for the `Workflow` itself from the DTO.
    pub fn build_base_workflow(dto: &WorkflowDto) -> ReservationBase {
        ReservationBase {
            id: dto.id.clone(),
            state: ReservationState::Open, // Workflow state is managed separately
            request_proceeding: ReservationProceeding::Commit, // Default
            arrival_time: dto.arrival_time,
            booking_interval_start: dto.booking_interval_start,
            booking_interval_end: dto.booking_interval_end,
            assigned_start: 0,
            assigned_end: 0,
            task_duration: 0,     // Duration is an aggregate of nodes
            reserved_capacity: 0, // Capacity is an aggregate of nodes
            is_moldable: false,
            moldable_work: 0,
        }
    }

    /// **Phase 1: Generate Workflow Nodes**
    pub fn generate_workflow_nodes(dto: &WorkflowDto) -> HashMap<String, WorkflowNode> {
        let mut nodes = HashMap::new();

        for task_dto in &dto.tasks {
            let node_res_dto = &task_dto.node_reservation;
            let node_id = task_dto.id.clone();

            // A dto task is a NodeReservation.
            let node_base = ReservationBase {
                id: node_id.clone(),
                state: map_reservation_state(task_dto.reservation_state),
                request_proceeding: map_reservation_proceeding(task_dto.request_proceeding),
                arrival_time: dto.arrival_time,
                booking_interval_start: dto.booking_interval_start,
                booking_interval_end: dto.booking_interval_end,
                assigned_start: 0, // Not scheduled yet
                assigned_end: 0,   // Not scheduled yet
                task_duration: node_res_dto.duration,
                reserved_capacity: node_res_dto.cpus,
                is_moldable: node_res_dto.is_moldable,
                moldable_work: node_res_dto.duration * node_res_dto.cpus,
            };

            let node_reservation = NodeReservation {
                base: node_base,
                task_path: node_res_dto.task_path.clone(),
                output_path: node_res_dto.output_path.clone(),
                error_path: node_res_dto.error_path.clone(),
            };

            // Create the WorkflowNode, data and sync links are added later
            let workflow_node = WorkflowNode {
                reservation: node_reservation,
                incoming_data: Vec::new(),
                outgoing_data: Vec::new(),
                incoming_sync: Vec::new(),
                outgoing_sync: Vec::new(),
                co_allocation_key: String::new(), // See Phase 4
            };

            nodes.insert(node_id, workflow_node);
        }
        nodes
    }

    /// **Phase 2: Build All Dependencies**
    ///
    /// Parses the DTO to create all `DataDependency` and `SyncDependency` objects.
    /// 1. Create "dangling" dependencies from `data_out`.
    /// 2. Connect them using `data_in`.
    /// 3. Create implicit dependencies from `dependencies: { data, sync }`.
    ///
    /// Returns the complete maps of data and sync dependencies.
    pub fn build_all_dependencies(
        dto: &WorkflowDto,
    ) -> Result<
        (
            HashMap<String, DataDependency>,
            HashMap<String, SyncDependency>,
        ),
        Error,
    > {
        let mut data_dependencies = HashMap::new();
        let mut sync_dependencies = HashMap::new();
        let mut dangling_deps: HashMap<String, DanglingDependency> = HashMap::new();
        let workflow_id = &dto.id;

        // Phase 2.1: Process DataOut
        for task_dto in &dto.tasks {
            let source_node_id = &task_dto.id;
            let node_res_dto = &task_dto.node_reservation;

            for data_out in &node_res_dto.data_out {
                let port_name = &data_out.name;
                let dangling_key = format!("{}/{}", source_node_id, port_name);

                let dep_id = format!("{}.{}.{}", workflow_id, source_node_id, port_name);

                let mut dep_base = ReservationBase {
                    id: dep_id,
                    state: ReservationState::Open,
                    request_proceeding: ReservationProceeding::Commit, // Default
                    arrival_time: dto.arrival_time,
                    booking_interval_start: 0, // TODO Will be set by scheduler
                    booking_interval_end: 0,   // TODO Will be set by scheduler
                    assigned_start: 0,
                    assigned_end: 0,
                    task_duration: 1, // Default for links
                    reserved_capacity: 0,
                    is_moldable: false,
                    moldable_work: 0,
                };

                // DataDependency (file transfer)
                if let Some(size) = data_out.size {
                    dep_base.is_moldable = true;
                    dep_base.reserved_capacity = size;
                    dep_base.moldable_work = size * dep_base.task_duration;

                    let data_dep = DataDependency {
                        reservation: LinkReservation {
                            base: dep_base,
                            start_point: String::new(), // TODO Set by scheduler
                            end_point: String::new(),   // TODO Set by scheduler
                        },
                        source_node: source_node_id.clone(),
                        target_node: String::new(),
                        port_name: port_name.clone(),
                        size,
                    };
                    dangling_deps.insert(dangling_key, DanglingDependency::Data(data_dep));
                }
                // SyncDependency
                else if let Some(bandwidth) = data_out.bandwidth {
                    dep_base.is_moldable = false;
                    dep_base.reserved_capacity = bandwidth;
                    dep_base.moldable_work = bandwidth * dep_base.task_duration;

                    let sync_dep = SyncDependency {
                        reservation: LinkReservation {
                            base: dep_base,
                            start_point: String::new(), // TODO Set by scheduler
                            end_point: String::new(),   // TODO Set by scheduler
                        },
                        source_node: source_node_id.clone(),
                        target_node: String::new(),
                        port_name: port_name.clone(),
                        bandwidth,
                    };
                    dangling_deps.insert(dangling_key, DanglingDependency::Sync(sync_dep));
                }
            }
        }

        // Phase 2.2: Process DataIn
        for task_dto in &dto.tasks {
            let target_node_id = &task_dto.id;
            let node_res_dto = &task_dto.node_reservation;

            for data_in in &node_res_dto.data_in {
                let dangling_key =
                    format!("{}/{}", data_in.source_reservation, data_in.source_port);

                if let Some(dangling_dep) = dangling_deps.remove(&dangling_key) {
                    match dangling_dep {
                        DanglingDependency::Data(mut data_dep) => {
                            data_dep.target_node = target_node_id.clone();
                            let dep_id = data_dep.reservation.base.id.clone();
                            data_dependencies.insert(dep_id, data_dep);
                        }
                        DanglingDependency::Sync(mut sync_dep) => {
                            sync_dep.target_node = target_node_id.clone();
                            let dep_id = sync_dep.reservation.base.id.clone();
                            sync_dependencies.insert(dep_id, sync_dep);
                        }
                    }
                } else {
                    // Dependency source not found!"
                    log::warn!("Could not find source for DataIn: {}", dangling_key);
                }
            }
        }

        // Phase 2.3: Process Dependencies (data/sync)
        for task_dto in &dto.tasks {
            let target_node_id = &task_dto.id;
            let dep_dto = &task_dto.node_reservation.dependencies;

            // "data" are DataDependencies with size 0
            Self::create_implicit_dependencies(
                workflow_id,
                &dep_dto.data,
                target_node_id,
                task_dto,
                dto.arrival_time,
                dto.booking_interval_start,
                dto.booking_interval_end,
                &mut data_dependencies,
                &mut sync_dependencies,
                "data",
            );

            // "sync" are SyncDependencies with bandwidth 0
            Self::create_implicit_dependencies(
                workflow_id,
                &dep_dto.sync,
                target_node_id,
                task_dto,
                dto.arrival_time,
                dto.booking_interval_start,
                dto.booking_interval_end,
                &mut data_dependencies,
                &mut sync_dependencies,
                "sync",
            );
        }

        Ok((data_dependencies, sync_dependencies))
    }

    /// **Phase 2.3 Helper:** Creates implicit "data" (Data) and "sync" (Sync) dependencies.
    #[allow(clippy::too_many_arguments)]
    pub fn create_implicit_dependencies(
        workflow_id: &str,
        source_ids: &[String],
        target_node_id: &str,
        task_dto: &TaskDto,
        arrival_time: i64,
        booking_start: i64,
        booking_end: i64,
        data_deps: &mut HashMap<String, DataDependency>,
        sync_deps: &mut HashMap<String, SyncDependency>,
        dep_type: &str,
    ) {
        for source_id in source_ids {
            let dep_id = format!(
                "{}.{}.{}.{}",
                workflow_id, dep_type, source_id, target_node_id
            );
            let dep_base = ReservationBase {
                id: dep_id.clone(),
                state: ReservationState::Open,
                request_proceeding: map_reservation_proceeding(task_dto.request_proceeding),
                arrival_time,
                booking_interval_start: booking_start,
                booking_interval_end: booking_end,
                assigned_start: 0,
                assigned_end: 0,
                task_duration: 0,
                reserved_capacity: 0,
                is_moldable: false,
                moldable_work: 0,
            };
            let link_res = LinkReservation {
                base: dep_base,
                start_point: String::new(),
                end_point: String::new(),
            };

            if dep_type == "data" {
                let data_dep = DataDependency {
                    reservation: link_res,
                    source_node: source_id.clone(),
                    target_node: target_node_id.to_string(),
                    port_name: "data".to_string(),
                    size: 0,
                };
                data_deps.insert(dep_id, data_dep);
            } else if dep_type == "sync" {
                let sync_dep = SyncDependency {
                    reservation: link_res,
                    source_node: source_id.clone(),
                    target_node: target_node_id.to_string(),
                    port_name: "sync".to_string(),
                    bandwidth: 0,
                };
                sync_deps.insert(dep_id, sync_dep);
            }
        }
    }

    /// **Phase 3: Populate Node Adjacency Lists**
    ///
    /// Connects the `WorkflowNode`s by populating their `incoming_` and `outgoing_`
    /// `Vec`s with the dependency IDs.
    pub fn populate_node_adjacency_lists(
        nodes: &mut HashMap<String, WorkflowNode>,
        data_dependencies: &HashMap<String, DataDependency>,
        sync_dependencies: &HashMap<String, SyncDependency>,
    ) {
        for (dep_id, data_dep) in data_dependencies {
            if let Some(source_node) = nodes.get_mut(&data_dep.source_node) {
                source_node.outgoing_data.push(dep_id.clone());
            } else {
                log::warn!(
                    "DataDep source node '{}' not found for dep '{}'",
                    data_dep.source_node,
                    dep_id
                );
            }
            if let Some(target_node) = nodes.get_mut(&data_dep.target_node) {
                target_node.incoming_data.push(dep_id.clone());
            } else {
                log::warn!(
                    "DataDep target node '{}' not found for dep '{}'",
                    data_dep.target_node,
                    dep_id
                );
            }
        }

        for (dep_id, sync_dep) in sync_dependencies {
            if let Some(source_node) = nodes.get_mut(&sync_dep.source_node) {
                source_node.outgoing_sync.push(dep_id.clone());
            } else {
                log::warn!(
                    "SyncDep source node '{}' not found for dep '{}'",
                    sync_dep.source_node,
                    dep_id
                );
            }
            if let Some(target_node) = nodes.get_mut(&sync_dep.target_node) {
                target_node.incoming_sync.push(dep_id.clone());
            } else {
                log::warn!(
                    "SyncDep target node '{}' not found for dep '{}'",
                    sync_dep.target_node,
                    dep_id
                );
            }
        }
    }

    /// **Phase 4: Build CoAllocation Graph**
    ///
    /// Identifys co-allocation groups. It uses a Disjoint Set Union (DSU) structure
    /// to merge nodes that are connected by `SyncDependency`.
    pub fn build_co_allocations(
        nodes: &HashMap<String, WorkflowNode>,
        sync_dependencies: &HashMap<String, SyncDependency>,
    ) -> Result<(HashMap<String, CoAllocation>, HashMap<String, String>), Error> {
        let mut preliminary_co_allocation: HashMap<String, CoAllocation> = HashMap::new();
        let mut node_to_co_allocation: HashMap<String, String> = HashMap::new();

        // 1. Create mappings between String IDs and usize indices for the DSU crate
        let node_ids: Vec<String> = nodes.keys().cloned().collect();
        let mut node_id_to_index: HashMap<String, usize> = HashMap::with_capacity(node_ids.len());
        for (index, id) in node_ids.iter().enumerate() {
            node_id_to_index.insert(id.clone(), index);
        }

        // 2. Initialize the DSU structure
        let mut dsu = QuickUnionUf::<UnionBySize>::new(node_ids.len());

        for (node_id, node) in nodes {
            let co_allocation = CoAllocation {
                id: node_id.clone(),
                representative: Some(node.clone()), // This node is its own rep for now
                members: vec![node_id.clone()],
                sync_dependencies: Vec::new(), // Will be populated below
                outgoing_co_allocation_dependencies: Vec::new(),
                outgoing_data_dependencies: Vec::new(),
                incoming_co_allocation_dependencies: Vec::new(),
                incoming_data_dependencies: Vec::new(),
                rank_upward: 0,
                rank_downward: 0,
                number_of_nodes_critical_path_downwards: 0,
                number_of_nodes_critical_path_upwards: 0,
                is_in_queue: false,
                unprocessed_predecessor_count: 0,
                unprocessed_successor_count: 0,
                spare_time: 0,
                max_succ_force: 0.0,
                max_pred_force: 0.0,
                is_discovered: false,
                is_processed: false,
                is_moveable: true,
                is_moveable_interval_start: true,
                is_moveable_interval_end: true,
                start_position: 0.0,
                end_position: 0.0,
            };
            preliminary_co_allocation.insert(node_id.clone(), co_allocation);
        }

        // 3. Perform the `union` operation for every SyncDependency
        for sync_dep in sync_dependencies.values() {
            if let (Some(&source_index), Some(&target_index)) = (
                node_id_to_index.get(&sync_dep.source_node),
                node_id_to_index.get(&sync_dep.target_node),
            ) {
                dsu.union(source_index, target_index);
            } else {
                log::warn!(
                    "Could not find node index for SyncDependency: {} -> {}",
                    sync_dep.source_node,
                    sync_dep.target_node
                );
            }
        }

        // 4. Create the final, merged CoAllocations
        let mut co_allocation: HashMap<String, CoAllocation> = HashMap::new();
        for node_id in &node_ids {
            let node_index = node_id_to_index
                .get(node_id)
                .expect("Node must have an index");
            let rep_index = dsu.find(*node_index);
            let rep_id = node_ids
                .get(rep_index)
                .expect("Representative index must be valid")
                .clone();

            node_to_co_allocation.insert(node_id.clone(), rep_id.clone());

            // If this node is not its own representative,
            // merge its preliminary group into the representative's group.
            if node_id != &rep_id {
                if let Some(prelim_group) = preliminary_co_allocation.remove(node_id) {
                    let final_group = co_allocation.entry(rep_id.clone()).or_insert_with(|| {
                        preliminary_co_allocation
                            .remove(&rep_id)
                            .expect("Representative's preliminary group must exist")
                    });
                    final_group.members.extend(prelim_group.members);
                }
            }
            // If this node *is* a representative, move its prelim group into the map
            else if !co_allocation.contains_key(&rep_id) {
                let base_group = preliminary_co_allocation
                    .remove(&rep_id)
                    .expect("Preliminary group must exist");
                co_allocation.insert(rep_id, base_group);
            }
        }

        // 5. Populate the `sync_dependencies` Vec within each CoAllocation
        for (dep_id, sync_dep) in sync_dependencies {
            let rep_id = node_to_co_allocation
                .get(&sync_dep.source_node)
                .expect("Node must be in a CoAllocation");

            if let Some(group) = co_allocation.get_mut(rep_id) {
                group.sync_dependencies.push(sync_dep.clone());
            } else {
                log::warn!(
                    "CoAllocation {} not found for SyncDependency {}",
                    rep_id,
                    dep_id
                );
            }
        }

        Ok((co_allocation, node_to_co_allocation))
    }

    /// **Phase 5: Build CoAllocation Dependencies (Coallocation Graph)**
    ///
    /// Creates the "CoAllocation graph" by building `CoAllocationDependency` edges
    /// *between* the `CoAllocation`s.
    ///
    /// This iterates over all `DataDependency`s and, if a dependency links
    /// nodes in two *different* `CoAllocation`s, its creates an edge between those groups.
    pub fn build_co_allocation_dependencies(
        data_dependencies: &HashMap<String, DataDependency>,
        node_to_co_allocation: &HashMap<String, String>,
        co_allocation: &mut HashMap<String, CoAllocation>,
    ) -> Result<HashMap<String, CoAllocationDependency>, Error> {
        let mut co_allocation_dependencies = HashMap::new();

        for (dep_id, data_dep) in data_dependencies {
            if let (Some(source_co_allocation_id), Some(target_co_allocation_id)) = (
                node_to_co_allocation.get(&data_dep.source_node),
                node_to_co_allocation.get(&data_dep.target_node),
            ) {
                // Only create sync group edges between *different* CoAllocations
                if source_co_allocation_id != target_co_allocation_id {
                    let co_allocation_dep = CoAllocationDependency {
                        id: dep_id.clone(),
                        source_group: source_co_allocation_id.clone(),
                        target_group: target_co_allocation_id.clone(),
                        data_dependency: dep_id.clone(),
                    };
                    let co_allocation_dep_id = co_allocation_dep.id.clone();
                    co_allocation_dependencies
                        .insert(co_allocation_dep_id, co_allocation_dep.clone());

                    // Correctly populate the adjacency lists in the CoAllocation
                    if let Some(source_co_allocation) =
                        co_allocation.get_mut(source_co_allocation_id)
                    {
                        source_co_allocation
                            .outgoing_co_allocation_dependencies
                            .push(co_allocation_dep.clone());
                        source_co_allocation
                            .outgoing_data_dependencies
                            .push(data_dep.clone());
                    }
                    if let Some(target_co_allocation) =
                        co_allocation.get_mut(target_co_allocation_id)
                    {
                        target_co_allocation
                            .incoming_co_allocation_dependencies
                            .push(co_allocation_dep);
                        target_co_allocation
                            .incoming_data_dependencies
                            .push(data_dep.clone());
                    }
                }
            } else {
                log::warn!(
                    "Skipping co_allocation dependency '{}' because source ('{}') or target ('{}') node was not found in co_allocation map.",
                    dep_id,
                    data_dep.source_node,
                    data_dep.target_node
                );
            }
        }
        Ok(co_allocation_dependencies)
    }

    /// **Phase 6: Find Entry and Exit Points**
    ///
    /// Finds all entry/exit nodes for the base graph and all entry/exit
    /// groups for the CoAllocation graph. This is used by the scheduler to find
    /// the starting points for traversal.
    pub fn find_entry_exit_points(
        nodes: &HashMap<String, WorkflowNode>,
        co_allocation: &HashMap<String, CoAllocation>,
    ) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
        let entry_nodes = nodes
            .values()
            .filter(|n| n.incoming_data.is_empty() && n.incoming_sync.is_empty())
            .map(|n| n.reservation.base.id.clone())
            .collect();

        let exit_nodes = nodes
            .values()
            .filter(|n| n.outgoing_data.is_empty() && n.outgoing_sync.is_empty())
            .map(|n| n.reservation.base.id.clone())
            .collect();

        // Find Entry/Exit SyncGroups based on the *overlay* graph
        let entry_co_allocation = co_allocation
            .values()
            .filter(|on| on.incoming_co_allocation_dependencies.is_empty())
            .map(|on| on.id.clone())
            .collect();

        let exit_co_allocation = co_allocation
            .values()
            .filter(|on| on.outgoing_co_allocation_dependencies.is_empty())
            .map(|on| on.id.clone())
            .collect();

        (
            entry_nodes,
            exit_nodes,
            entry_co_allocation,
            exit_co_allocation,
        )
    }
}

// Helper
pub fn map_reservation_state(dto_state: ReservationStateDto) -> ReservationState {
    match dto_state {
        ReservationStateDto::Rejected => ReservationState::Rejected,
        ReservationStateDto::Deleted => ReservationState::Deleted,
        ReservationStateDto::Open => ReservationState::Open,
        ReservationStateDto::ProbeAnswer => ReservationState::ProbeAnswer,
        ReservationStateDto::ReserveAnswer => ReservationState::ReserveAnswer,
        ReservationStateDto::Committed => ReservationState::Committed,
        ReservationStateDto::Finished => ReservationState::Finished,
    }
}

// Helper
pub fn map_reservation_proceeding(dto_proc: ReservationProceedingDto) -> ReservationProceeding {
    match dto_proc {
        ReservationProceedingDto::Probe => ReservationProceeding::Probe,
        ReservationProceedingDto::Reserve => ReservationProceeding::Reserve,
        ReservationProceedingDto::Commit => ReservationProceeding::Commit,
        ReservationProceedingDto::Delete => ReservationProceeding::Delete,
    }
}

impl Workflow {
    /// Computes the upward rank for all `CoAllocation`s in the Workflow.
    ///
    /// The upward rank (`rank_upward`) is the length of the longest path through the workflow.
    ///
    /// This function also computes the number of nodes in the critical upward path
    /// (`number_of_nodes_critical_path_upwards`) for all nodes.
    ///
    /// A `Vec<Option<WorkflowNode>>` containing the `representative` node for
    /// every `CoAllocation` in the workflow, ordered by `rank_upward` in descending
    /// order (largest ranks are first).
    fn calculate_upward_rank(mut self, avg_net_speed: i64) -> Vec<Option<WorkflowNode>> {
        let mut finished_node_keys: Vec<String> = Vec::with_capacity(self.co_allocations.len());
        let mut queue: Vec<String> = Vec::new();

        // 1. Clear marks on all Nodes
        for co_allocation in self.co_allocations.values_mut() {
            co_allocation.is_discovered = false;
            co_allocation.is_processed = false;
        }

        // 2. Initiate queue with the entry nodes
        for group_key in &self.entry_co_allocation {
            if let Some(entry_group) = self.co_allocations.get_mut(group_key) {
                entry_group.is_discovered = true;
                queue.push(group_key.clone());
            }
        }

        // 3. Compute the ranks of every node
        while let Some(next_key) = queue.last() {
            let next_key = next_key.clone();
            let node = self.co_allocations.get(&next_key).unwrap_or_else(|| {
                panic!("CoAllocation key '{}' in queue but not in map.", next_key)
            });
            if node.is_processed {
                queue.pop();
                continue;
            }

            let node_duration = node.get_co_allocation_duration(&self.nodes);
            let outgoing_deps = node.outgoing_co_allocation_dependencies.clone();
            let mut rank = node_duration;
            let mut number_of_nodes_critical_path = 1;
            let mut is_successor_without_rank = false;

            for outgoing_dep in &outgoing_deps {
                let target_key = &outgoing_dep.target_group;

                if let Some(target_group) = self.co_allocations.get(target_key) {
                    if !target_group.is_processed {
                        is_successor_without_rank = true;
                        queue.push(target_key.clone());
                    } else {
                        let size = self
                            .data_dependencies
                            .get(&outgoing_dep.data_dependency)
                            .expect("Data dependency not found")
                            .size;

                        let communication_time = if avg_net_speed > 0 {
                            size / avg_net_speed
                        } else {
                            log::warn!("avg_net_speed is 0, setting communication_time to 0");
                            0
                        };

                        let successor_rank = target_group.rank_upward;
                        let new_possible_rank = node_duration + communication_time + successor_rank;

                        if rank < new_possible_rank {
                            rank = new_possible_rank;
                            number_of_nodes_critical_path =
                                target_group.number_of_nodes_critical_path_upwards + 1;
                        }
                    }
                } else {
                    log::warn!("Target CoAllocation '{}' not found.", target_key);
                }
            }

            // 5. Calculate rank for this node if all successors are processed
            if !is_successor_without_rank {
                let processed_node = self
                    .co_allocations
                    .get_mut(&next_key)
                    .expect("Node must exist");

                processed_node.rank_upward = rank;
                processed_node.number_of_nodes_critical_path_upwards =
                    number_of_nodes_critical_path;
                processed_node.is_processed = true;

                queue.pop();
                finished_node_keys.push(next_key);
            }
        }

        // 6. Build sortedList
        finished_node_keys.sort_by(|a_key, b_key| {
            let a_rank = self.co_allocations.get(a_key).unwrap().rank_upward;
            let b_rank = self.co_allocations.get(b_key).unwrap().rank_upward;
            b_rank.cmp(&a_rank)
        });

        // 7. Map keys to the representative nodes
        return finished_node_keys
            .into_iter()
            .map(|key| {
                self.co_allocations
                    .get(&key)
                    .unwrap()
                    .representative
                    .clone()
            })
            .collect();
    }

    /// Computes the downward rank for all `CoAllocation`s in the Workflow.
    ///
    /// The downward rank (`rank_downward`) is the length of the longest path through the workflow (starting at an entry node).
    ///
    /// This function also computes the number of nodes in the critical downward path
    /// (`number_of_nodes_critical_path_downwards`) for all nodes.
    ///
    /// A `Vec<Option<WorkflowNode>>` containing the `representative` node for
    /// every `CoAllocation` in the workflow, ordered by `rank_downward` in descending
    /// order (largest ranks are first).
    fn calculate_downward_rank(mut self, avg_net_speed: i64) -> Vec<Option<WorkflowNode>> {
        let mut finished_node_keys: Vec<String> = Vec::with_capacity(self.co_allocations.len());
        let mut queue: Vec<String> = Vec::new();

        for co_allocation in self.co_allocations.values_mut() {
            co_allocation.is_discovered = false;
            co_allocation.is_processed = false;
        }

        for group_key in &self.exit_co_allocation {
            if let Some(exit_group) = self.co_allocations.get_mut(group_key) {
                exit_group.is_discovered = true;
                queue.push(group_key.clone());
            }
        }

        while let Some(next_key) = queue.last() {
            // Like Java's `queue.getFirst()` (peek)
            let next_key = next_key.clone(); // Avoid borrow issue

            // Get the node to check if it's processed
            let node = self.co_allocations.get(&next_key).unwrap_or_else(|| {
                panic!("CoAllocation key '{}' in queue but not in map.", next_key)
            });

            if node.is_processed {
                queue.pop();
                continue;
            }

            let node_duration = node.get_co_allocation_duration(&self.nodes);
            let incoming_deps = node.incoming_co_allocation_dependencies.clone();

            let mut rank = node_duration;
            let mut number_of_nodes_critical_path = 1;
            let mut is_predecessor_without_rank = false;

            for incoming_dep in &incoming_deps {
                let source_key = &incoming_dep.source_group;

                if let Some(source_group) = self.co_allocations.get(source_key) {
                    if !source_group.is_processed {
                        is_predecessor_without_rank = true;

                        queue.push(source_key.clone());
                    } else {
                        let size = self
                            .data_dependencies
                            .get(&incoming_dep.data_dependency)
                            .expect("Data dependency not found")
                            .size;

                        let communication_time = if avg_net_speed > 0 {
                            size / avg_net_speed
                        } else {
                            log::warn!("avg_net_speed is 0, setting communication_time to 0");
                            0
                        };

                        let predecessor_rank = source_group.rank_downward;
                        let new_possible_rank =
                            node_duration + communication_time + predecessor_rank;

                        if rank < new_possible_rank {
                            rank = new_possible_rank;
                            number_of_nodes_critical_path =
                                source_group.number_of_nodes_critical_path_downwards + 1;
                        }
                    }
                } else {
                    log::warn!("Source CoAllocation '{}' not found.", source_key);
                }
            }

            if !is_predecessor_without_rank {
                let processed_node = self
                    .co_allocations
                    .get_mut(&next_key)
                    .expect("Node must exist");

                processed_node.rank_downward = rank;
                processed_node.number_of_nodes_critical_path_downwards =
                    number_of_nodes_critical_path;
                processed_node.is_processed = true;

                queue.pop();
                finished_node_keys.push(next_key);
            }
        }

        finished_node_keys.sort_by(|a_key, b_key| {
            let a_rank = self.co_allocations.get(a_key).unwrap().rank_downward;
            let b_rank = self.co_allocations.get(b_key).unwrap().rank_downward;
            b_rank.cmp(&a_rank)
        });

        return finished_node_keys
            .into_iter()
            .map(|key| {
                self.co_allocations
                    .get(&key)
                    .unwrap()
                    .representative
                    .clone()
            })
            .collect();
    }
}
