use vrm_rust_workflow::{
    domain::client::SystemModel, 
    error::{Error, Result}, 
    domain::reservation::{ReservationState, ReservationProceeding},
    generate_system_model
};

#[test]
fn test_generate_system_model() {
    let file_path: &str = "/home/vincent/Desktop/Repository/VRM-Rust-Workflow/src/data/test/test_workflow_loading_01.json";
    let clients: Result<SystemModel> = generate_system_model(file_path);

    if let Ok(clients) = clients {
        let clients = clients.clients;
        assert_eq!(clients.len(), 1);
        assert!(clients.contains_key("7209cffb-259f-404b-ac91-4795b4ad39e7"));
        
        let client= clients.get("7209cffb-259f-404b-ac91-4795b4ad39e7").unwrap();
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
        assert_eq!(reservation.base.request_proceeding, ReservationProceeding::Commit);
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
        assert_eq!(reservation.output_path, Some("/data/logs/sim.out".to_string()));
        assert_eq!(reservation.error_path, Some("/data/logs/sim.err".to_string()));

        // Test WorkflowNode dependency lists
        // Incoming Data
        let mut incoming_data = vec![
            "Simulation-Run-0.pre.Data-Preprocessing-2.Data-Preprocessing-3",
            "Simulation-Run-0.pre.Data-Preprocessing-1.Data-Preprocessing-3",
        ];
        assert_eq!(node.incoming_data.clone().sort(), incoming_data.sort());
        
        // Outgoing Data
        let expected_outgoing_data = vec![
            "Simulation-Run-0.pre.Data-Preprocessing-3.Data-Preprocessing-4",
        ];
        assert_eq!(node.outgoing_data, expected_outgoing_data);
        
        // Incoming Sync
        let mut incoming_sync = vec![
            "Simulation-Run-0.sync.Data-Preprocessing-2.Data-Preprocessing-3",
            "Simulation-Run-0.sync.Data-Preprocessing-1.Data-Preprocessing-3",
        ];
        assert_eq!(node.incoming_sync.clone().sort(), incoming_sync.sort());

        // Outgoing Sync
        let outgoing_sync = vec![
            "Simulation-Run-0.sync.Data-Preprocessing-3.Data-Preprocessing-4",
        ];
        assert_eq!(node.outgoing_sync, outgoing_sync);

        println!("Workflow-Details (Pretty Debug):\n{:#?}", node);
    }
}

#[test]
fn test_error_file_not_found() {
    let non_existent_file = "non_existent_file.json";
    
    let result = generate_system_model(non_existent_file);

    assert!(result.is_err());

    // Check that the error is the correct type
    if let Some(err) = result.err() {
        assert!(matches!(err, Error::IoError(_)), "Expected IoError, got {:?}", err);
    } else {
        panic!("Expected an error but got Ok");
    }
}