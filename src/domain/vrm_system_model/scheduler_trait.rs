use std::cmp::Ordering;
use std::fmt::Debug;

// TODO Add Types, if parts are implemented
type Reservations = Vec<Reservation>;
type StatisticEvent = String;
type LoadStatus = i32;
type Reservation = String;

// TODO Add Comments if trait is first implemented
// TODO Sync is potenzialy unsafe; if total struct Sync than this should be redundent
pub trait Schedule: Debug + Send + Sync {
    /// Returns the acceptedReservations
    fn get_reservations(&self) -> Reservations;
    fn generate_statistics(&self) -> StatisticEvent;
    fn get_fragmentation(&self, start_time: i64, end_time: i64) -> f64;
    fn get_system_fragmentation(&self) -> f64;
    fn get_load(&self, start_time: i64, end_time: i64) -> LoadStatus;
    fn get_simulation_load(&self) -> LoadStatus;
    fn probe(&self, reservation: &Reservation) -> Reservations;

    /// TODO Function signatur could be false please check during first implementation
    fn probe_best(
        &self,
        reservation: &Reservation,
        comparator: &dyn Fn(&Reservation, &Reservation) -> Ordering,
    ) -> Option<Reservation>;
    fn reserve(&mut self, reservation: Reservation) -> Option<Reservation>;
    fn reserve_without_check(&mut self, reservation: Reservation) -> Option<Reservation>;
    fn delete_reservation(&mut self, reservation: &Reservation) -> Option<Reservation>;
    fn clear(&mut self);
    fn update(&mut self);

    /// Returns the amount of resource units managed by this schedule
    fn get_capacity(&self) -> i64;
}

impl Clone for Box<dyn Schedule> {
    fn clone(&self) -> Box<dyn Schedule> {
        self.clone_box()
    }
}
