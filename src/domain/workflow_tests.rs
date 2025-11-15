/// This file contains the unit tests for the `workflow.rs` module.
///
/// We test each helper function (Phases 0-6) in isolation to ensure
/// logic is correct and to achieve high code coverage. This complements
/// the integration-style test in `test_workflow_loading.rs`.
#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    // --- Core Imports ---
    use crate::{
        api::{
            dependency_dto::DependencyDto,
            reservation_dto::{
                DataInDto, DataOutDto, LinkReservationDto, NodeReservationDto,
                ReservationProceedingDto, ReservationStateDto,
            },
            workflow_dto::{TaskDto, WorkflowDto},
        },
        domain::{
            // [FIX] Import SyncGroup
            co_allocation::CoAllocation,
            dependency::{DataDependency, SyncDependency},
            reservation::{ReservationProceeding, ReservationState},
            // [FIX] Import the helper functions directly
            workflow::{Workflow, map_reservation_proceeding, map_reservation_state},
            workflow_node::WorkflowNode,
        },
    };

    // --- HELPER FUNCTIONS FOR TEST SETUP ---

    /// A helper to create a minimal, valid WorkflowDto for testing.
    fn mock_workflow_dto(tasks: Vec<TaskDto>) -> WorkflowDto {
        WorkflowDto {
            id: "test-wf-01".to_string(),
            arrival_time: 1000,
            booking_interval_start: 1001,
            booking_interval_end: 9001,
            tasks,
        }
    }

    /// A helper to create a minimal, valid TaskDto with a NodeReservation.
    fn mock_task_dto(
        id: &str,
        dependencies: DependencyDto,
        data_in: Vec<DataInDto>,
        data_out: Vec<DataOutDto>,
    ) -> TaskDto {
        TaskDto {
            id: id.to_string(),
            reservation_state: ReservationStateDto::Open,
            request_proceeding: ReservationProceedingDto::Commit,
            node_reservation: NodeReservationDto {
                task_path: Some(format!("/bin/{}", id)),
                output_path: None,
                error_path: None,
                duration: 100,
                is_moldable: false,
                cpus: 4,
                dependencies,
                data_in,
                data_out,
            },
            // [FIX] `link_reservation` is not optional. Provide a mock.
            link_reservation: LinkReservationDto {
                start_point: "".to_string(),
                end_point: "".to_string(),
                amount: None,
                bandwidth: None,
            },
        }
    }

    /// A helper to create a barebones TaskDto.
    fn bare_task_dto(id: &str) -> TaskDto {
        // [FIX] Use manual struct init instead of `::default()`
        let deps = DependencyDto {
            data: vec![],
            sync: vec![],
        };
        mock_task_dto(id, deps, vec![], vec![])
    }

    /// A helper to compare two vectors as sets, ignoring order.
    fn assert_vec_eq_set<T: Eq + std::hash::Hash + std::fmt::Debug>(a: &Vec<T>, b: &Vec<T>) {
        // [FIX] Correct HashSet type syntax
        let set_a: HashSet<_> = a.iter().collect();
        let set_b: HashSet<_> = b.iter().collect();
        assert_eq!(set_a, set_b);
    }

    // --- UNIT TESTS FOR HELPER FUNCTIONS ---

    #[test]
    fn test_map_reservation_variants() {
        // [FIX] Call the imported function directly, not as `Workflow::`
        // Test all states
        assert_eq!(
            map_reservation_state(ReservationStateDto::Open),
            ReservationState::Open
        );
        assert_eq!(
            map_reservation_state(ReservationStateDto::ProbeAnswer),
            ReservationState::ProbeAnswer
        );
        assert_eq!(
            map_reservation_state(ReservationStateDto::ReserveAnswer),
            ReservationState::ReserveAnswer
        );
        assert_eq!(
            map_reservation_state(ReservationStateDto::Committed),
            ReservationState::Committed
        );
        assert_eq!(
            map_reservation_state(ReservationStateDto::Finished),
            ReservationState::Finished
        );
        assert_eq!(
            map_reservation_state(ReservationStateDto::Rejected),
            ReservationState::Rejected
        );
        assert_eq!(
            map_reservation_state(ReservationStateDto::Deleted),
            ReservationState::Deleted
        );

        // Test all proceedings
        assert_eq!(
            map_reservation_proceeding(ReservationProceedingDto::Probe),
            ReservationProceeding::Probe
        );
        assert_eq!(
            map_reservation_proceeding(ReservationProceedingDto::Reserve),
            ReservationProceeding::Reserve
        );
        assert_eq!(
            map_reservation_proceeding(ReservationProceedingDto::Commit),
            ReservationProceeding::Commit
        );
        assert_eq!(
            map_reservation_proceeding(ReservationProceedingDto::Delete),
            ReservationProceeding::Delete
        );
    }

    #[test]
    fn test_phase0_build_base_workflow() {
        let dto = mock_workflow_dto(vec![]);
        let base = Workflow::build_base_workflow(&dto);

        assert_eq!(base.id, "test-wf-01");
        assert_eq!(base.arrival_time, 1000);
        assert_eq!(base.booking_interval_start, 1001);
        assert_eq!(base.booking_interval_end, 9001);
        assert_eq!(base.state, ReservationState::Open);
    }

    #[test]
    fn test_phase1_generate_workflow_nodes() {
        let tasks = vec![bare_task_dto("Node-A"), bare_task_dto("Node-B")];
        let dto = mock_workflow_dto(tasks);
        let nodes = Workflow::generate_workflow_nodes(&dto);

        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains_key("Node-A"));
        assert!(nodes.contains_key("Node-B"));

        let node_a = nodes.get("Node-A").unwrap();
        assert_eq!(node_a.reservation.base.id, "Node-A");
        assert_eq!(node_a.reservation.base.task_duration, 100);
        assert_eq!(node_a.reservation.base.reserved_capacity, 4);
        assert_eq!(node_a.reservation.base.moldable_work, 400); // 100 * 4
        assert_eq!(node_a.reservation.base.arrival_time, 1000); // Inherited from WF
    }

    #[test]
    fn test_phase1_generate_workflow_nodes_empty() {
        let dto = mock_workflow_dto(vec![]);
        let nodes = Workflow::generate_workflow_nodes(&dto);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_phase2_build_all_dependencies_data_and_sync() {
        let tasks = vec![
            mock_task_dto(
                "Node-A",
                // [FIX] Use manual struct init
                DependencyDto {
                    data: vec![],
                    sync: vec![],
                },
                vec![],
                vec![
                    DataOutDto {
                        name: "file-out".to_string(),
                        file: None,
                        size: Some(5000),
                        bandwidth: None,
                    },
                    DataOutDto {
                        name: "sync-out".to_string(),
                        file: None,
                        size: None,
                        bandwidth: Some(100),
                    },
                ],
            ),
            mock_task_dto(
                "Node-B",
                DependencyDto {
                    data: vec!["Node-A".to_string()],
                    sync: vec!["Node-A".to_string()],
                },
                vec![
                    DataInDto {
                        source_reservation: "Node-A".to_string(),
                        source_port: "file-out".to_string(),
                        file: None,
                    },
                    DataInDto {
                        source_reservation: "Node-A".to_string(),
                        source_port: "sync-out".to_string(),
                        file: None,
                    },
                ],
                vec![],
            ),
        ];
        let dto = mock_workflow_dto(tasks);
        // [FIX] `build_all_dependencies` only takes `&dto`
        let (data_deps, sync_deps) = Workflow::build_all_dependencies(&dto).unwrap();

        // Should have 2 data deps:
        // 1. From DataOut/DataIn (Node-A.file-out)
        // 2. From implicit 'pre' (Node-A -> Node-B)
        assert_eq!(data_deps.len(), 2);
        assert!(data_deps.contains_key("test-wf-01.Node-A.file-out"));
        assert!(data_deps.contains_key("test-wf-01.pre.Node-A.Node-B"));

        // Should have 2 sync deps:
        // 1. From DataOut/DataIn (Node-A.sync-out)
        // 2. From implicit 'sync' (Node-A -> Node-B)
        assert_eq!(sync_deps.len(), 2);
        assert!(sync_deps.contains_key("test-wf-01.Node-A.sync-out"));
        assert!(sync_deps.contains_key("test-wf-01.sync.Node-A.Node-B"));

        // Check details
        let data_dep = data_deps.get("test-wf-01.Node-A.file-out").unwrap();
        assert_eq!(data_dep.size, 5000);
        assert_eq!(data_dep.source_node, "Node-A");
        assert_eq!(data_dep.target_node, "Node-B");
        assert!(data_dep.reservation.base.is_moldable);

        let sync_dep = sync_deps.get("test-wf-01.Node-A.sync-out").unwrap();
        assert_eq!(sync_dep.bandwidth, 100);
        assert_eq!(sync_dep.source_node, "Node-A");
        assert_eq!(sync_dep.target_node, "Node-B");
        assert!(!sync_dep.reservation.base.is_moldable);
    }

    #[test]
    fn test_phase2_build_all_dependencies_dangling_in() {
        // Test a DataIn that points to a non-existent source
        let tasks = vec![mock_task_dto(
            "Node-A",
            // [FIX] Use manual struct init
            DependencyDto {
                data: vec![],
                sync: vec![],
            },
            vec![DataInDto {
                source_reservation: "EXTERNAL".to_string(),
                source_port: "data".to_string(),
                file: None,
            }],
            vec![],
        )];
        let dto = mock_workflow_dto(tasks);
        // [FIX] `build_all_dependencies` only takes `&dto`
        let (data_deps, sync_deps) = Workflow::build_all_dependencies(&dto).unwrap();

        // No dependency should be created, and it should not panic
        assert!(data_deps.is_empty());
        assert!(sync_deps.is_empty());
    }

    #[test]
    fn test_phase3_populate_node_adjacency_lists() {
        // [FIX] Call `WorkflowNode::from(TaskDto)`
        let mut nodes = HashMap::from([
            ("A".to_string(), WorkflowNode::from(bare_task_dto("A"))),
            ("B".to_string(), WorkflowNode::from(bare_task_dto("B"))),
        ]);
        let data_deps = HashMap::from([(
            "d1".to_string(),
            DataDependency {
                source_node: "A".to_string(),
                target_node: "B".to_string(),
                // ... other fields irrelevant for this test
                // [FIX] Use `.into()` on a DTO to trigger mock impl
                reservation: bare_task_dto("A").node_reservation.into(),
                port_name: "".to_string(),
                size: 0,
            },
        )]);
        let sync_deps = HashMap::from([(
            "s1".to_string(),
            SyncDependency {
                source_node: "B".to_string(),
                target_node: "A".to_string(),
                // ... other fields irrelevant for this test
                // [FIX] Use `.into()` on a DTO to trigger mock impl
                reservation: bare_task_dto("A").node_reservation.into(),
                port_name: "".to_string(),
                bandwidth: 0,
            },
        )]);

        Workflow::populate_node_adjacency_lists(&mut nodes, &data_deps, &sync_deps);

        let node_a = nodes.get("A").unwrap();
        let node_b = nodes.get("B").unwrap();

        assert_eq!(node_a.outgoing_data, vec!["d1"]);
        assert_eq!(node_a.incoming_data, Vec::<String>::new());
        assert_eq!(node_a.outgoing_sync, Vec::<String>::new());
        assert_eq!(node_a.incoming_sync, vec!["s1"]);

        assert_eq!(node_b.incoming_data, vec!["d1"]);
        assert_eq!(node_b.outgoing_data, Vec::<String>::new());
        assert_eq!(node_b.incoming_sync, Vec::<String>::new());
        assert_eq!(node_b.outgoing_sync, vec!["s1"]);
    }

    #[test]
    fn test_phase4_build_co_allocations_no_sync() {
        // Test case: No sync dependencies, every node is its own group.
        // [FIX] Call `WorkflowNode::from(TaskDto)`
        let nodes = HashMap::from([
            ("A".to_string(), WorkflowNode::from(bare_task_dto("A"))),
            ("B".to_string(), WorkflowNode::from(bare_task_dto("B"))),
        ]);
        let sync_deps = HashMap::new();

        let (co_allocations, node_to_co_allocation) =
            Workflow::build_co_allocations(&nodes, &sync_deps).unwrap();

        assert_eq!(co_allocations.len(), 2); // Group A, Group B
        assert!(co_allocations.contains_key("A"));
        assert!(co_allocations.contains_key("B"));
        assert_eq!(co_allocations.get("A").unwrap().members, vec!["A"]);
        assert_eq!(co_allocations.get("B").unwrap().members, vec!["B"]);

        assert_eq!(node_to_co_allocation.get("A").unwrap(), "A");
        assert_eq!(node_to_co_allocation.get("B").unwrap(), "B");
    }

    #[test]
    fn test_phase4_build_co_allocations_simple_pair() {
        // Test case: A and B are synced, C is separate.
        // [FIX] Call `WorkflowNode::from(TaskDto)`
        let nodes = HashMap::from([
            ("A".to_string(), WorkflowNode::from(bare_task_dto("A"))),
            ("B".to_string(), WorkflowNode::from(bare_task_dto("B"))),
            ("C".to_string(), WorkflowNode::from(bare_task_dto("C"))),
        ]);
        let sync_deps = HashMap::from([(
            "s1".to_string(),
            SyncDependency {
                source_node: "A".to_string(),
                target_node: "B".to_string(),
                // ...
                // [FIX] Use `.into()` on a DTO to trigger mock impl
                reservation: bare_task_dto("A").node_reservation.into(),
                port_name: "".to_string(),
                bandwidth: 0,
            },
        )]);

        let (co_allocations, node_to_co_allocation) =
            Workflow::build_co_allocations(&nodes, &sync_deps).unwrap();

        assert_eq!(co_allocations.len(), 2); // Group [A, B] and Group [C]

        let rep_ab = node_to_co_allocation.get("A").unwrap();
        let rep_c = node_to_co_allocation.get("C").unwrap();

        assert_eq!(rep_ab, node_to_co_allocation.get("B").unwrap());
        assert_ne!(rep_ab, rep_c);

        let group_ab = co_allocations.get(rep_ab).unwrap();
        let group_c = co_allocations.get(rep_c).unwrap();

        assert_vec_eq_set(&group_ab.members, &vec!["A".to_string(), "B".to_string()]);
        assert_eq!(group_ab.sync_dependencies.len(), 1); // Has the s1 dependency
        assert_vec_eq_set(&group_c.members, &vec!["C".to_string()]);
        assert_eq!(group_c.sync_dependencies.len(), 0);
    }

    #[test]
    fn test_phase4_build_co_allocations_transitive() {
        // User request: Test A->B, B->C => [A, B, C]
        // [FIX] Call `WorkflowNode::from(TaskDto)`
        let nodes = HashMap::from([
            ("A".to_string(), WorkflowNode::from(bare_task_dto("A"))),
            ("B".to_string(), WorkflowNode::from(bare_task_dto("B"))),
            ("C".to_string(), WorkflowNode::from(bare_task_dto("C"))),
            ("D".to_string(), WorkflowNode::from(bare_task_dto("D"))),
        ]);
        let sync_deps = HashMap::from([
            (
                "s_ab".to_string(),
                SyncDependency {
                    source_node: "A".to_string(),
                    target_node: "B".to_string(),
                    // ...
                    // [FIX] Use `.into()` on a DTO to trigger mock impl
                    reservation: bare_task_dto("A").node_reservation.into(),
                    port_name: "".to_string(),
                    bandwidth: 0,
                },
            ),
            (
                "s_bc".to_string(),
                SyncDependency {
                    source_node: "B".to_string(),
                    target_node: "C".to_string(),
                    // ...
                    // [FIX] Use `.into()` on a DTO to trigger mock impl
                    reservation: bare_task_dto("A").node_reservation.into(),
                    port_name: "".to_string(),
                    bandwidth: 0,
                },
            ),
        ]);

        let (co_allocations, node_to_co_allocation) =
            Workflow::build_co_allocations(&nodes, &sync_deps).unwrap();

        assert_eq!(co_allocations.len(), 2); // Group [A, B, C] and Group [D]

        let rep_abc = node_to_co_allocation.get("A").unwrap();
        let rep_d = node_to_co_allocation.get("D").unwrap();

        // Check that A, B, and C all map to the *same representative*
        assert_eq!(rep_abc, node_to_co_allocation.get("B").unwrap());
        assert_eq!(rep_abc, node_to_co_allocation.get("C").unwrap());
        assert_ne!(rep_abc, rep_d); // D is separate

        let group_abc = co_allocations.get(rep_abc).unwrap();
        assert_vec_eq_set(
            &group_abc.members,
            &vec!["A".to_string(), "B".to_string(), "C".to_string()],
        );
        assert_eq!(group_abc.sync_dependencies.len(), 2);

        let group_d = co_allocations.get(rep_d).unwrap();
        assert_vec_eq_set(&group_d.members, &vec!["D".to_string()]);
    }

    #[test]
    fn test_phase5_build_co_allocation_dependencies() {
        // Test overlay graph creation.
        // GroupX [A], GroupY [B, C]. DataDep: A -> B.
        let mut co_allocations = HashMap::from([
            (
                "A".to_string(),
                // [FIX] Call `SyncGroup::from(WorkflowNode)`
                CoAllocation::from(WorkflowNode::from(bare_task_dto("A"))),
            ),
            (
                "B".to_string(),
                // [FIX] Use struct literal
                CoAllocation {
                    id: "B".to_string(),
                    members: vec!["B".to_string(), "C".to_string()],
                    // ...
                    representative: None,
                    sync_dependencies: vec![],
                    outgoing_co_allocation_dependencies: vec![],
                    outgoing_data_dependencies: vec![],
                    incoming_co_allocation_dependencies: vec![],
                    incoming_data_dependencies: vec![],
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
                },
            ),
        ]);
        let data_deps = HashMap::from([(
            "d_ab".to_string(),
            DataDependency {
                source_node: "A".to_string(),
                target_node: "B".to_string(),
                // ...
                // [FIX] Use `.into()` on a DTO to trigger mock impl
                reservation: bare_task_dto("A").node_reservation.into(),
                port_name: "".to_string(),
                size: 0,
            },
        )]);
        let node_to_co_allocation = HashMap::from([
            ("A".to_string(), "A".to_string()),
            ("B".to_string(), "B".to_string()),
            ("C".to_string(), "B".to_string()),
        ]);

        let sg_deps = Workflow::build_co_allocation_dependencies(
            &data_deps,
            &node_to_co_allocation,
            &mut co_allocations,
        )
        .unwrap();

        // 1. Check the returned map
        assert_eq!(sg_deps.len(), 1);
        let overlay_dep = sg_deps.get("d_ab").unwrap();
        assert_eq!(overlay_dep.source_group, "A");
        assert_eq!(overlay_dep.target_group, "B");
        assert_eq!(overlay_dep.data_dependency, "d_ab");

        // 2. Check that the SyncGroups themselves were updated
        let group_a = co_allocations.get("A").unwrap();
        let group_b = co_allocations.get("B").unwrap();

        assert_eq!(group_a.outgoing_co_allocation_dependencies.len(), 1);
        assert_eq!(group_a.incoming_co_allocation_dependencies.len(), 0);
        assert_eq!(group_b.outgoing_co_allocation_dependencies.len(), 0);
        assert_eq!(group_b.incoming_co_allocation_dependencies.len(), 1);
        assert_eq!(group_a.outgoing_data_dependencies[0].source_node, "A");
        assert_eq!(group_b.incoming_data_dependencies[0].target_node, "B");
    }

    #[test]
    fn test_phase5_build_co_allocation_dependencies_ignores_internal() {
        // Test overlay graph creation.
        // GroupX [A, B]. DataDep: A -> B.
        let mut co_allocations = HashMap::from([(
            "A".to_string(),
            // [FIX] Use struct literal
            CoAllocation {
                id: "A".to_string(),
                members: vec!["A".to_string(), "B".to_string()],
                // ...
                representative: None,
                sync_dependencies: vec![],
                outgoing_co_allocation_dependencies: vec![],
                outgoing_data_dependencies: vec![],
                incoming_co_allocation_dependencies: vec![],
                incoming_data_dependencies: vec![],
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
            },
        )]);
        let data_deps = HashMap::from([(
            "d_ab".to_string(),
            DataDependency {
                source_node: "A".to_string(),
                target_node: "B".to_string(),
                // ...
                // [FIX] Use `.into()` on a DTO to trigger mock impl
                reservation: bare_task_dto("A").node_reservation.into(),
                port_name: "".to_string(),
                size: 0,
            },
        )]);
        let node_to_co_allocation = HashMap::from([
            ("A".to_string(), "A".to_string()),
            ("B".to_string(), "A".to_string()),
        ]);

        let sg_deps = Workflow::build_co_allocation_dependencies(
            &data_deps,
            &node_to_co_allocation,
            &mut co_allocations,
        )
        .unwrap();

        // 1. No overlay dependency should be created
        assert!(sg_deps.is_empty());

        // 2. The SyncGroup's lists should be empty
        let group_a = co_allocations.get("A").unwrap();
        assert!(group_a.outgoing_co_allocation_dependencies.is_empty());
        assert!(group_a.incoming_co_allocation_dependencies.is_empty());
    }

    #[test]
    fn test_phase6_find_entry_exit_points() {
        // Test a diamond graph: A -> B, A -> C, B -> D, C -> D
        // [FIX] Call `WorkflowNode::from(TaskDto)`
        let mut nodes = HashMap::from([
            ("A".to_string(), WorkflowNode::from(bare_task_dto("A"))),
            ("B".to_string(), WorkflowNode::from(bare_task_dto("B"))),
            ("C".to_string(), WorkflowNode::from(bare_task_dto("C"))),
            ("D".to_string(), WorkflowNode::from(bare_task_dto("D"))),
        ]);
        // A -> B
        nodes
            .get_mut("A")
            .unwrap()
            .outgoing_data
            .push("d_ab".to_string());
        nodes
            .get_mut("B")
            .unwrap()
            .incoming_data
            .push("d_ab".to_string());
        // A -> C
        nodes
            .get_mut("A")
            .unwrap()
            .outgoing_data
            .push("d_ac".to_string());
        nodes
            .get_mut("C")
            .unwrap()
            .incoming_data
            .push("d_ac".to_string());
        // B -> D
        nodes
            .get_mut("B")
            .unwrap()
            .outgoing_data
            .push("d_bd".to_string());
        nodes
            .get_mut("D")
            .unwrap()
            .incoming_data
            .push("d_bd".to_string());
        // C -> D
        nodes
            .get_mut("C")
            .unwrap()
            .outgoing_data
            .push("d_cd".to_string());
        nodes
            .get_mut("D")
            .unwrap()
            .incoming_data
            .push("d_cd".to_string());

        // For this test, assume 1-to-1 node-to-group mapping
        let mut co_allocations = HashMap::new();
        for id in ["A", "B", "C", "D"] {
            // [FIX] Call `SyncGroup::from(WorkflowNode)`
            let mut group = CoAllocation::from(WorkflowNode::from(bare_task_dto(id)));
            // Manually copy node adjacency to group adjacency for this test
            if id == "A" {
                group
                    .outgoing_co_allocation_dependencies
                    .push(Default::default()); // Dummy
                group
                    .outgoing_co_allocation_dependencies
                    .push(Default::default()); // Dummy
            }
            if id == "B" {
                group
                    .incoming_co_allocation_dependencies
                    .push(Default::default());
                group
                    .outgoing_co_allocation_dependencies
                    .push(Default::default());
            }
            if id == "C" {
                group
                    .incoming_co_allocation_dependencies
                    .push(Default::default());
                group
                    .outgoing_co_allocation_dependencies
                    .push(Default::default());
            }
            if id == "D" {
                group
                    .incoming_co_allocation_dependencies
                    .push(Default::default());
                group
                    .incoming_co_allocation_dependencies
                    .push(Default::default());
            }
            co_allocations.insert(id.to_string(), group);
        }

        let (entry_nodes, exit_nodes, entry_co_allocations, exit_co_allocations) =
            Workflow::find_entry_exit_points(&nodes, &co_allocations);

        assert_vec_eq_set(&entry_nodes, &vec!["A".to_string()]);
        assert_vec_eq_set(&exit_nodes, &vec!["D".to_string()]);
        assert_vec_eq_set(&entry_co_allocations, &vec!["A".to_string()]);
        assert_vec_eq_set(&exit_co_allocations, &vec!["D".to_string()]);
    }

    // --- MOCK IMPLEMENTATIONS FOR COMPILATION ---
    // [FIX] All mock impls moved *inside* the `mod tests` block.

    impl From<TaskDto> for WorkflowNode {
        fn from(task_dto: TaskDto) -> Self {
            let node_res_dto = task_dto.node_reservation;
            let node_base = crate::domain::reservation::ReservationBase {
                id: task_dto.id,
                // [FIX] Call imported function
                state: map_reservation_state(task_dto.reservation_state),
                request_proceeding: map_reservation_proceeding(task_dto.request_proceeding),
                arrival_time: 0,
                booking_interval_start: 0,
                booking_interval_end: 0,
                assigned_start: 0,
                assigned_end: 0,
                task_duration: node_res_dto.duration,
                reserved_capacity: node_res_dto.cpus,
                is_moldable: node_res_dto.is_moldable,
                moldable_work: node_res_dto.duration * node_res_dto.cpus,
            };
            Self {
                reservation: crate::domain::reservation::NodeReservation {
                    base: node_base,
                    task_path: node_res_dto.task_path,
                    output_path: node_res_dto.output_path,
                    error_path: node_res_dto.error_path,
                },
                incoming_data: Vec::new(),
                outgoing_data: Vec::new(),
                incoming_sync: Vec::new(),
                outgoing_sync: Vec::new(),
                co_allocation_key: String::new(),
            }
        }
    }

    impl From<NodeReservationDto> for crate::domain::reservation::LinkReservation {
        fn from(_: NodeReservationDto) -> Self {
            Self {
                base: crate::domain::reservation::ReservationBase {
                    id: "mock-link".to_string(),
                    // [FIX] Import and use ReservationState
                    state: ReservationState::Open,
                    // [FIX] Import and use ReservationProceeding
                    request_proceeding: ReservationProceeding::Commit,
                    arrival_time: 0,
                    booking_interval_start: 0,
                    booking_interval_end: 0,
                    assigned_start: 0,
                    assigned_end: 0,
                    task_duration: 0,
                    reserved_capacity: 0,
                    is_moldable: false,
                    moldable_work: 0,
                },
                start_point: String::new(),
                end_point: String::new(),
            }
        }
    }

    impl From<WorkflowNode> for crate::domain::co_allocation::CoAllocation {
        fn from(node: WorkflowNode) -> Self {
            Self {
                id: node.reservation.base.id.clone(),
                representative: Some(node.clone()),
                members: vec![node.reservation.base.id],
                sync_dependencies: vec![],
                outgoing_co_allocation_dependencies: vec![],
                outgoing_data_dependencies: vec![],
                incoming_co_allocation_dependencies: vec![],
                incoming_data_dependencies: vec![],
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
            }
        }
    }

    impl Default for crate::domain::dependency::CoAllocationDependency {
        fn default() -> Self {
            Self {
                id: "dummy".to_string(),
                source_group: "dummy".to_string(),
                target_group: "dummy".to_string(),
                data_dependency: "dummy".to_string(),
            }
        }
    }
}
