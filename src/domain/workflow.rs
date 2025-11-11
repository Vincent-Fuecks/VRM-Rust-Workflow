use std::collections::HashMap;

use crate::api::workflow_dto::{WorkflowDto};
use crate::api::reservation_dto::{ReservationStateDto, ReservationProceedingDto};
use crate::domain::workflow_node::{WorkflowNode};
use crate::domain::reservation::{ReservationProceeding, ReservationState, ReservationBase, NodeReservation, LinkReservation};
use crate::domain::dependency::{DataDependency, SyncDependency, SyncGroupDependency};
use crate::domain::sync_group::{SyncGroup};
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Workflow {
    pub base: ReservationBase,

    // The graph components, stored in HashMaps
    pub nodes: HashMap<String, WorkflowNode>,
    pub data_dependencies: HashMap<String, DataDependency>,
    pub sync_dependencies: HashMap<String, SyncDependency>,

    // The sync groups, which are utilized for scheduling sync groups.
    pub sync_groups: HashMap<String, SyncGroup>,
    pub sync_group_dependencies: HashMap<String, SyncGroupDependency>,

    /// Keys to Workflow.nodes
    pub entry_nodes: Vec<String>,

    /// Keys to Workflow.nodes
    pub exit_nodes: Vec<String>,
    
    /// Keys to Workflow.sync_groups
    /// TODO could also be Workflow.nodes
    pub sync_group_entry_nodes: Vec<String>,
    
    /// Keys to Workflow.sync_groups
    /// TODO could also be Workflow.nodes
    pub sync_group_exit_nodes: Vec<String>,
}

// A temporary struct to hold dependencies that have a source but no target yet.
#[derive(Debug, Clone)]
enum DanglingDependency {
    Data(DataDependency),
    Sync(SyncDependency),
}

// Helper to map DTO ReservationState to internal ReservationState
fn map_reservation_state(dto_state: ReservationStateDto) -> ReservationState {
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

// Helper to map DTO ReservationProceeding to internal ReservationProceeding
fn map_reservation_proceeding(dto_proc: ReservationProceedingDto) -> ReservationProceeding {
    match dto_proc {
        ReservationProceedingDto::Probe => ReservationProceeding::Probe,
        ReservationProceedingDto::Reserve => ReservationProceeding::Reserve,
        ReservationProceedingDto::Commit => ReservationProceeding::Commit,
        ReservationProceedingDto::Delete => ReservationProceeding::Delete,
    }
}

// Contains only help functions for the impl TryFrom<WorkflowDto> for Workflow
impl Workflow {
    /// Generates all WorklowNodes, from the parsed json
    fn generate_workflow_nodes(dto: &WorkflowDto) -> HashMap<String, WorkflowNode> {
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
                assigned_start: 0,
                assigned_end: 0,
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
                
                // Will be set during sync group build
                sync_group_key: String::new(), 
            };

            nodes.insert(node_id, workflow_node);
        }

        return nodes;
    }
}
/// Constructs a complete Workflow graph from a WorkflowDto.
impl TryFrom<WorkflowDto> for Workflow {
    type Error = Error;

    fn try_from(dto: WorkflowDto) -> Result<Self, Self::Error> {


        let workflow_id = dto.id.clone();
        let base = ReservationBase {
            id: workflow_id.clone(),
            state: ReservationState::Open, // Workflow state is managed separately
            request_proceeding: ReservationProceeding::Commit, // Default
            arrival_time: dto.arrival_time,
            booking_interval_start: dto.booking_interval_start,
            booking_interval_end: dto.booking_interval_end,
            assigned_start: 0, 
            assigned_end: 0,  
            task_duration: 0,   
            reserved_capacity: 0, 
            is_moldable: false,
            moldable_work: 0,
        };

        let mut data_dependencies = HashMap::new();
        let mut sync_dependencies = HashMap::new();

        // Phase 1: Create all WorkflowNodes
        let mut nodes = Workflow::generate_workflow_nodes(&dto);

        // Phase 2: Build Edges aka Dependencies
        let mut dangling_deps: HashMap<String, DanglingDependency> = HashMap::new();

        for task_dto in &dto.tasks {
            let source_node_id = &task_dto.id;
            let node_res_dto = &task_dto.node_reservation;

            // Phase 2.1: Process DataOut (create dangling dependencies)
            for data_out in &node_res_dto.data_out {
                let port_name = &data_out.name;
                let dangling_key = format!("{}/{}", source_node_id, port_name);
                
                let dep_id = format!(
                    "{}.{}.{}", // TODO should be done differently maybe with a struct as key?
                    workflow_id,
                    source_node_id,
                    port_name
                );

                let mut dep_base = ReservationBase {
                    id: dep_id.clone(),
                    state: ReservationState::Open,
                    request_proceeding: map_reservation_proceeding(task_dto.request_proceeding),
                    arrival_time: dto.arrival_time,
                    booking_interval_start: dto.booking_interval_start,
                    booking_interval_end: dto.booking_interval_end,
                    assigned_start: 0,
                    assigned_end: 0,
                    task_duration: 1, // Default for links
                    reserved_capacity: 0, 
                    is_moldable: false, 
                    moldable_work: 0, 
                };

                // This is a DataDependency (file transfer)
                if let Some(size) = data_out.size {
                    dep_base.is_moldable = true;
                    dep_base.reserved_capacity = size;
                    dep_base.moldable_work = size;
                    
                    let link_res = LinkReservation { base: dep_base, start_point: String::new(), end_point: String::new() };

                    let data_dep = DataDependency {
                        reservation: link_res,
                        source_node: source_node_id.clone(),
                        target_node: String::new(), // Unknown!
                        port_name: port_name.clone(),
                        size: size,
                    };
                    dangling_deps.insert(dangling_key, DanglingDependency::Data(data_dep));
                
                // This is a SyncDependency (bandwidth)
                } else if let Some(bandwidth) = data_out.bandwidth {
                    dep_base.is_moldable = false;
                    dep_base.reserved_capacity = bandwidth;
                    dep_base.moldable_work = 0; // Not moldable

                    let link_res = LinkReservation { base: dep_base, start_point: String::new(), end_point: String::new() };
                    
                    let sync_dep = SyncDependency {
                        reservation: link_res,
                        source_node: source_node_id.clone(),
                        target_node: String::new(), // Unknown!
                        port_name: port_name.clone(),
                        bandwidth: bandwidth,
                    };

                    dangling_deps.insert(dangling_key, DanglingDependency::Sync(sync_dep));
                }
            }
        }
        
        // Phase 2.2: Process DataIn (complete dangling dependencies)
        for task_dto in &dto.tasks {
            let target_node_id = &task_dto.id;
            let node_res_dto = &task_dto.node_reservation;

            for data_in in &node_res_dto.data_in {
                let dangling_key = format!("{}/{}", data_in.source_reservation, data_in.source_port);
                
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
                    // Dependency source not found
                    log::warn!("Could not find source for DataIn: {}", dangling_key);
                }
            }
        }

        // Phase 2.3: Process pure Dependencies (pre/sync)
        for task_dto in &dto.tasks {
            let target_node_id = &task_dto.id;
            let dep_dto = &task_dto.node_reservation.dependencies;

            // "pre" are DataDependencies with size 0
            for pre_source_id in &dep_dto.pre {
                let dep_id = format!("{}.pre.{}.{}", workflow_id, pre_source_id, target_node_id);
                let dep_base = ReservationBase {
                    id: dep_id.clone(),
                    state: ReservationState::Open,
                    request_proceeding: map_reservation_proceeding(task_dto.request_proceeding),
                    arrival_time: dto.arrival_time,
                    booking_interval_start: dto.booking_interval_start,
                    booking_interval_end: dto.booking_interval_end,
                    assigned_start: 0, 
                    assigned_end: 0,
                    task_duration: 0, 
                    reserved_capacity: 0,
                    is_moldable: true, 
                    moldable_work: 0,
                };

                let data_dep = DataDependency {
                    reservation: LinkReservation { base: dep_base, start_point: String::new(), end_point: String::new() },
                    source_node: pre_source_id.clone(),
                    target_node: target_node_id.clone(),
                    port_name: "pre".to_string(),
                    size: 0,
                };
                data_dependencies.insert(dep_id, data_dep);
            }

            // "sync" are SyncDependencies
            for sync_source_id in &dep_dto.sync {
                 let dep_id = format!("{}.sync.{}.{}", workflow_id, sync_source_id, target_node_id);
                 let dep_base = ReservationBase {
                    id: dep_id.clone(),
                    state: ReservationState::Open,
                    request_proceeding: map_reservation_proceeding(task_dto.request_proceeding),
                    arrival_time: dto.arrival_time,
                    booking_interval_start: dto.booking_interval_start,
                    booking_interval_end: dto.booking_interval_end,
                    assigned_start: 0, 
                    assigned_end: 0,
                    task_duration: 1, 
                    reserved_capacity: 0, // 0 bandwidth
                    is_moldable: false, 
                    moldable_work: 0,
                };

                let sync_dep = SyncDependency {
                    reservation: LinkReservation { base: dep_base, start_point: String::new(), end_point: String::new() },
                    source_node: sync_source_id.clone(),
                    target_node: target_node_id.clone(),
                    port_name: "sync".to_string(),
                    bandwidth: 0,
                };
                sync_dependencies.insert(dep_id, sync_dep);
            }
        }
        
        // Phase 3: Populate node adjacency lists
        for (dep_id, data_dep) in &data_dependencies {
            if let Some(source_node) = nodes.get_mut(&data_dep.source_node) {
                source_node.outgoing_data.push(dep_id.clone());
            } else {
                log::warn!("DataDep source node '{}' not found for dep '{}'", data_dep.source_node, dep_id);
            }
            if let Some(target_node) = nodes.get_mut(&data_dep.target_node) {
                target_node.incoming_data.push(dep_id.clone());
            } else {
                log::warn!("DataDep target node '{}' not found for dep '{}'", data_dep.target_node, dep_id);
            }
        }

        for (dep_id, sync_dep) in &sync_dependencies {
             if let Some(source_node) = nodes.get_mut(&sync_dep.source_node) {
                source_node.outgoing_sync.push(dep_id.clone());
            } else {
                log::warn!("SyncDep source node '{}' not found for dep '{}'", sync_dep.source_node, dep_id);
            }
            if let Some(target_node) = nodes.get_mut(&sync_dep.target_node) {
                target_node.incoming_sync.push(dep_id.clone());
            } else {
                log::warn!("SyncDep target node '{}' not found for dep '{}'", sync_dep.target_node, dep_id);
            }
        }

        // Phase 4: Building Sync Groups
        // TODO This is a simplified version using union-find logic.
        let mut sync_groups = HashMap::new();
        let mut node_to_sync_group: HashMap<String, String> = HashMap::new();

        // Each node starts in its own sync group
        for node_id in nodes.keys() {
            let sync_group_id = node_id.clone();

            sync_groups.insert(sync_group_id.clone(), SyncGroup {
                id: sync_group_id.clone(),

                representative: None,
                members: vec![node_id.clone()],
                
                sync_dependencies: Vec::new(),
                
                outgoing_sync_group_dependencies:  Vec::new(),
                outgoing_data_dependencies:  Vec::new(),

                incoming_sync_group_dependencies:  Vec::new(),
                incoming_data_dependencies:  Vec::new(),

                rank_upward: 0,
                rank_downward: 0,

                number_of_nodes_critical_path_downwards: 0,
                number_of_nodes_critical_path_upwards: 0,
                
                // Temporary calculation values (internal state)
                is_in_queue: false,
                unprocessed_predecessors: 0,
                spare_time: 0, 

                // FRAG-WINDOW Scheduling forces/properties
                max_succ_force: 0.0,
                max_pred_force: 0.0,
                
                // Search flags
                is_discovered: false,
                is_processed: false,

                is_moveable: true,
                is_moveable_interval_start: true, 
                is_moveable_interval_end: true, 
                start_position: 0.0,
                end_position: 0.0, 

            });

            node_to_sync_group.insert(node_id.clone(), sync_group_id);
        }

        // TODO: Implement logic to merge sync groups based on SyncDependencies.
        // This involves a graph traversal or a union-find data structure.
        // For now, we'll skip the merging part, so every node is in its own sync group.
        // The `sync` dependencies in the DTO imply which nodes to merge.
        
        for node in nodes.values_mut() {
            node.sync_group_key = node_to_sync_group.get(&node.reservation.base.id).unwrap().clone();
        }

        // Phase 5: Build SyncGroupDependenies
        let mut sync_group_dependencies = HashMap::new();
        for (dep_id, data_dep) in &data_dependencies {

            if let (Some(source_sync_group_id), Some(target_sync_group_id)) = (
                node_to_sync_group.get(&data_dep.source_node),
                node_to_sync_group.get(&data_dep.target_node),
            ) {
                // Only create sync group edges between different sync groups
                if source_sync_group_id != target_sync_group_id {
                    let sync_group_dep = SyncGroupDependency {
                        id: dep_id.clone(),
                        source_node: source_sync_group_id.clone(),
                        target_node: target_sync_group_id.clone(),
                        data_dependency: dep_id.clone(),
                    };
                    let sync_group_dep_id = sync_group_dep.id.clone();
                    sync_group_dependencies.insert(sync_group_dep_id.clone(), sync_group_dep);

                    // Add links to the sync_groupNodes
                    // TODO sould be incoming and outgoing dependencies etc. 
                    // TODO For simplicity currently with String member --> incoming/outgoing
                    if let Some(source_sync_group) = sync_groups.get_mut(source_sync_group_id) {
                        source_sync_group.members.push(sync_group_dep_id.clone());
                    }
                    if let Some(target_sync_group) = sync_groups.get_mut(target_sync_group_id) {
                        target_sync_group.members.push(sync_group_dep_id.clone());
                    }
                }
            } else {

                log::warn!(
                    "Skipping sync_group dependency '{}' because source ('{}') or target ('{}') node was not found in sync_group map.",
                    dep_id,
                    data_dep.source_node,
                    data_dep.target_node
                );
            }
        }
        
        // Phase 6: Find Entry/Exit Nodes
        let entry_nodes = nodes.values()
            .filter(|n| n.incoming_data.is_empty() && n.incoming_sync.is_empty())
            .map(|n| n.reservation.base.id.clone())
            .collect();
            
        let exit_nodes = nodes.values()
            .filter(|n| n.outgoing_data.is_empty() && n.outgoing_sync.is_empty())
            .map(|n| n.reservation.base.id.clone())
            .collect();

        let sync_group_entry_nodes = sync_groups.values()
            .filter(|on| on.incoming_data_dependencies.is_empty())
            .map(|on| on.id.clone())
            .collect();

        let sync_group_exit_nodes = sync_groups.values()
            .filter(|on| on.outgoing_data_dependencies.is_empty())
            .map(|on| on.id.clone())
            .collect();


        Ok(Workflow {
            base,
            nodes,
            data_dependencies,
            sync_dependencies,
            sync_groups,
            sync_group_dependencies,
            entry_nodes,
            exit_nodes,
            sync_group_entry_nodes,
            sync_group_exit_nodes,
        })
    }
}