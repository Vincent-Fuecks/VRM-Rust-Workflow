use crate::reservation::ReservationBase;

pub enum RequestType {
    Compute,
    DataTransfer,
    SyncDataTransfer,
}

pub trait Schedulable {
    fn assigned_reservation(&self) -> &ReservationBase;
    fn request_type(&self) -> RequestType;
}