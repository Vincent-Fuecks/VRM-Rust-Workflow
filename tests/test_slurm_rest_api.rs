use std::sync::Arc;

use vrm_rust_workflow::{api::rms_config_dto::rms_dto::{SlurmConfigDto, SlurmRmsDto, SlurmSwitchDto}, domain::{simulator::{simulator::SystemSimulator, simulator_mock::MockSimulator}, vrm_system_model::{reservation::reservation_store::ReservationStore, rms::{advance_reservation_trait::AdvanceReservationRms, slurm::{rms_trait::SlurmRestApi, slurm::SlurmRms, slurm_rest_client::SlurmRestApiClient}}, utils::id::AciId}}};

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

async fn create_slurm_rms_mock() -> Result<SlurmRms, Box<dyn std::error::Error>> {
        let simulator: Arc<dyn SystemSimulator> = Arc::new(MockSimulator::new(0));
    let aci_id = AciId::new("Test-AcI");
    let reservation_store = ReservationStore::new();
    let rest_api_config: SlurmConfigDto = SlurmConfigDto { 
        base_url: "http://localhost:6820".to_string(), 
        version: "v0.0.41".to_string(), 
        user_name: "root".to_string(), 
        jwt_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjIwOTE0MjU2ODIsImlhdCI6MTc3NjA2NTY4Miwic3VuIjoic2x1cm0ifQ.FYYCgbSPHz54xMkSHg5KSt8d2UbYLTy9jeE3OeDGFDw".to_string() 
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