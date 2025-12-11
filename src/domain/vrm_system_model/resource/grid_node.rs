use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;

#[derive(Debug, Clone)]
pub struct GridNode {
    pub id: ReservationKey,
    pub cpus: i64,
    pub connected_to_router: Vec<ReservationKey>,
}
