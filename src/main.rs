mod workflow;
mod loader;
mod error;
mod logger;
mod traits;

use crate::workflow::reservation::ReservationBase;

fn main() {
    let reservation = ReservationBase::new("/home/vincent/Desktop/Repository/VRM-Rust-Workflow/src/data/exampleReservation.json");
    println!("Successfully loaded reservation: {:?}", reservation);
}