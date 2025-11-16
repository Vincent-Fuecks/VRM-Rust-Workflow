use vrm_rust_workflow::{
    domain::{
        client::SystemModel,
        reservation::{ReservationProceeding, ReservationState},
    },
    error::{Error, Result},
    generate_system_model,
};

/// The CoAllocation is formed by any WorkflowNodes that are linked, directly or indirectly, by a SyncDependency.
/// This test case, consits of three SyncDependencies A -> B        B -> C      D -> E
/// => CoAllocation(A,B,C) and CoAllocation(D,E)
/// Plus a DataDependency between the two CoAllocations (CoAllocation(A,B,C) and CoAllocation(D,E))
#[test]
fn test_co_allocation_graph_creation() {
    let file_path: &str = "/home/vincent/Desktop/Repository/VRM-Rust-Workflow/src/data/test/test_workflow_with_simple_co_allocation_graph.json";
    let clients: Result<SystemModel> = generate_system_model(file_path);

    if let Ok(clients) = clients {
        let clients = clients.clients;

        assert_eq!(clients.len(), 1);
        assert!(clients.contains_key("7209cffb-259f-404b-ac91-4795b4ad39e7"));
        let client = clients
            .get("7209cffb-259f-404b-ac91-4795b4ad39e7")
            .expect("System Model should contain client with this Id!");

        let workflow = client
            .workflows
            .get("Simulation-Run-0")
            .expect("Clients should contain workflow with this Id!");

        // Test CoAllocation Graph is correctly constructed
        let allowed_key_for_co_allocation_0 = vec!["A", "B", "C"];
        let allowed_key_for_co_allocation_1 = vec!["D", "E"];
        let allowed_key_for_co_allocation = vec![
            allowed_key_for_co_allocation_0,
            allowed_key_for_co_allocation_1,
        ];

        let mut co_allocation_keys_sorted: Vec<String> =
            workflow.co_allocations.keys().map(|s| s.clone()).collect();

        co_allocation_keys_sorted.sort();

        for (i, key_ref) in co_allocation_keys_sorted.iter().enumerate() {
            assert!(allowed_key_for_co_allocation[i].contains(&key_ref.as_str()));
        }

        // Test CoAllocation Grahp Exit and Entry Node
        assert!(
            allowed_key_for_co_allocation[0].contains(&workflow.entry_co_allocation[0].as_str())
        );
        assert!(
            allowed_key_for_co_allocation[1].contains(&workflow.exit_co_allocation[0].as_str())
        );

        // Test CoAllocation Dependencies
        for key_ref in workflow.co_allocation_dependencies.values() {
            assert!(key_ref.source_group == co_allocation_keys_sorted[0]);
            assert!(key_ref.target_group == co_allocation_keys_sorted[1]);
        }
    }
}

#[test]
fn test_workflow_node_creation_for_system_model() {
    let file_path: &str = "/home/vincent/Desktop/Repository/VRM-Rust-Workflow/src/data/test/test_workflow_loading_01.json";
    let clients: Result<SystemModel> = generate_system_model(file_path);

    if let Ok(clients) = clients {
        let clients = clients.clients;
        assert_eq!(clients.len(), 1);
        assert!(clients.contains_key("7209cffb-259f-404b-ac91-4795b4ad39e7"));

        let client = clients.get("7209cffb-259f-404b-ac91-4795b4ad39e7").unwrap();
        let workflows = client.workflows.clone();
        assert_eq!(workflows.len(), 1);

        assert!(workflows.contains_key("Simulation-Run-0"));
        let workflow = workflows.get("Simulation-Run-0").unwrap();

        assert!(workflow.nodes.contains_key("Data-Preprocessing-3"));
        let node = workflow.nodes.get("Data-Preprocessing-3").unwrap();

        let reservation = &node.reservation;

        // Test ReservationBase attributes
        assert_eq!(reservation.base.id, "Data-Preprocessing-3");
        assert_eq!(reservation.base.state, ReservationState::Open);
        assert_eq!(
            reservation.base.request_proceeding,
            ReservationProceeding::Commit
        );
        assert_eq!(reservation.base.arrival_time, 0);
        assert_eq!(reservation.base.booking_interval_start, 10);
        assert_eq!(reservation.base.booking_interval_end, 0);
        assert_eq!(reservation.base.assigned_start, 0);
        assert_eq!(reservation.base.assigned_end, 0);
        assert_eq!(reservation.base.task_duration, 1800);
        assert_eq!(reservation.base.reserved_capacity, 8);
        assert_eq!(reservation.base.is_moldable, false);
        assert_eq!(reservation.base.moldable_work, 14400);

        // Test NodeReservation specific attributes
        assert_eq!(reservation.task_path, Some("".to_string()));
        assert_eq!(
            reservation.output_path,
            Some("/data/logs/sim.out".to_string())
        );
        assert_eq!(
            reservation.error_path,
            Some("/data/logs/sim.err".to_string())
        );

        // Test WorkflowNode dependency lists
        // Incoming Data
        let mut incoming_data = vec![
            "Simulation-Run-0.data.Data-Preprocessing-2.Data-Preprocessing-3",
            "Simulation-Run-0.data.Data-Preprocessing-1.Data-Preprocessing-3",
        ];
        assert_eq!(node.incoming_data.clone().sort(), incoming_data.sort());

        // Outgoing Data
        let expected_outgoing_data =
            vec!["Simulation-Run-0.data.Data-Preprocessing-3.Data-Preprocessing-4"];
        assert_eq!(node.outgoing_data, expected_outgoing_data);

        // Incoming Sync
        let mut incoming_sync = vec![
            "Simulation-Run-0.sync.Data-Preprocessing-2.Data-Preprocessing-3",
            "Simulation-Run-0.sync.Data-Preprocessing-1.Data-Preprocessing-3",
        ];
        assert_eq!(node.incoming_sync.clone().sort(), incoming_sync.sort());

        // Outgoing Sync
        let outgoing_sync = vec!["Simulation-Run-0.sync.Data-Preprocessing-3.Data-Preprocessing-4"];
        assert_eq!(node.outgoing_sync, outgoing_sync);
    } else {
        assert_eq!(true, false, "Error during loading process!");
    }
}

#[test]
fn test_error_file_not_found() {
    let non_existent_file = "non_existent_file.json";

    let result = generate_system_model(non_existent_file);

    assert!(result.is_err());

    // Check that the error is the correct type
    if let Some(err) = result.err() {
        assert!(
            matches!(err, Error::IoError(_)),
            "Expected IoError, got {:?}",
            err
        );
    } else {
        panic!("Expected an error but got Ok");
    }
}
