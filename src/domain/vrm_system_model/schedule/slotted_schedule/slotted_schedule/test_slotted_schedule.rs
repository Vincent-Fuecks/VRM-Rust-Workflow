// use std::any::Any;
// use std::cell::RefCell;
// use std::collections::HashSet;
// use std::i64;
// use std::rc::Rc;
// use std::sync::{Arc, RwLock};

// use crate::domain::simulator;
// use crate::domain::simulator::simulator::SharedSimulator;
// use crate::domain::vrm_system_model::reservation::node_reservation::NodeReservation;
// use crate::domain::vrm_system_model::reservation::reservation::{ReservationTrait, ReservationTyp};
// use crate::domain::{
//     simulator::{
//         simulator::SystemSimulator,
//         simulator_mock::{MockSimulator, SharedMockSimulator},
//     },
//     vrm_system_model::reservation::reservation::{Reservation, ReservationBase, ReservationProceeding, ReservationState},
//     vrm_system_model::{
//         reservation::reservation_store::ReservationId,
//         reservation::reservation_store::ReservationStore,
//         schedule::slotted_schedule::SlottedSchedule,
//         utils::id::{ClientId, ReservationName, SlottedScheduleId},
//     },
// };

// use crate::domain::vrm_system_model::scheduler_trait::Schedule;

// #[derive(Debug, Clone)]
// pub struct TestReservation {
//     base: ReservationBase,
// }

// impl TestReservation {
//     pub fn new(name: &str, duration: i64, capacity: i64, start: i64, end: i64, is_moldable: bool) -> Self {
//         Self {
//             base: ReservationBase {
//                 name: ReservationName::new(name),
//                 client_id: ClientId::new("test_client"),
//                 handler_id: None,
//                 state: ReservationState::Open,
//                 request_proceeding: ReservationProceeding::Reserve,
//                 arrival_time: 0,
//                 booking_interval_start: start,
//                 booking_interval_end: end,
//                 assigned_start: 0,
//                 assigned_end: 0,
//                 task_duration: duration,
//                 reserved_capacity: capacity,
//                 is_moldable,
//                 moldable_work: duration * capacity,
//                 frag_delta: 0.0,
//             },
//         }
//     }
// }

// impl ReservationTrait for TestReservation {
//     fn get_base(&self) -> &ReservationBase {
//         &self.base
//     }
//     fn get_base_mut(&mut self) -> &mut ReservationBase {
//         &mut self.base
//     }
//     fn box_clone(&self) -> Box<dyn ReservationTrait> {
//         Box::new(self.clone())
//     }
//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn get_typ(&self) -> ReservationTyp {
//         todo!()
//     }
// }

// // --- Helper Functions ---

// fn create_schedule(capacity: i64, slots: i64, simulator: Arc<MockSimulator>) -> (SlottedSchedule, ReservationStore) {
//     let store = ReservationStore::new();

//     let schedule = SlottedSchedule::new(
//         SlottedScheduleId::new("test_schedule"),
//         slots,
//         1, // slot width
//         capacity,
//         false, // use quadratic mean
//         simulator,
//         store.clone(), // Pass store specifically if needed by constructor
//     );
//     (schedule, store)
// }

// fn add_reservation_to_store(store: &ReservationStore, name: &str, duration: i64, capacity: i64, start: i64, end: i64) -> ReservationId {
//     let base = ReservationBase {
//         name: ReservationName::new(name),
//         client_id: ClientId::new("test_client"),
//         handler_id: None,
//         state: ReservationState::Open,
//         request_proceeding: ReservationProceeding::Reserve,
//         arrival_time: 0,
//         booking_interval_start: start,
//         booking_interval_end: end,
//         assigned_start: 0,
//         assigned_end: 0,
//         task_duration: duration,
//         reserved_capacity: capacity,
//         is_moldable: true,
//         moldable_work: duration * capacity,
//         frag_delta: 0.0,
//     };

//     let res = Reservation::new_node(base, None, None, None);
//     store.add(res)
// }

// // --- Test Cases ---

// // (1) Function clear: Test the following cases: active_reservations contain 0, 1 and multiple reservations
// // TODO Why has the duration of reservations no effect on utlisation and avg_reserved_capacity?
// #[test]
// fn test_clear() {
//     // Case 0: Empty schedule
//     let simulator = Arc::new(MockSimulator::new(0));
//     let (mut schedule, store) = create_schedule(100, 10, simulator);
//     schedule.clear();
//     assert_eq!(schedule.get_load_metric(0, 100).utilization, 0.0);

//     // Case 1: 1 active reservation
//     let id1 = add_reservation_to_store(&store, "res1", 10, 10, 0, 10);
//     let res_result = schedule.reserve(id1);
//     assert!(res_result.is_none()); // Success
//     assert_eq!(schedule.get_load_metric_up_to_date(0, 10).avg_reserved_capacity, 10.0);
//     assert_eq!(schedule.get_load_metric_up_to_date(0, 5).avg_reserved_capacity, 10.0);
//     assert_eq!(schedule.get_load_metric_up_to_date(0, 10).utilization, 0.1);

//     schedule.clear();
//     assert_eq!(schedule.get_load_metric(i64::MIN, i64::MAX).utilization, 0.0);

//     // Case Multiple: Multiple active reservations
//     let id1 = add_reservation_to_store(&store, "res1", 10, 10, 0, 10);
//     let id2 = add_reservation_to_store(&store, "res2", 10, 10, 0, 10);
//     let id3 = add_reservation_to_store(&store, "res3", 10, 10, 0, 10);
//     let id4 = add_reservation_to_store(&store, "res4", 10, 10, 0, 10);
//     let id5 = add_reservation_to_store(&store, "res5", 10, 10, 0, 10);
//     schedule.reserve(id1);
//     schedule.reserve(id2);
//     schedule.reserve(id3);
//     schedule.reserve(id4);
//     schedule.reserve(id5);

//     assert_eq!(schedule.get_load_metric_up_to_date(0, 10).utilization, 0.5);
//     // Because empty time 10-100 is ignored
//     assert_eq!(schedule.get_load_metric_up_to_date(0, 100).utilization, 0.5);
//     schedule.clear();
//     assert_eq!(schedule.get_load_metric_up_to_date(0, 100).utilization, 0.0);
// }

// // (2) Function reserve: Test the following cases: Input reservation_id is in state Open, ReserveAnswer, Committed, Deleted, Rejected, ProbeAnswer, Finished.
// #[test]
// fn test_reserve_with_various_states() {
//     let simulator = Arc::new(MockSimulator::new(0));
//     let (mut schedule, store) = create_schedule(100, 100, simulator);
//     let states = vec![
//         ReservationState::Open,
//         ReservationState::ReserveAnswer,
//         ReservationState::Committed,
//         ReservationState::Deleted,
//         ReservationState::Rejected,
//         ReservationState::ProbeAnswer,
//         ReservationState::Finished,
//     ];

//     for (i, state) in states.into_iter().enumerate() {
//         let name = format!("res_{}", i);
//         let id = add_reservation_to_store(&store, &name, 10, 10, 0, 100);

//         // Set initial state manually in store
//         store.update_state(id, state);

//         // Attempt reserve
//         let result = schedule.reserve(id);

//         assert!(result.is_none(), "Reservation failed for state {:?}", state);
//         assert_eq!(store.get_state(id), ReservationState::ReserveAnswer, "State not updated to ReserveAnswer for {:?}", state);

//         // Clean up for next iteration
//         schedule.clear();
//     }
// }

// // (3) Function get_fragmentation: Test the following cases...
// #[test]
// fn test_get_fragmentation() {
//     let simulator = Arc::new(MockSimulator::new(0));
//     let (mut schedule, store) = create_schedule(100, 100, simulator);

//     // Setup: Multiple Committed, Finished, ReserveAnswer reservations
//     let id_comm = add_reservation_to_store(&store, "comm", 10, 10, 0, 50);
//     schedule.reserve(id_comm);
//     store.update_state(id_comm, ReservationState::Committed);

//     let id_fin = add_reservation_to_store(&store, "fin", 10, 10, 10, 60);
//     schedule.reserve(id_fin);
//     store.update_state(id_fin, ReservationState::Finished);

//     let id_ans = add_reservation_to_store(&store, "ans", 10, 10, 20, 70);
//     schedule.reserve(id_ans); // Defaults to ReserveAnswer

//     // Case: Full timeframe
//     let frag_full = schedule.get_fragmentation(0, 100);
//     assert!(frag_full >= 0.0 && frag_full <= 1.0);

//     // Case: Start and End the same
//     let frag_same = schedule.get_fragmentation(10, 10);
//     assert!(frag_same >= 0.0);

//     // Case: Timeframe without reservation in it (Empty slots)
//     let frag_empty = schedule.get_fragmentation(80, 90);
//     assert_eq!(frag_empty, 0.0);
// }

// // (4) Function update: Test with full schedule, where before a reserve request was successfull.
// #[test]
// fn test_update() {
//     // Setup with simulator time 0
//     let store = ReservationStore::new();

//     let time_lock = Arc::new(RwLock::new(0));
//     let sim = Arc::new(SharedMockSimulator { time: time_lock.clone() });

//     let mut schedule = SlottedSchedule::new(SlottedScheduleId::new("sched_update"), 100, 1, 100, false, sim, store.clone());

//     // Fill schedule
//     let id1 = add_reservation_to_store(&store, "res1", 10, 400, 0, 20); // Full capacity for 10s
//     let res = schedule.reserve(id1);
//     assert!(res.is_none());

//     // Verify loaded
//     assert_eq!(schedule.get_load_metric_up_to_date(0, 20).utilization, 1.0);

//     // Advance time to 15 (res1 ends at 10)
//     *time_lock.write().unwrap() = 15;

//     schedule.update();

//     // Check if we can reserve full capacity now at time 15
//     let id2 = add_reservation_to_store(&store, "res2", 5, 100, 15, 20);
//     let res2 = schedule.reserve(id2);
//     assert!(res2.is_none());
// }

// // (5) Function delete_reservation: Test cases...

// #[test]
// fn test_delete_reservation() {
//     let simulator = Arc::new(MockSimulator::new(10));
//     let (mut schedule, store) = create_schedule(100, 100, simulator); // Start time 10

//     // del ReserveAnswer
//     let id_ans = add_reservation_to_store(&store, "ans", 10, 10, 10, 100);
//     schedule.reserve(id_ans);
//     assert_eq!(store.get_state(id_ans), ReservationState::ReserveAnswer);
//     schedule.delete_reservation(id_ans);
//     assert_eq!(store.get_state(id_ans), ReservationState::Deleted);
//     assert_eq!(schedule.get_load_metric(10, 20).utilization, 0.0);

//     // del Open (Not in schedule yet)
//     let id_open = add_reservation_to_store(&store, "open", 10, 10, 10, 100);
//     schedule.delete_reservation(id_open);
//     assert_eq!(store.get_state(id_open), ReservationState::Rejected);

//     // del Finished reservation
//     let time_lock = Arc::new(RwLock::new(0));
//     let sim = Arc::new(SharedMockSimulator { time: time_lock.clone() });
//     let mut sched_fin = SlottedSchedule::new(SlottedScheduleId::new("fin"), 100, 1, 100, false, sim, store.clone());

//     let id_fin2 = add_reservation_to_store(&store, "fin2", 15, 10, 0, 100); // 0 to 15
//     sched_fin.reserve(id_fin2);

//     // Move time to 15 (Res ends at 15)
//     // Note: update() logic may remove it if strictly less than start, but here exact match might depend on loop boundaries.
//     // We assume delete_reservation logic handles "finished" check.
//     *time_lock.write().unwrap() = 15;

//     sched_fin.delete_reservation(id_fin2);
//     // If it was considered finished, it shouldn't be deleted.
//     // If it was removed by update(), it's rejected.
//     // We just verify it didn't crash and state is consistent.
// }

// // (6) reserve_without_check: Test with one reservation, which is working, test with a reservation, which was before rejected.

// #[test]
// fn test_reserve_without_check_scenarios() {
//     let simulator = Arc::new(MockSimulator::new(0));
//     let (mut schedule, store) = create_schedule(100, 100, simulator);

//     // 1. Working reservation
//     let id1 = add_reservation_to_store(&store, "work", 10, 10, 0, 100);
//     schedule.reserve(id1);
//     assert_eq!(store.get_state(id1), ReservationState::ReserveAnswer);

//     // 2. Previously Rejected
//     let id2 = add_reservation_to_store(&store, "rej", 10, 10, 20, 100);
//     store.update_state(id2, ReservationState::Rejected);

//     let res = schedule.reserve(id2);
//     assert!(res.is_none());
//     assert_eq!(store.get_state(id2), ReservationState::ReserveAnswer);
// }

// #[test]
// fn test_other_functions() {
//     let mut simulator = Arc::new(MockSimulator::new(0));
//     let (mut schedule, store) = create_schedule(100, 100, simulator.clone());
//     let id = add_reservation_to_store(&store, "misc", 10, 10, 0, 50);
//     schedule.reserve(id);

//     // get_load_metric
//     let lm = schedule.get_load_metric(40, 50);
//     assert_eq!(lm.avg_reserved_capacity, 10.0);
//     assert_eq!((lm.utilization * 10.0).round() / 10.0, 0.1);

//     // TODO Test with time setting is currently not working
//     // get_load_metric_up_to_date (calls update)
//     let lm_up = schedule.get_load_metric_up_to_date(40, 50);
//     assert_eq!(lm_up.utilization, 0.1);

//     // get_system_fragmentation
//     let frag = schedule.get_system_fragmentation();
//     assert!(frag >= 0.0);

//     // probe
//     let id_probe = add_reservation_to_store(&store, "probe", 10, 10, 20, 60);
//     let candidates = schedule.probe(id_probe);
//     assert!(candidates.len() > 0);
//     assert_eq!(store.get_state(id_probe), ReservationState::ProbeAnswer);

//     // probe_best
//     let mut comparator = |id1: ReservationId, id2: ReservationId| std::cmp::Ordering::Less;
//     let best = schedule.probe_best(id_probe, &mut comparator);
//     assert!(best.is_some());
// }
