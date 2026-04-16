use std::{sync::Arc};

use vrm_rust_workflow::{api::rms_config_dto::rms_dto::{SlurmConfigDto, SlurmRmsDto, SlurmSwitchDto}, domain::{simulator::{simulator::SystemSimulator, simulator_mock::MockSimulator}, vrm_system_model::{reservation::reservation_store::ReservationStore, rms::{advance_reservation_trait::AdvanceReservationRms, slurm::{payload::{self, task_properties::{JobProperties, TaskSubmission}}, rms_trait::SlurmRestApi, slurm::SlurmRms, slurm_rest_client::SlurmRestApiClient}}, utils::id::AciId}}};

#[tokio::test]
async fn test_is_rms_alive() {
    let slurm_rms_dummy = create_slurm_rms_mock().await;

    match slurm_rms_dummy {
        Ok(slurm_rms) => {
            let is_rms_alive = slurm_rms.slurm_rest_client.is_rms_alive().await;
            match is_rms_alive {
                Ok(is_rms_alive) => {
                    assert!(is_rms_alive, "Slurm reported it is not alive");
                
                }
                Err(e) => {
                    panic!("Docker Slurm Cluster is offline or API key is missing. Error: {}", e);
                }
            }
        }

        Err(e) => {
            panic!("Error during the create_slurm_rms_mock creation process: {}", e);
        }
    }
}

#[tokio::test]
async fn test_get_tasks() {
    let slurm_rms_dummy = create_slurm_rms_mock().await;

    match slurm_rms_dummy {
        Ok(slurm_rms) => {
            let tasks = slurm_rms.slurm_rest_client.get_tasks().await;
            match tasks {
                Ok(tasks) => {
                    for task in tasks.jobs {
                        println!("{:?}", task);
                    }
                
                }
                Err(e) => {
                    panic!("Docker Slurm Cluster is offline or API key is missing. Error: {}", e);
                }
            }
        }

        Err(e) => {
            panic!("Error during the create_slurm_rms_mock creation process: {}", e);
        }
    }
}


#[tokio::test]
async fn test_get_nodes() {
    let slurm_rms_dummy = create_slurm_rms_mock().await;

    match slurm_rms_dummy {
        Ok(slurm_rms) => {
            let nodes = slurm_rms.slurm_rest_client.get_nodes().await;
            match nodes {
                Ok(nodes) => {
                    for node in nodes.nodes {
                        println!("{:?}", node);
                    }
                }
                Err(e) => {
                    panic!("Docker Slurm Cluster is offline or API key is missing. Error: {}", e);
                }
            }
        }

        Err(e) => {
            panic!("Error during the create_slurm_rms_mock creation process: {}", e);
        }
    }
}

#[tokio::test]
async fn test_delete() {
    let slurm_rms_dummy = create_slurm_rms_mock().await;
    let task_id_to_delete: u32 = 7;

    match slurm_rms_dummy {
        Ok(slurm_rms) => {
            let is_deleted = slurm_rms.slurm_rest_client.delete(task_id_to_delete).await;
            match is_deleted {
                Ok(is_deleted) => {
                    assert!(is_deleted, "A failure during the deletion of task {:?} occurred.", task_id_to_delete);
                }
                Err(e) => {
                    panic!("Docker Slurm Cluster is offline or API key is missing. Error: {}", e);
                }
            }
        }

        Err(e) => {
            panic!("Error during the create_slurm_rms_mock creation process: {}", e);
        }
    }
}

#[tokio::test]
async fn test_commit() {
    let slurm_rms_dummy = create_slurm_rms_mock().await;
    let task_properties = JobProperties {
            name: "task-001".to_string(), 
            nodes: Some("1-2".to_string()),
            cpus_per_task: 1,
            begin: 0,
            deadline: 1000,
            memory_per_node: 256,
            current_working_directory: Some("/tmp".to_string()),
            standard_error: Some("/task-001.error".to_string()),
            standard_output: Some("/task-001.out".to_string()),
            environment: Some(vec!["PATH=/usr/bin:/bin".to_string()]),
    };

    let script = "#!/bin/bash\nhostname\nsleep 10".to_string();

    let payload = TaskSubmission {
        job: task_properties, 
        script: script,
    };

    match slurm_rms_dummy {
        Ok(slurm_rms) => {
            let slurm_task_id = slurm_rms.slurm_rest_client.commit(payload).await;
            match slurm_task_id {
                Ok(task) => {
                    println!("{:?}", task);
                }
                Err(e) => {
                    panic!("Docker Slurm Cluster is offline or API key is missing. Error: {}", e);
                }
            }
        }

        Err(e) => {
            panic!("Error during the create_slurm_rms_mock creation process: {}", e);
        }
    }
}





async fn create_slurm_rms_mock() -> Result<SlurmRms, Box<dyn std::error::Error>> {
        let simulator: Arc<dyn SystemSimulator> = Arc::new(MockSimulator::new(0));
    let aci_id = AciId::new("Test-AcI");
    let reservation_store = ReservationStore::new();
    let rest_api_config: SlurmConfigDto = SlurmConfigDto { 
        base_url: "http://localhost:6820".to_string(), 
        version: "v0.0.41".to_string(), 
        user_name: "root".to_string(), 
        jwt_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjIwOTE1MDYxNDQsImlhdCI6MTc3NjE0NjE0NCwic3VuIjoic2x1cm0ifQ.1c0D0fH2bP9MS3qmwf944xH9894r_aeaHFgnGaMYw-Q".to_string() 
    };
    
    
// 2. Define the individual switches for the topology
    let slurm_switch_dto_0 = SlurmSwitchDto {
        switch_name: "s0".to_string(),
        switches: vec![],
        nodes: vec!["c0".to_string(), "c1".to_string()],
        link_speed: 1000,
    };

    let slurm_switch_dto_1 = SlurmSwitchDto {
        switch_name: "s1".to_string(),
        switches: vec![],
        nodes: vec![
            "c3".to_string(),
            "c4".to_string(),
            "c5".to_string(),
            "c6".to_string(),
        ],
        link_speed: 1000,
    };

    let slurm_switch_dto_2 = SlurmSwitchDto {
        switch_name: "s2".to_string(),
        switches: vec!["s0".to_string(), "s1".to_string()],
        nodes: vec!["c2".to_string()],
        link_speed: 1000,
    };

    // 3. Assemble the topology vector
    let topology: Vec<SlurmSwitchDto> = vec![
        slurm_switch_dto_0,
        slurm_switch_dto_1,
        slurm_switch_dto_2,
    ];

    let slurm_rms_dto: SlurmRmsDto = SlurmRmsDto { id: "RMS-ID".to_string(), scheduler_typ: "SlottedSchedule".to_string(), slot_width: 60, num_of_slots: 60, rest_api_config: rest_api_config, topology: topology};
    
    return SlurmRms::new(slurm_rms_dto, simulator, aci_id, reservation_store).await;
}