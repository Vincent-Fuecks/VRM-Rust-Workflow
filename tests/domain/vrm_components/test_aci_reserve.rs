use std::sync::Arc;

use vrm_rust_workflow::api::rms_config_dto::rms_dto::{DummyRmsDto, GridNodeDto, NetworkLinkDto, RmsSystemWrapper};
use vrm_rust_workflow::api::vrm_system_model_dto::aci_dto::AcIDto;
use vrm_rust_workflow::domain::simulator::simulator::GlobalClock;
use vrm_rust_workflow::domain::vrm_system_model::grid_resource_management_system::aci::AcI;
use vrm_rust_workflow::domain::vrm_system_model::grid_resource_management_system::vrm_component_trait::VrmComponent;
use vrm_rust_workflow::domain::vrm_system_model::reservation::node_reservation::NodeReservation;
use vrm_rust_workflow::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationBase, ReservationProceeding, ReservationState};
use vrm_rust_workflow::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use vrm_rust_workflow::domain::vrm_system_model::utils::id::{ClientId, ReservationName};

/// Try normal reserve request
#[tokio::test]
async fn test_reserve() {
    let clock = Arc::new(GlobalClock::new(true));
    let store = ReservationStore::new();
    let res_name = ReservationName::new("test_job_to_reserve".to_string());
    let mut aci = create_dummy_aci(clock.clone(), store.clone()).await;
    let node_reservation = create_node_reservation(res_name, 2, 0, 5, ReservationState::Open, clock);
    let res_id = store.add(node_reservation);
    let _ = aci.reserve(res_id, None);

    assert_eq!(store.get_state(res_id), ReservationState::ReserveAnswer, "Reservation process was not successful.");
}

/// Test Reserve with false reservation state
#[tokio::test]
async fn test_reserve_with_false_state() {
    let clock = Arc::new(GlobalClock::new(true));
    let store = ReservationStore::new();
    let res_name = ReservationName::new("test_job_to_reserve".to_string());
    let mut aci = create_dummy_aci(clock.clone(), store.clone()).await;
    let node_reservation = create_node_reservation(res_name, 2, 0, 5, ReservationState::Deleted, clock);
    let res_id = store.add(node_reservation);
    let _ = aci.reserve(res_id, None);

    assert_eq!(store.get_state(res_id), ReservationState::Rejected, "Reservation process was not successful.");
}

/// Request more capacity a single compute node has max is 256 request 500
#[tokio::test]
async fn test_reserve_exceeds_capacity() {
    let clock = Arc::new(GlobalClock::new(true));
    let store = ReservationStore::new();
    let mut aci = create_dummy_aci(clock.clone(), store.clone()).await;
    let res_name = ReservationName::new("test_job_over_capacity".to_string());
    let res_id = store.add(create_node_reservation(res_name, 500, 100, 700, ReservationState::Open, clock.clone()));

    let _ = aci.reserve(res_id, None);
    assert_eq!(store.get_state(res_id), ReservationState::Rejected, "Reservation should be Rejected when exceeding capacity.");
}

/// Request reserve for reservation with negative reserved capacity --> ReservationState::Rejected
#[tokio::test]
async fn test_reserve_negative_capacity() {
    let clock = Arc::new(GlobalClock::new(true));
    let store = ReservationStore::new();
    let mut aci = create_dummy_aci(clock.clone(), store.clone()).await;
    let res_name = ReservationName::new("test_job_over_negative_capacity".to_string());

    let res_id = store.add(create_node_reservation(res_name, -10, 100, 700, ReservationState::Open, clock.clone()));

    let _ = aci.reserve(res_id, None);

    assert_eq!(store.get_state(res_id), ReservationState::Rejected, "Reservation should be Rejected for negative capacity.");
}

/// Request reserve of Reservation with invalid reservation start time. 
#[tokio::test]
async fn test_reserve_before_start_time() {
    let clock = Arc::new(GlobalClock::new(true));
    let store = ReservationStore::new();
    let mut aci = create_dummy_aci(clock.clone(), store.clone()).await;
    let res_name = ReservationName::new("test_job_past_reservation".to_string());

    // Requesting a window that starts at -500 and ends at -100
    let res_id = store.add(create_node_reservation(res_name, 1, -500, -100, ReservationState::Open, clock.clone()));

    let _ = aci.reserve(res_id, None);

    assert_eq!(store.get_state(res_id), ReservationState::Rejected, "Reservation should be Rejected if scheduled in the past.");
}
/// Reservations are only excepted, if they are in site the slot window
/// Slot window: slot_width * num_of_slots
#[tokio::test]
async fn test_reserve_of_reservation_outside_slot_window() {
    let clock = Arc::new(GlobalClock::new(true));
    let store = ReservationStore::new();
    let mut aci = create_dummy_aci(clock.clone(), store.clone()).await;
    let res_name = ReservationName::new("test_job_out_site_slot_window".to_string());

    let start_in_site_slot_window = (60 * 10) - 1;
    let end_out_site_slot_window = start_in_site_slot_window + 10;

    let res_id =
        store.add(create_node_reservation(res_name, 1, start_in_site_slot_window, end_out_site_slot_window, ReservationState::Open, clock.clone()));

    let _ = aci.reserve(res_id, None);

    // Assuming the RMS can handle distant horizons, this should succeed
    assert_eq!(store.get_state(res_id), ReservationState::Rejected, "AcI should be able to handle reservations far in the future.");
}

/// Reservations still in slot window
/// Slot window: slot_width * num_of_slots
#[tokio::test]
async fn test_reserve_of_reservation_still_in_slot_window() {
    let clock = Arc::new(GlobalClock::new(true));
    let store = ReservationStore::new();
    let mut aci = create_dummy_aci(clock.clone(), store.clone()).await;
    let res_name = ReservationName::new("test_job_out_site_slot_window".to_string());
    let start_in_site_slot_window = (60 * 10) - 10;
    let end_in_site_slot_window = start_in_site_slot_window + 10;

    let res_id =
        store.add(create_node_reservation(res_name, 1, start_in_site_slot_window, end_in_site_slot_window, ReservationState::Open, clock.clone()));

    let _ = aci.reserve(res_id, None);

    // Assuming the RMS can handle distant horizons, this should succeed
    assert_eq!(store.get_state(res_id), ReservationState::ReserveAnswer, "AcI should be able to handle reservations far in the future.");
}

fn create_node_reservation(
    res_name: ReservationName,
    capacity: i64,
    start: i64,
    end: i64,
    reservation_state: ReservationState,
    clock: Arc<GlobalClock>,
) -> Reservation {
    let client_id = ClientId::new("test_client".to_string());
    let duration = end - start;

    let base = ReservationBase {
        name: res_name.clone(),
        client_id,
        handler_id: None,
        state: reservation_state,
        request_proceeding: ReservationProceeding::Commit,
        arrival_time: clock.get_system_time_s(),
        booking_interval_start: start,
        booking_interval_end: end,
        assigned_start: start,
        assigned_end: end,
        task_duration: duration,
        reserved_capacity: capacity,
        is_moldable: false,
        moldable_work: duration,
        frag_delta: 0.0,
    };

    let node_res = NodeReservation {
        base,
        current_working_directory: Some("/tmp".to_string()),
        environment: Some(vec!["PATH=/usr/bin:/bin".to_string()]),
        task_path: "/bin/sleep".to_string(),
        output_path: Some("/tmp/slurm_test.out".to_string()),
        error_path: Some("/tmp/slurm_test.err".to_string()),
    };

    return Reservation::Node(node_res);
}

async fn create_dummy_aci(clock: Arc<GlobalClock>, reservation_store: ReservationStore) -> AcI {
    let grid_nodes = vec![
        GridNodeDto { id: "Node-001".to_string(), cpus: 256, connected_to_router: vec!["Router-001".to_string()] },
        GridNodeDto { id: "Node-002".to_string(), cpus: 256, connected_to_router: vec!["Router-002".to_string()] },
        GridNodeDto { id: "Node-003".to_string(), cpus: 256, connected_to_router: vec!["Router-003".to_string()] },
        GridNodeDto { id: "Node-004".to_string(), cpus: 256, connected_to_router: vec!["Router-001".to_string(), "Router-003".to_string()] },
    ];

    let network_links = vec![
        NetworkLinkDto {
            id: "Router-001--To--Router-002".to_string(),
            start_point: "Router-001".to_string(),
            end_point: "Router-002".to_string(),
            capacity: 10000,
        },
        NetworkLinkDto {
            id: "Router-001--To--Router-003".to_string(),
            start_point: "Router-001".to_string(),
            end_point: "Router-003".to_string(),
            capacity: 10000,
        },
        NetworkLinkDto {
            id: "Router-002--To--Router-001".to_string(),
            start_point: "Router-002".to_string(),
            end_point: "Router-001".to_string(),
            capacity: 5000,
        },
        NetworkLinkDto {
            id: "Router-002--To--Router-003".to_string(),
            start_point: "Router-002".to_string(),
            end_point: "Router-003".to_string(),
            capacity: 5000,
        },
    ];

    let dummy_rms_dto = DummyRmsDto {
        typ: "RmsNodeSimulator".to_string(),
        scheduler_typ: "SlottedSchedule".to_string(),
        num_of_slots: 10,
        slot_width: 60,
        grid_nodes,
        network_links,
    };

    let rms_system = RmsSystemWrapper::DummyRms(dummy_rms_dto);

    let dto = AcIDto { adc_id: "Adc-001".to_string(), commit_timeout: 256, id: "AcI-001".to_string(), rms_system: rms_system };

    return AcI::from_dto(dto, clock, reservation_store).await.expect("Error in the AcI Mock process happened.");
}
